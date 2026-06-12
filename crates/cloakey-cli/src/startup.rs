//! Windows startup integration via the registry.
//!
//! Adds or removes CloaKey from the Windows startup programs list by writing
//! to: `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run`
//!
//! This is the standard, user-level (no admin required) startup mechanism.

#[cfg(target_os = "windows")]
mod windows_startup {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    use tracing::info;
    use windows::Win32::System::Registry::{
        RegCloseKey, RegDeleteValueW, RegOpenKeyExW, RegSetValueExW, HKEY_CURRENT_USER,
        KEY_SET_VALUE, REG_SZ,
    };

    const STARTUP_KEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run\0";
    const VALUE_NAME: &str = "CloaKey\0";

    fn to_wide(s: &str) -> Vec<u16> {
        OsStr::new(s)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect()
    }

    /// Register CloaKey to run at Windows startup.
    ///
    /// Gets the current executable path and adds it to the Run registry key.
    pub fn enable_startup() -> Result<(), String> {
        let exe_path =
            std::env::current_exe().map_err(|e| format!("Cannot get executable path: {}", e))?;

        let tray_path = exe_path.parent()
            .map(|p| p.join("cloak-tray.exe"))
            .ok_or_else(|| "Failed to get parent directory".to_string())?;

        let exe_to_register = if tray_path.exists() {
            tray_path
        } else {
            exe_path
        };

        let exe_str = exe_to_register
            .to_str()
            .ok_or_else(|| "Executable path contains invalid characters".to_string())?;

        unsafe {
            let key_name = to_wide(STARTUP_KEY);
            let mut hkey = std::mem::zeroed();

            let result = RegOpenKeyExW(
                HKEY_CURRENT_USER,
                windows::core::PCWSTR(key_name.as_ptr()),
                0,
                KEY_SET_VALUE,
                &mut hkey,
            );

            if result.is_err() {
                return Err(format!("Failed to open startup registry key: {:?}", result));
            }

            let value_name = to_wide(VALUE_NAME);
            let value_data = to_wide(exe_str);
            let value_bytes =
                std::slice::from_raw_parts(value_data.as_ptr() as *const u8, value_data.len() * 2);

            let set_result = RegSetValueExW(
                hkey,
                windows::core::PCWSTR(value_name.as_ptr()),
                0,
                REG_SZ,
                Some(value_bytes),
            );

            let _ = RegCloseKey(hkey);

            if set_result.is_err() {
                return Err(format!(
                    "Failed to write startup registry value: {:?}",
                    set_result
                ));
            }

            info!("CloaKey added to Windows startup: {}", exe_str);
            Ok(())
        }
    }

    /// Remove CloaKey from Windows startup.
    pub fn disable_startup() -> Result<(), String> {
        unsafe {
            let key_name = to_wide(STARTUP_KEY);
            let mut hkey = std::mem::zeroed();

            let result = RegOpenKeyExW(
                HKEY_CURRENT_USER,
                windows::core::PCWSTR(key_name.as_ptr()),
                0,
                KEY_SET_VALUE,
                &mut hkey,
            );

            if result.is_err() {
                // Key doesn't exist — nothing to remove
                return Ok(());
            }

            let value_name = to_wide(VALUE_NAME);
            let _ = RegDeleteValueW(hkey, windows::core::PCWSTR(value_name.as_ptr()));
            let _ = RegCloseKey(hkey);

            info!("CloaKey removed from Windows startup");
            Ok(())
        }
    }
}

#[cfg(not(target_os = "windows"))]
mod windows_startup {
    pub fn enable_startup() -> Result<(), String> {
        Ok(())
    }
    pub fn disable_startup() -> Result<(), String> {
        Ok(())
    }
}

/// Enable or disable CloaKey Windows startup integration.
///
/// `enabled = true` → adds to startup; `false` → removes from startup.
pub fn set_startup_enabled(enabled: bool) -> Result<(), String> {
    if enabled {
        windows_startup::enable_startup()
    } else {
        windows_startup::disable_startup()
    }
}
