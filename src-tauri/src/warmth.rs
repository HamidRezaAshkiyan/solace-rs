use crate::config::{WarmthConfig, WarmthSchedule};
use crate::solar::SolarTimes;

/// Neutral daylight color temperature in Kelvin.
pub const NEUTRAL_KELVIN: u32 = 6500;
/// Warmest candlelight color temperature in Kelvin.
pub const MIN_KELVIN: u32 = 1900;
/// Gradual ease-in transition duration in minutes.
pub const TRANSITION_MINUTES: f64 = 15.0;

/// Convert a color temperature in Kelvin (1900K..=6500K) to normalized (R, G, B) multipliers in [0.0, 1.0].
/// Uses Tanner Helland's blackbody approximation calibrated so 6500K = (1.0, 1.0, 1.0).
pub fn kelvin_to_rgb(kelvin: u32) -> (f32, f32, f32) {
    let k = kelvin.clamp(MIN_KELVIN, NEUTRAL_KELVIN) as f64;
    if k >= 6500.0 {
        return (1.0, 1.0, 1.0);
    }
    let temp = k / 100.0;

    let r = 1.0_f64;

    let g = if temp <= 66.0 {
        ((99.4708025861 * temp.ln() - 161.1195681661) / 255.0).clamp(0.0, 1.0)
    } else {
        1.0
    };

    let b = if temp >= 66.0 {
        1.0
    } else if temp <= 19.0 {
        0.0
    } else {
        ((138.5177312231 * (temp - 10.0).ln() - 305.0447927307) / 255.0).clamp(0.0, 1.0)
    };

    // Calibrate so 6500K smoothly reaches (1.0, 1.0, 1.0)
    let g_ref = (99.4708025861 * 65.0_f64.ln() - 161.1195681661) / 255.0;
    let b_ref = (138.5177312231 * 55.0_f64.ln() - 305.0447927307) / 255.0;

    (
        r as f32,
        (g / g_ref).clamp(0.0, 1.0) as f32,
        (b / b_ref).clamp(0.0, 1.0) as f32,
    )
}

/// Compute the effective color temperature right now, taking schedule and 15-minute gradual fade into account.
pub fn compute_effective_kelvin(
    cfg: &WarmthConfig,
    solar: Option<&SolarTimes>,
    current_minute: i32,
) -> (u32, bool) {
    if !cfg.enabled {
        return (NEUTRAL_KELVIN, false);
    }

    let target = cfg.kelvin.clamp(MIN_KELVIN, NEUTRAL_KELVIN);
    let sunrise_min = solar.map(|s| s.effective_sunrise_min).unwrap_or(420); // 7:00 AM fallback

    match cfg.schedule {
        WarmthSchedule::Always => (target, true),
        WarmthSchedule::Sunset => {
            let start_min = solar.map(|s| s.effective_sunset_min).unwrap_or(1080); // 6:00 PM fallback
            evaluate_window(current_minute, start_min, sunrise_min, target)
        }
        WarmthSchedule::FixedTime { hour, minute } => {
            let start_min = (hour as i32) * 60 + (minute as i32);
            evaluate_window(current_minute, start_min, sunrise_min, target)
        }
    }
}

/// Evaluate whether `current_minute` falls in the active warmth window `[start_min -> sunrise_min]`
/// (wrapping past midnight) with a smooth 15-minute ease-in ramp.
fn evaluate_window(current_min: i32, start_min: i32, sunrise_min: i32, target: u32) -> (u32, bool) {
    let active = if start_min > sunrise_min {
        // Normal overnight window (e.g. 20:00 -> 07:00)
        current_min >= start_min || current_min < sunrise_min
    } else {
        current_min >= start_min && current_min < sunrise_min
    };

    if !active {
        return (NEUTRAL_KELVIN, false);
    }

    // Minutes elapsed since start_min (mod 1440)
    let elapsed = (current_min - start_min).rem_euclid(1440) as f64;
    if elapsed < TRANSITION_MINUTES {
        let t = (elapsed / TRANSITION_MINUTES).clamp(0.0, 1.0);
        // Smoothstep easing
        let eased = t * t * (3.0 - 2.0 * t);
        let blended =
            NEUTRAL_KELVIN as f64 + (target as f64 - NEUTRAL_KELVIN as f64) * eased;
        (blended.round() as u32, true)
    } else {
        (target, true)
    }
}

/// Apply color temperature to all active displays using native OS gamma LUTs.
pub fn apply_kelvin(kelvin: u32, _compatibility_mode: bool) -> Result<(), String> {
    let (r_mul, g_mul, b_mul) = kelvin_to_rgb(kelvin);

    #[cfg(target_os = "macos")]
    {
        apply_macos_gamma(r_mul, g_mul, b_mul)
    }

    #[cfg(target_os = "windows")]
    {
        apply_windows_gamma(r_mul, g_mul, b_mul)
    }

    #[cfg(target_os = "linux")]
    {
        apply_linux_gamma(r_mul, g_mul, b_mul)
    }
}

/// Restore neutral 6500K display gamma cleanly.
pub fn reset_gamma() {
    #[cfg(target_os = "macos")]
    {
        #[link(name = "CoreGraphics", kind = "framework")]
        extern "C" {
            fn CGDisplayRestoreColorSyncSettings();
        }
        unsafe {
            CGDisplayRestoreColorSyncSettings();
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = apply_kelvin(NEUTRAL_KELVIN, false);
    }
}

#[cfg(target_os = "macos")]
fn apply_macos_gamma(r_mul: f32, g_mul: f32, b_mul: f32) -> Result<(), String> {
    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        fn CGGetActiveDisplayList(
            max_displays: u32,
            active_displays: *mut u32,
            display_count: *mut u32,
        ) -> i32;
        fn CGSetDisplayTransferByTable(
            display: u32,
            table_size: u32,
            red_table: *const f32,
            green_table: *const f32,
            blue_table: *const f32,
        ) -> i32;
    }

    const TABLE_SIZE: usize = 256;
    let mut red = [0.0_f32; TABLE_SIZE];
    let mut green = [0.0_f32; TABLE_SIZE];
    let mut blue = [0.0_f32; TABLE_SIZE];

    for i in 0..TABLE_SIZE {
        let base = i as f32 / (TABLE_SIZE - 1) as f32;
        red[i] = (base * r_mul).clamp(0.0, 1.0);
        green[i] = (base * g_mul).clamp(0.0, 1.0);
        blue[i] = (base * b_mul).clamp(0.0, 1.0);
    }

    let mut displays = [0u32; 16];
    let mut count: u32 = 0;
    let err = unsafe { CGGetActiveDisplayList(16, displays.as_mut_ptr(), &mut count) };
    if err != 0 {
        return Err(format!("CGGetActiveDisplayList failed: {err}"));
    }

    for &display in &displays[..count as usize] {
        let res = unsafe {
            CGSetDisplayTransferByTable(
                display,
                TABLE_SIZE as u32,
                red.as_ptr(),
                green.as_ptr(),
                blue.as_ptr(),
            )
        };
        if res != 0 {
            return Err(format!("CGSetDisplayTransferByTable failed for display {display}: {res}"));
        }
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn apply_windows_gamma(r_mul: f32, g_mul: f32, b_mul: f32) -> Result<(), String> {
    use windows_sys::Win32::Graphics::Gdi::{GetDC, ReleaseDC};
    use windows_sys::Win32::UI::ColorSystem::SetDeviceGammaRamp;

    #[repr(C)]
    struct GammaRamp {
        red: [u16; 256],
        green: [u16; 256],
        blue: [u16; 256],
    }

    let mut ramp = GammaRamp {
        red: [0; 256],
        green: [0; 256],
        blue: [0; 256],
    };

    for i in 0..256 {
        let base = (i as f32 / 255.0) * 65535.0;
        ramp.red[i] = (base * r_mul).clamp(0.0, 65535.0) as u16;
        ramp.green[i] = (base * g_mul).clamp(0.0, 65535.0) as u16;
        ramp.blue[i] = (base * b_mul).clamp(0.0, 65535.0) as u16;
    }

    unsafe {
        let hdc = GetDC(std::ptr::null_mut());
        if hdc.is_null() {
            return Err("GetDC(NULL) returned null".to_string());
        }
        let ok = SetDeviceGammaRamp(hdc, &ramp as *const GammaRamp as *const _);
        ReleaseDC(std::ptr::null_mut(), hdc);
        if ok == 0 {
            return Err("SetDeviceGammaRamp failed".to_string());
        }
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn apply_linux_gamma(r_mul: f32, g_mul: f32, b_mul: f32) -> Result<(), String> {
    use std::process::Command;
    // Apply via xrandr on all connected outputs (X11)
    let out = Command::new("xrandr").arg("--query").output();
    if let Ok(output) = out {
        let text = String::from_utf8_lossy(&output.stdout);
        let gamma_arg = format!("{:.3}:{:.3}:{:.3}", r_mul.max(0.1), g_mul.max(0.1), b_mul.max(0.1));
        for line in text.lines() {
            if line.contains(" connected") {
                if let Some(display_name) = line.split_whitespace().next() {
                    let _ = Command::new("xrandr")
                        .args(["--output", display_name, "--gamma", &gamma_arg])
                        .status();
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn neutral_kelvin_is_identity() {
        let (r, g, b) = kelvin_to_rgb(6500);
        assert!((r - 1.0).abs() < 1e-4);
        assert!((g - 1.0).abs() < 1e-4);
        assert!((b - 1.0).abs() < 1e-4);
    }

    #[test]
    fn candlelight_reduces_blue_strongly() {
        let (r, g, b) = kelvin_to_rgb(1900);
        assert!((r - 1.0).abs() < 1e-4);
        assert!(g < 0.65);
        assert!(b < 0.05);
    }

    #[test]
    fn gradual_fade_ramps_smoothly() {
        let cfg = WarmthConfig {
            enabled: true,
            kelvin: 2500,
            schedule: WarmthSchedule::FixedTime { hour: 20, minute: 0 },
            compatibility_mode: false,
        };
        // Before 20:00 -> neutral
        assert_eq!(compute_effective_kelvin(&cfg, None, 19 * 60 + 50), (6500, false));
        // 7 minutes into 15-min ramp -> intermediate
        let (mid_k, active) = compute_effective_kelvin(&cfg, None, 20 * 60 + 7);
        assert!(active);
        assert!(mid_k > 2500 && mid_k < 6500);
        // After 15 minutes -> full target
        assert_eq!(compute_effective_kelvin(&cfg, None, 20 * 60 + 20), (2500, true));
    }
}
