use crate::config::ThemeMode;

/// Detect current OS appearance mode (Light or Dark).
pub fn get_system_appearance() -> ThemeMode {
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        if let Ok(output) = Command::new("defaults")
            .args(["read", "-g", "AppleInterfaceStyle"])
            .output()
        {
            if output.status.success() {
                let s = String::from_utf8_lossy(&output.stdout);
                if s.trim().eq_ignore_ascii_case("Dark") {
                    return ThemeMode::Dark;
                }
            }
        }
        ThemeMode::Light
    }

    #[cfg(target_os = "windows")]
    {
        use windows_sys::Win32::System::Registry::{
            RegCloseKey, RegOpenKeyExW, RegQueryValueExW, HKEY_CURRENT_USER, KEY_READ,
        };
        let subkey: Vec<u16> =
            "Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize\0"
                .encode_utf16()
                .collect();
        let value_name: Vec<u16> = "AppsUseLightTheme\0".encode_utf16().collect();
        unsafe {
            let mut hkey = std::ptr::null_mut();
            if RegOpenKeyExW(HKEY_CURRENT_USER, subkey.as_ptr(), 0, KEY_READ, &mut hkey) == 0 {
                let mut data: u32 = 1;
                let mut data_len = std::mem::size_of::<u32>() as u32;
                let res = RegQueryValueExW(
                    hkey,
                    value_name.as_ptr(),
                    std::ptr::null(),
                    std::ptr::null_mut(),
                    &mut data as *mut u32 as *mut u8,
                    &mut data_len,
                );
                RegCloseKey(hkey);
                if res == 0 && data == 0 {
                    return ThemeMode::Dark;
                }
            }
        }
        ThemeMode::Light
    }

    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        if let Ok(output) = Command::new("gsettings")
            .args(["get", "org.gnome.desktop.interface", "color-scheme"])
            .output()
        {
            if output.status.success() {
                let s = String::from_utf8_lossy(&output.stdout);
                if s.contains("prefer-dark") {
                    return ThemeMode::Dark;
                }
            }
        }
        ThemeMode::Light
    }
}

/// Set the OS appearance mode (Light or Dark).
pub fn set_system_appearance(mode: ThemeMode) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        let dark_bool = match mode {
            ThemeMode::Dark => "true",
            ThemeMode::Light => "false",
        };
        let script = format!(
            "tell application \"System Events\" to tell appearance preferences to set dark mode to {dark_bool}"
        );
        let output = Command::new("osascript")
            .args(["-e", &script])
            .output()
            .map_err(|e| format!("Failed to invoke osascript: {e}"))?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
            return Err(if err.is_empty() {
                "macOS blocked appearance switch (check Automation permissions)".to_string()
            } else {
                err
            });
        }
        Ok(())
    }

    #[cfg(target_os = "windows")]
    {
        use windows_sys::Win32::System::Registry::{
            RegCloseKey, RegOpenKeyExW, RegSetValueExW, HKEY_CURRENT_USER, KEY_SET_VALUE,
            REG_DWORD,
        };
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            SendMessageTimeoutW, HWND_BROADCAST, SMTO_ABORTIFHUNG, WM_SETTINGCHANGE,
        };

        let subkey: Vec<u16> =
            "Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize\0"
                .encode_utf16()
                .collect();
        let val: u32 = match mode {
            ThemeMode::Light => 1,
            ThemeMode::Dark => 0,
        };
        let apps_name: Vec<u16> = "AppsUseLightTheme\0".encode_utf16().collect();
        let sys_name: Vec<u16> = "SystemUsesLightTheme\0".encode_utf16().collect();

        unsafe {
            let mut hkey = std::ptr::null_mut();
            let open_res = RegOpenKeyExW(
                HKEY_CURRENT_USER,
                subkey.as_ptr(),
                0,
                KEY_SET_VALUE,
                &mut hkey,
            );
            if open_res != 0 {
                return Err(format!("RegOpenKeyExW failed with code {open_res}"));
            }
            let bytes = val.to_le_bytes();
            let r1 = RegSetValueExW(
                hkey,
                apps_name.as_ptr(),
                0,
                REG_DWORD,
                bytes.as_ptr(),
                bytes.len() as u32,
            );
            let r2 = RegSetValueExW(
                hkey,
                sys_name.as_ptr(),
                0,
                REG_DWORD,
                bytes.as_ptr(),
                bytes.len() as u32,
            );
            RegCloseKey(hkey);
            if r1 != 0 || r2 != 0 {
                return Err(format!("RegSetValueExW failed ({r1}, {r2})"));
            }

            let param: Vec<u16> = "ImmersiveColorSet\0".encode_utf16().collect();
            let mut result: usize = 0;
            SendMessageTimeoutW(
                HWND_BROADCAST,
                WM_SETTINGCHANGE,
                0,
                param.as_ptr() as isize,
                SMTO_ABORTIFHUNG,
                2000,
                &mut result,
            );
        }
        Ok(())
    }

    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        let scheme = match mode {
            ThemeMode::Dark => "prefer-dark",
            ThemeMode::Light => "default",
        };
        let _ = Command::new("gsettings")
            .args(["set", "org.gnome.desktop.interface", "color-scheme", scheme])
            .status();
        let kde_theme = match mode {
            ThemeMode::Dark => "BreezeDark",
            ThemeMode::Light => "BreezeLight",
        };
        let _ = Command::new("plasma-apply-colorscheme")
            .arg(kde_theme)
            .status();
        Ok(())
    }
}
