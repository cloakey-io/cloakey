use anyhow::{anyhow, Result};
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use windows::core::{Interface, PCWSTR};
use windows::Win32::Foundation::{LPARAM, WPARAM};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, IPersistFile,
};
use windows::Win32::System::Registry::{
    RegCloseKey, RegCreateKeyExW, RegDeleteKeyW, RegOpenKeyExW, RegQueryValueExW, RegSetValueExW,
    HKEY, HKEY_CURRENT_USER, KEY_READ, KEY_WRITE, REG_OPTION_NON_VOLATILE, REG_SZ, REG_VALUE_TYPE,
};
use windows::Win32::UI::Shell::{IShellLinkW, ShellLink};
use windows::Win32::UI::WindowsAndMessaging::{
    SendMessageTimeoutW, HWND_BROADCAST, SMTO_ABORTIFHUNG, WM_SETTINGCHANGE,
};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let uninstall = args.iter().any(|arg| arg == "--uninstall");

    if uninstall {
        run_uninstall()?;
    } else {
        run_install()?;
    }

    Ok(())
}

fn to_wide_str(str: &OsStr) -> Vec<u16> {
    str.encode_wide().chain(std::iter::once(0)).collect()
}

fn get_install_dir() -> Result<PathBuf> {
    let local_app_data = dirs::data_local_dir()
        .or_else(|| env::var_os("LOCALAPPDATA").map(PathBuf::from))
        .or_else(|| env::var_os("USERPROFILE").map(|p| PathBuf::from(p).join("AppData").join("Local")))
        .or_else(|| env::var_os("ALLUSERSPROFILE").map(PathBuf::from))
        .or_else(|| env::var_os("ProgramData").map(PathBuf::from))
        .ok_or_else(|| anyhow!("Failed to locate Local AppData directory"))?;
    Ok(local_app_data.join("Programs").join("CloaKey"))
}

fn get_start_menu_shortcut_path() -> Result<PathBuf> {
    let app_data = dirs::data_dir()
        .or_else(|| env::var_os("APPDATA").map(PathBuf::from))
        .or_else(|| env::var_os("USERPROFILE").map(|p| PathBuf::from(p).join("AppData").join("Roaming")))
        .or_else(|| env::var_os("ALLUSERSPROFILE").map(|p| PathBuf::from(p).join("Microsoft").join("Windows").join("Start Menu").join("Programs")))
        .ok_or_else(|| anyhow!("Failed to locate AppData directory"))?;
    
    if app_data.ends_with("Programs") {
        Ok(app_data.join("CloaKey.lnk"))
    } else {
        Ok(app_data
            .join("Microsoft")
            .join("Windows")
            .join("Start Menu")
            .join("Programs")
            .join("CloaKey.lnk"))
    }
}

fn create_shortcut(link_path: &Path, target_path: &Path, work_dir: &Path, icon_path: &Path) -> Result<()> {
    unsafe {
        let hr = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        let com_initialized = hr.is_ok();

        let shell_link: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER)?;

        let target_w = to_wide_str(target_path.as_os_str());
        let work_dir_w = to_wide_str(work_dir.as_os_str());
        let icon_w = to_wide_str(icon_path.as_os_str());

        shell_link.SetPath(PCWSTR(target_w.as_ptr()))?;
        shell_link.SetWorkingDirectory(PCWSTR(work_dir_w.as_ptr()))?;
        shell_link.SetIconLocation(PCWSTR(icon_w.as_ptr()), 0)?;

        let persist_file: IPersistFile = shell_link.cast()?;
        let link_w = to_wide_str(link_path.as_os_str());
        persist_file.Save(PCWSTR(link_w.as_ptr()), true)?;

        if com_initialized {
            CoUninitialize();
        }
    }
    Ok(())
}

fn reg_get_string(hkey: HKEY, sub_key: &str, name: &str) -> Result<Option<String>> {
    unsafe {
        let sub_key_w = to_wide_str(OsStr::new(sub_key));
        let mut hsub_key = HKEY::default();
        let status = RegOpenKeyExW(
            hkey,
            PCWSTR(sub_key_w.as_ptr()),
            0,
            KEY_READ,
            &mut hsub_key,
        );
        if status.0 != 0 {
            if status.0 == 2 { // ERROR_FILE_NOT_FOUND
                return Ok(None);
            }
            return Err(anyhow!("Failed to open registry key: status {}", status.0));
        }

        let name_w = to_wide_str(OsStr::new(name));
        let mut value_type = REG_VALUE_TYPE::default();
        let mut buf_len = u32::default();
        
        let status = RegQueryValueExW(
            hsub_key,
            PCWSTR(name_w.as_ptr()),
            None,
            Some(&mut value_type),
            None,
            Some(&mut buf_len),
        );
        
        if status.0 != 0 {
            let _ = RegCloseKey(hsub_key);
            if status.0 == 2 { // ERROR_FILE_NOT_FOUND
                return Ok(None);
            }
            return Err(anyhow!("Failed to query registry value size: status {}", status.0));
        }

        let mut buf: Vec<u16> = vec![0; (buf_len as usize + 1) / 2];
        let status = RegQueryValueExW(
            hsub_key,
            PCWSTR(name_w.as_ptr()),
            None,
            Some(&mut value_type),
            Some(buf.as_mut_ptr() as *mut u8),
            Some(&mut buf_len),
        );
        
        let _ = RegCloseKey(hsub_key);

        if status.0 != 0 {
            return Err(anyhow!("Failed to query registry value: status {}", status.0));
        }

        let len = buf.iter().position(|&x| x == 0).unwrap_or(buf.len());
        let string_val = String::from_utf16(&buf[..len])?;
        Ok(Some(string_val))
    }
}

fn reg_set_string(hkey: HKEY, sub_key: &str, name: &str, value: &str) -> Result<()> {
    unsafe {
        let sub_key_w = to_wide_str(OsStr::new(sub_key));
        let mut hsub_key = HKEY::default();
        let status = RegCreateKeyExW(
            hkey,
            PCWSTR(sub_key_w.as_ptr()),
            0,
            PCWSTR::null(),
            REG_OPTION_NON_VOLATILE,
            KEY_WRITE,
            None,
            &mut hsub_key,
            None,
        );
        if status.0 != 0 {
            return Err(anyhow!("Failed to create registry key {}: status {}", sub_key, status.0));
        }

        let name_w = to_wide_str(OsStr::new(name));
        let value_w = to_wide_str(OsStr::new(value));
        let status = RegSetValueExW(
            hsub_key,
            PCWSTR(name_w.as_ptr()),
            0,
            REG_SZ,
            Some(std::slice::from_raw_parts(
                value_w.as_ptr() as *const u8,
                value_w.len() * 2,
            )),
        );
        let _ = RegCloseKey(hsub_key);

        if status.0 != 0 {
            return Err(anyhow!("Failed to set registry value {}: status {}", name, status.0));
        }
    }
    Ok(())
}

fn reg_delete_key(hkey: HKEY, sub_key: &str) -> Result<()> {
    unsafe {
        let sub_key_w = to_wide_str(OsStr::new(sub_key));
        let status = RegDeleteKeyW(hkey, PCWSTR(sub_key_w.as_ptr()));
        if status.0 != 0 && status.0 != 2 { // 2 = ERROR_FILE_NOT_FOUND (fine)
            return Err(anyhow!("Failed to delete registry key {}: status {}", sub_key, status.0));
        }
    }
    Ok(())
}

fn broadcast_environment_change() {
    unsafe {
        let env_w = to_wide_str(OsStr::new("Environment"));
        let mut result = usize::default();
        let _ = SendMessageTimeoutW(
            HWND_BROADCAST,
            WM_SETTINGCHANGE,
            WPARAM(0),
            LPARAM(env_w.as_ptr() as isize),
            SMTO_ABORTIFHUNG,
            1000,
            Some(&mut result),
        );
    }
}

fn add_to_path(install_dir: &Path) -> Result<()> {
    let install_dir_str = install_dir.to_str().ok_or_else(|| anyhow!("Invalid install dir path"))?;
    
    let current_path = reg_get_string(HKEY_CURRENT_USER, "Environment", "Path")?
        .unwrap_or_default();
    
    let current_path_lower = current_path.to_lowercase();
    let install_dir_lower = install_dir_str.to_lowercase();
    
    if !current_path_lower.contains(&install_dir_lower) {
        let new_path = if current_path.is_empty() {
            install_dir_str.to_string()
        } else {
            let separator = if current_path.ends_with(';') { "" } else { ";" };
            format!("{}{}{}", current_path, separator, install_dir_str)
        };
        
        reg_set_string(HKEY_CURRENT_USER, "Environment", "Path", &new_path)?;
        broadcast_environment_change();
        println!("Added {} to PATH.", install_dir_str);
    } else {
        println!("PATH already contains {}.", install_dir_str);
    }
    
    Ok(())
}

fn remove_from_path(install_dir: &Path) -> Result<()> {
    let install_dir_str = install_dir.to_str().ok_or_else(|| anyhow!("Invalid install dir path"))?;
    
    let current_path = reg_get_string(HKEY_CURRENT_USER, "Environment", "Path")?;
    if let Some(path_val) = current_path {
        let paths: Vec<&str> = path_val.split(';').collect();
        let install_dir_lower = install_dir_str.to_lowercase();
        
        let new_paths: Vec<&str> = paths
            .into_iter()
            .filter(|p| p.to_lowercase().trim() != install_dir_lower.trim())
            .collect();
        
        let new_path_val = new_paths.join(";");
        if new_path_val != path_val {
            reg_set_string(HKEY_CURRENT_USER, "Environment", "Path", &new_path_val)?;
            broadcast_environment_change();
            println!("Removed {} from PATH.", install_dir_str);
        }
    }
    
    Ok(())
}

fn register_uninstaller(target_uninstall: &Path, target_cloak: &Path) -> Result<()> {
    let sub_key = "Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\CloaKey";
    let uninstall_string = format!("\"{}\" --uninstall", target_uninstall.to_str().unwrap());
    let display_icon = format!("\"{}\"", target_cloak.to_str().unwrap());
    
    reg_set_string(HKEY_CURRENT_USER, sub_key, "DisplayName", "CloaKey")?;
    reg_set_string(HKEY_CURRENT_USER, sub_key, "UninstallString", &uninstall_string)?;
    reg_set_string(HKEY_CURRENT_USER, sub_key, "DisplayIcon", &display_icon)?;
    reg_set_string(HKEY_CURRENT_USER, sub_key, "Publisher", "CloaKey")?;
    reg_set_string(HKEY_CURRENT_USER, sub_key, "DisplayVersion", "1.0.0")?;
    
    Ok(())
}

fn unregister_uninstaller() -> Result<()> {
    let sub_key = "Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\CloaKey";
    reg_delete_key(HKEY_CURRENT_USER, sub_key)?;
    Ok(())
}

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> std::io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

fn run_install() -> Result<()> {
    println!("Installing CloaKey...");

    let current_exe = env::current_exe()?;
    let current_dir = current_exe
        .parent()
        .ok_or_else(|| anyhow!("Failed to get current directory"))?;

    // Find the executables
    let mut cloak_exe = current_dir.join("cloak.exe");
    let mut cloakey_exe = current_dir.join("cloakey.exe");
    let mut cloak_tray_exe = current_dir.join("cloak-tray.exe");

    // If we're running inside target/debug or target/release, look for the binaries there
    if !cloak_exe.exists() {
        if let Some(workspace_root) = current_dir.parent().and_then(|p| p.parent()) {
            let alt_cloak = workspace_root.join("cloak.exe");
            if alt_cloak.exists() {
                cloak_exe = alt_cloak;
            }
        }
    }
    if !cloakey_exe.exists() {
        if let Some(workspace_root) = current_dir.parent().and_then(|p| p.parent()) {
            let alt_cloakey = workspace_root.join("cloakey.exe");
            if alt_cloakey.exists() {
                cloakey_exe = alt_cloakey;
            }
        }
    }
    if !cloak_tray_exe.exists() {
        if let Some(workspace_root) = current_dir.parent().and_then(|p| p.parent()) {
            let alt_tray = workspace_root.join("cloak-tray.exe");
            if alt_tray.exists() {
                cloak_tray_exe = alt_tray;
            }
        }
    }

    // Fallback: look in target/release or target/debug directly relative to current directory
    if !cloak_exe.exists() {
        let release_cloak = current_dir.join("target").join("release").join("cloak.exe");
        if release_cloak.exists() {
            cloak_exe = release_cloak;
        } else {
            let debug_cloak = current_dir.join("target").join("debug").join("cloak.exe");
            if debug_cloak.exists() {
                cloak_exe = debug_cloak;
            }
        }
    }
    if !cloakey_exe.exists() {
        let release_cloakey = current_dir
            .join("target")
            .join("release")
            .join("cloakey.exe");
        if release_cloakey.exists() {
            cloakey_exe = release_cloakey;
        } else {
            let debug_cloakey = current_dir.join("target").join("debug").join("cloakey.exe");
            if debug_cloakey.exists() {
                cloakey_exe = debug_cloakey;
            }
        }
    }
    if !cloak_tray_exe.exists() {
        let release_tray = current_dir
            .join("target")
            .join("release")
            .join("cloak-tray.exe");
        if release_tray.exists() {
            cloak_tray_exe = release_tray;
        } else {
            let debug_tray = current_dir
                .join("target")
                .join("debug")
                .join("cloak-tray.exe");
            if debug_tray.exists() {
                cloak_tray_exe = debug_tray;
            }
        }
    }

    if !cloak_exe.exists() {
        return Err(anyhow!(
            "cloak.exe not found. Please compile the release profile first: 'cargo build --release'"
        ));
    }

    let install_dir = get_install_dir()?;
    fs::create_dir_all(&install_dir)?;

    let target_cloak = install_dir.join("cloak.exe");
    let target_cloakey = install_dir.join("cloakey.exe");
    let target_tray = install_dir.join("cloak-tray.exe");
    let target_uninstall = install_dir.join("uninstall.exe");

    println!("Copying files to {}...", install_dir.display());
    fs::copy(&cloak_exe, &target_cloak)?;
    if cloakey_exe.exists() {
        fs::copy(&cloakey_exe, &target_cloakey)?;
    } else {
        fs::copy(&cloak_exe, &target_cloakey)?;
    }
    if cloak_tray_exe.exists() {
        fs::copy(&cloak_tray_exe, &target_tray)?;
    }

    // Copy assets folder if present
    let mut assets_src = current_dir.join("assets");
    if !assets_src.exists() {
        if let Some(workspace_root) = current_dir.parent().and_then(|p| p.parent()) {
            let alt_assets = workspace_root.join("assets");
            if alt_assets.exists() {
                assets_src = alt_assets;
            }
        }
    }
    if assets_src.exists() {
        println!("Copying assets folder...");
        let target_assets = install_dir.join("assets");
        let _ = copy_dir_all(&assets_src, &target_assets);
    }

    // Copy our own installer binary as uninstall.exe
    fs::copy(&current_exe, &target_uninstall)?;

    println!("Creating Start Menu shortcut...");
    let shortcut_path = get_start_menu_shortcut_path()?;
    if let Some(shortcut_dir) = shortcut_path.parent() {
        let _ = fs::create_dir_all(shortcut_dir);
    }
    if let Err(e) = create_shortcut(&shortcut_path, &target_cloak, &install_dir, &target_cloak) {
        println!("Warning: Failed to create Start Menu shortcut: {}", e);
    }

    println!("Adding CloaKey to PATH...");
    if let Err(e) = add_to_path(&install_dir) {
        println!("Warning: Failed to add CloaKey to PATH: {}", e);
    }

    println!("Registering uninstaller in Registry...");
    if let Err(e) = register_uninstaller(&target_uninstall, &target_cloak) {
        println!("Warning: Failed to register uninstaller: {}", e);
    }

    println!("\n✓ CloaKey installed successfully!");
    println!("You can search for 'CloaKey' in the Windows Start Menu,");
    println!("or open a new terminal/powershell window and run: cloak");
    Ok(())
}

fn run_uninstall() -> Result<()> {
    println!("Uninstalling CloaKey...");

    let install_dir = get_install_dir()?;
    let shortcut_path = get_start_menu_shortcut_path()?;

    println!("Removing Start Menu shortcut...");
    if shortcut_path.exists() {
        let _ = fs::remove_file(&shortcut_path);
    }

    println!("Removing CloaKey from PATH...");
    if let Err(e) = remove_from_path(&install_dir) {
        println!("Warning: Failed to remove CloaKey from PATH: {}", e);
    }

    println!("Removing Registry entries...");
    if let Err(e) = unregister_uninstaller() {
        println!("Warning: Failed to unregister uninstaller: {}", e);
    }

    println!("Deleting installed files...");
    let target_cloak = install_dir.join("cloak.exe");
    let target_cloakey = install_dir.join("cloakey.exe");
    let target_tray = install_dir.join("cloak-tray.exe");
    let target_assets = install_dir.join("assets");

    if target_cloak.exists() {
        let _ = fs::remove_file(target_cloak);
    }
    if target_cloakey.exists() {
        let _ = fs::remove_file(target_cloakey);
    }
    if target_tray.exists() {
        let _ = fs::remove_file(target_tray);
    }
    if target_assets.exists() {
        let _ = fs::remove_dir_all(target_assets);
    }

    // Trigger self-deletion of uninstall.exe and target folder
    let current_exe = env::current_exe()?;
    if current_exe.parent() == Some(&install_dir) {
        println!("Scheduling final directory cleanup...");
        let cleanup_cmd = format!(
            "timeout /t 1 /nobreak > NUL && del /f /q \"{}\" && rd \"{}\"",
            current_exe.to_str().unwrap(),
            install_dir.to_str().unwrap()
        );
        Command::new("cmd").arg("/c").arg(cleanup_cmd).spawn()?;
    }

    println!("\n✓ CloaKey has been successfully uninstalled.");
    Ok(())
}
