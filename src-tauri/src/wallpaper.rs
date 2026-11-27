use std::path::Path;

/// Check whether a wallpaper path exists on disk.
pub fn wallpaper_exists(path: &Option<String>) -> bool {
    match path {
        Some(p) if !p.is_empty() => Path::new(p).exists(),
        _ => true,
    }
}

/// Read the current primary desktop wallpaper path (best-effort).
pub fn get_current_wallpaper() -> Option<String> {
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        let output = Command::new("osascript")
            .args([
                "-e",
                "tell application \"System Events\" to get picture of first desktop",
            ])
            .output()
            .ok()?;
        if output.status.success() {
            let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !s.is_empty() {
                return Some(s);
            }
        }
        None
    }

    #[cfg(target_os = "windows")]
    {
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            SystemParametersInfoW, SPI_GETDESKWALLPAPER,
        };
        let mut buf = [0u16; 520];
        let ok = unsafe {
            SystemParametersInfoW(
                SPI_GETDESKWALLPAPER,
                buf.len() as u32,
                buf.as_mut_ptr() as *mut _,
                0,
            )
        };
        if ok != 0 {
            let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
            let s = String::from_utf16_lossy(&buf[..len]);
            if !s.is_empty() {
                return Some(s);
            }
        }
        None
    }

    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        let output = Command::new("gsettings")
            .args(["get", "org.gnome.desktop.background", "picture-uri"])
            .output()
            .ok()?;
        if output.status.success() {
            let raw = String::from_utf8_lossy(&output.stdout)
                .trim()
                .trim_matches('\'')
                .to_string();
            return Some(raw.strip_prefix("file://").unwrap_or(&raw).to_string());
        }
        None
    }
}

/// Apply a desktop wallpaper across all displays.
pub fn set_wallpaper(path: &str) -> Result<(), String> {
    if !Path::new(path).exists() {
        return Err(format!("Wallpaper file not found: {path}"));
    }

    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        let escaped = path.replace('\\', "\\\\").replace('"', "\\\"");
        let script = format!(
            "tell application \"System Events\" to tell every desktop to set picture to POSIX file \"{escaped}\""
        );
        let output = Command::new("osascript")
            .args(["-e", &script])
            .output()
            .map_err(|e| format!("Failed to set wallpaper: {e}"))?;
        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
        }
        Ok(())
    }

    #[cfg(target_os = "windows")]
    {
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            SystemParametersInfoW, SPIF_SENDCHANGE, SPIF_UPDATEINIFILE, SPI_SETDESKWALLPAPER,
        };
        let wide: Vec<u16> = path.encode_utf16().chain(std::iter::once(0)).collect();
        let ok = unsafe {
            SystemParametersInfoW(
                SPI_SETDESKWALLPAPER,
                0,
                wide.as_ptr() as *mut _,
                SPIF_UPDATEINIFILE | SPIF_SENDCHANGE,
            )
        };
        if ok == 0 {
            return Err("SystemParametersInfoW(SPI_SETDESKWALLPAPER) failed".to_string());
        }
        Ok(())
    }

    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        let uri = format!("file://{path}");
        let _ = Command::new("gsettings")
            .args(["set", "org.gnome.desktop.background", "picture-uri", &uri])
            .status();
        let _ = Command::new("gsettings")
            .args([
                "set",
                "org.gnome.desktop.background",
                "picture-uri-dark",
                &uri,
            ])
            .status();
        let _ = Command::new("plasma-apply-wallpaperimage")
            .arg(path)
            .status();
        Ok(())
    }
}
