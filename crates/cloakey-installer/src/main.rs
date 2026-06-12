use anyhow::{anyhow, Result};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

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

fn get_install_dir() -> Result<PathBuf> {
    let local_app_data = dirs::data_local_dir()
        .ok_or_else(|| anyhow!("Failed to locate Local AppData directory"))?;
    Ok(local_app_data.join("Programs").join("CloaKey"))
}

fn get_start_menu_shortcut_path() -> Result<PathBuf> {
    let app_data = dirs::data_dir().ok_or_else(|| anyhow!("Failed to locate AppData directory"))?;
    Ok(app_data
        .join("Microsoft")
        .join("Windows")
        .join("Start Menu")
        .join("Programs")
        .join("CloaKey.lnk"))
}

fn run_powershell_cmd(cmd: &str) -> Result<()> {
    let output = Command::new("powershell")
        .arg("-NoProfile")
        .arg("-Command")
        .arg(cmd)
        .output()?;
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("PowerShell command failed: {}", err));
    }
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
    let shortcut_script = format!(
        "$WshShell = New-Object -ComObject WScript.Shell; \
         $Shortcut = $WshShell.CreateShortcut('{}'); \
         $Shortcut.TargetPath = '{}'; \
         $Shortcut.WorkingDirectory = '{}'; \
         $Shortcut.IconLocation = '{},0'; \
         $Shortcut.Save();",
        shortcut_path.to_str().unwrap().replace("'", "''"),
        target_cloak.to_str().unwrap().replace("'", "''"),
        install_dir.to_str().unwrap().replace("'", "''"),
        target_cloak.to_str().unwrap().replace("'", "''")
    );
    run_powershell_cmd(&shortcut_script)?;

    println!("Adding CloaKey to PATH...");
    let path_script = format!(
        "$p = [System.Environment]::GetEnvironmentVariable('PATH', 'User'); \
         if ($p -notlike '*{}*') {{ \
             [System.Environment]::SetEnvironmentVariable('PATH', $p + ';{}', 'User'); \
         }}",
        install_dir.to_str().unwrap().replace("'", "''"),
        install_dir.to_str().unwrap().replace("'", "''")
    );
    let _ = run_powershell_cmd(&path_script);

    println!("Registering uninstaller in Registry...");
    let registry_script = format!(
        "New-Item -Path 'HKCU:\\Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\CloaKey' -Force | Out-Null; \
         Set-ItemProperty -Path 'HKCU:\\Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\CloaKey' -Name 'DisplayName' -Value 'CloaKey' -Type String; \
         Set-ItemProperty -Path 'HKCU:\\Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\CloaKey' -Name 'UninstallString' -Value '\"{}\" --uninstall' -Type String; \
         Set-ItemProperty -Path 'HKCU:\\Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\CloaKey' -Name 'DisplayIcon' -Value '\"{}\"' -Type String; \
         Set-ItemProperty -Path 'HKCU:\\Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\CloaKey' -Name 'Publisher' -Value 'CloaKey' -Type String; \
         Set-ItemProperty -Path 'HKCU:\\Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\CloaKey' -Name 'DisplayVersion' -Value '1.0.0' -Type String;",
        target_uninstall.to_str().unwrap().replace("'", "''"),
        target_cloak.to_str().unwrap().replace("'", "''")
    );
    let _ = run_powershell_cmd(&registry_script);

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
    let path_script = format!(
        "$p = [System.Environment]::GetEnvironmentVariable('PATH', 'User'); \
         $p = $p -replace [regex]::Escape(';{}'), ''; \
         $p = $p -replace [regex]::Escape('{};'), ''; \
         $p = $p -replace [regex]::Escape('{}'), ''; \
         [System.Environment]::SetEnvironmentVariable('PATH', $p, 'User');",
        install_dir.to_str().unwrap().replace("'", "''"),
        install_dir.to_str().unwrap().replace("'", "''"),
        install_dir.to_str().unwrap().replace("'", "''")
    );
    let _ = run_powershell_cmd(&path_script);

    println!("Removing Registry entries...");
    let registry_script = "Remove-Item -Path 'HKCU:\\Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\CloaKey' -Recurse -ErrorAction SilentlyContinue | Out-Null";
    let _ = run_powershell_cmd(registry_script)?;

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
