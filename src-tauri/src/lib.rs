pub mod appearance;
pub mod config;
pub mod scheduler;
pub mod solar;
pub mod wallpaper;
pub mod warmth;
pub mod weather;

use crate::config::{
    LocationConfig, TemperatureUnit, ThemeMode, WarmthSchedule, WeatherSensitivity,
};
use crate::scheduler::{SharedRuntime, SolaceRuntime, SolaceStateSnapshot};
use crate::weather::CitySearchResult;
use chrono::Local;
use parking_lot::Mutex;
use std::sync::Arc;
use tauri::image::Image;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, State};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};
use tauri_plugin_notification::NotificationExt;

const TRAY_ID: &str = "solace-tray";
const SUN_ICON_BYTES: &[u8] = include_bytes!("../icons/tray-sun.png");
const MOON_ICON_BYTES: &[u8] = include_bytes!("../icons/tray-moon.png");

pub struct AppState {
    pub runtime: SharedRuntime,
}

fn update_tray_icon(app: &AppHandle, mode: ThemeMode) {
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let bytes = match mode {
            ThemeMode::Light => SUN_ICON_BYTES,
            ThemeMode::Dark => MOON_ICON_BYTES,
        };
        if let Ok(img) = Image::from_bytes(bytes) {
            let _ = tray.set_icon(Some(img));
            #[cfg(target_os = "macos")]
            let _ = tray.set_icon_as_template(true);
        }
        let tooltip = format!("Solace — {}", mode.label());
        let _ = tray.set_tooltip(Some(&tooltip));
    }
}

fn emit_state(app: &AppHandle, runtime: &SharedRuntime) {
    let snap = {
        let guard = runtime.lock();
        guard.snapshot()
    };
    update_tray_icon(app, snap.current_mode);
    let _ = app.emit("solace://state-changed", &snap);
}

fn toggle_panel_window(app: &AppHandle, tray_pos: Option<PhysicalPosition<f64>>) {
    if let Some(window) = app.get_webview_window("main") {
        let visible = window.is_visible().unwrap_or(false);
        if visible {
            let _ = window.hide();
        } else {
            if let Some(pos) = tray_pos {
                if let Ok(win_size) = window.outer_size() {
                    let mut x = (pos.x - (win_size.width as f64) / 2.0).round() as i32;
                    let mut y = (pos.y + 12.0).round() as i32;
                    if let Ok(Some(monitor)) = window.current_monitor() {
                        let m_pos = monitor.position();
                        let m_size = monitor.size();
                        let max_x = m_pos.x + (m_size.width as i32) - (win_size.width as i32) - 8;
                        let max_y = m_pos.y + (m_size.height as i32) - (win_size.height as i32) - 8;
                        x = x.clamp(m_pos.x + 8, max_x.max(m_pos.x + 8));
                        // On Windows/Linux taskbar at bottom, open panel above tray icon
                        if y > max_y {
                            y = (pos.y as i32) - (win_size.height as i32) - 12;
                        }
                        y = y.clamp(m_pos.y + 8, max_y.max(m_pos.y + 8));
                    }
                    let _ = window.set_position(PhysicalPosition::new(x, y));
                }
            }
            let _ = window.show();
            let _ = window.set_focus();
        }
    }
}

fn show_panel_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn register_global_shortcut(app: &AppHandle, shortcut_str: &str) -> Result<(), String> {
    let gs = app.global_shortcut();
    let _ = gs.unregister_all();
    if shortcut_str.trim().is_empty() {
        return Ok(());
    }
    let handle = app.clone();
    gs.on_shortcut(shortcut_str, move |_app, _shortcut, event| {
        if event.state() == ShortcutState::Pressed {
            let state: State<'_, AppState> = handle.state();
            {
                let mut guard = state.runtime.lock();
                guard.manual_toggle("Keyboard shortcut");
            }
            emit_state(&handle, &state.runtime);
        }
    })
    .map_err(|e| format!("Could not register shortcut '{shortcut_str}': {e}"))
}

// ---------------- Tauri IPC Commands ----------------

#[tauri::command]
fn get_state(state: State<'_, AppState>) -> SolaceStateSnapshot {
    let guard = state.runtime.lock();
    guard.snapshot()
}

#[tauri::command]
fn toggle_mode(app: AppHandle, state: State<'_, AppState>) -> SolaceStateSnapshot {
    {
        let mut guard = state.runtime.lock();
        guard.manual_toggle("Manual switch");
    }
    emit_state(&app, &state.runtime);
    state.runtime.lock().snapshot()
}

#[tauri::command]
fn respond_first_switch(
    app: AppHandle,
    state: State<'_, AppState>,
    accept: bool,
) -> SolaceStateSnapshot {
    {
        let mut guard = state.runtime.lock();
        guard.config.first_switch_confirmed = true;
        if let Some(prompt) = guard.first_switch_prompt.take() {
            if accept {
                guard.apply_mode(prompt.target_mode, false);
            } else {
                let current = guard.current_mode;
                guard.set_manual_mode(current, "Kept current mode");
            }
        }
        let _ = guard.config.save();
    }
    emit_state(&app, &state.runtime);
    state.runtime.lock().snapshot()
}

#[tauri::command]
fn keep_light_today(app: AppHandle, state: State<'_, AppState>) -> SolaceStateSnapshot {
    {
        let mut guard = state.runtime.lock();
        guard.config.weather.keep_light_date = Some(Local::now().format("%Y-%m-%d").to_string());
        guard.set_manual_mode(ThemeMode::Light, "Kept Light today");
    }
    emit_state(&app, &state.runtime);
    state.runtime.lock().snapshot()
}

#[tauri::command]
fn set_wallpapers_enabled(
    app: AppHandle,
    state: State<'_, AppState>,
    enabled: bool,
) -> SolaceStateSnapshot {
    {
        let mut guard = state.runtime.lock();
        if enabled && !guard.config.wallpapers.enabled {
            // Save original wallpaper before Solace takes over
            if guard.config.wallpapers.original_path.is_none() {
                guard.config.wallpapers.original_path = wallpaper::get_current_wallpaper();
            }
            guard.config.wallpapers.enabled = true;
            let mode = guard.current_mode;
            guard.sync_wallpaper_for_mode(mode, true);
        } else if !enabled && guard.config.wallpapers.enabled {
            guard.config.wallpapers.enabled = false;
            guard.last_applied_wallpaper = None;
            if let Some(orig) = guard.config.wallpapers.original_path.clone() {
                let _ = wallpaper::set_wallpaper(&orig);
            }
        }
        let _ = guard.config.save();
    }
    emit_state(&app, &state.runtime);
    state.runtime.lock().snapshot()
}

#[tauri::command]
fn set_wallpaper_path(
    app: AppHandle,
    state: State<'_, AppState>,
    slot: String,
    path: String,
) -> SolaceStateSnapshot {
    {
        let mut guard = state.runtime.lock();
        if slot == "light" {
            guard.config.wallpapers.light_path = Some(path);
        } else {
            guard.config.wallpapers.dark_path = Some(path);
        }
        if !guard.config.wallpapers.enabled {
            if guard.config.wallpapers.original_path.is_none() {
                guard.config.wallpapers.original_path = wallpaper::get_current_wallpaper();
            }
            guard.config.wallpapers.enabled = true;
        }
        let mode = guard.current_mode;
        guard.sync_wallpaper_for_mode(mode, true);
        let _ = guard.config.save();
    }
    emit_state(&app, &state.runtime);
    state.runtime.lock().snapshot()
}

#[tauri::command]
fn set_schedule_enabled(
    app: AppHandle,
    state: State<'_, AppState>,
    enabled: bool,
) -> SolaceStateSnapshot {
    {
        let mut guard = state.runtime.lock();
        guard.config.schedule.enabled = enabled;
        if enabled {
            // Changing schedule starts fresh
            guard.config.manual_override = None;
        }
        let _ = guard.config.save();
        guard.tick();
    }
    emit_state(&app, &state.runtime);
    state.runtime.lock().snapshot()
}

#[tauri::command]
fn set_schedule_offsets(
    app: AppHandle,
    state: State<'_, AppState>,
    sunrise_offset_min: i32,
    sunset_offset_min: i32,
) -> SolaceStateSnapshot {
    {
        let mut guard = state.runtime.lock();
        guard.config.schedule.sunrise_offset_min = sunrise_offset_min.clamp(-120, 120);
        guard.config.schedule.sunset_offset_min = sunset_offset_min.clamp(-120, 120);
        guard.config.manual_override = None;
        let _ = guard.config.save();
        guard.tick();
    }
    emit_state(&app, &state.runtime);
    state.runtime.lock().snapshot()
}

#[tauri::command]
async fn set_location(
    app: AppHandle,
    state: State<'_, AppState>,
    location: LocationConfig,
) -> Result<SolaceStateSnapshot, String> {
    let weather_on = {
        let mut guard = state.runtime.lock();
        guard.config.location = Some(location);
        guard.config.manual_override = None;
        let _ = guard.config.save();
        guard.tick();
        guard.config.weather.enabled
    };
    emit_state(&app, &state.runtime);
    if weather_on {
        let rt = state.runtime.clone();
        scheduler::refresh_weather_async(rt, "Location updated".to_string()).await;
        emit_state(&app, &state.runtime);
    }
    Ok(state.runtime.lock().snapshot())
}

#[tauri::command]
async fn search_cities(query: String) -> Result<Vec<CitySearchResult>, String> {
    weather::search_cities(&query).await
}

#[tauri::command]
async fn auto_detect_location(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<SolaceStateSnapshot, String> {
    let loc = weather::auto_detect_location().await?;
    set_location(app, state, loc).await
}

#[tauri::command]
fn set_warmth_config(
    app: AppHandle,
    state: State<'_, AppState>,
    enabled: bool,
    kelvin: u32,
    schedule: WarmthSchedule,
    compatibility_mode: bool,
) -> SolaceStateSnapshot {
    {
        let mut guard = state.runtime.lock();
        guard.config.warmth.enabled = enabled;
        guard.config.warmth.kelvin = kelvin.clamp(warmth::MIN_KELVIN, warmth::NEUTRAL_KELVIN);
        guard.config.warmth.schedule = schedule;
        guard.config.warmth.compatibility_mode = compatibility_mode;
        guard.preview_kelvin = None;
        let _ = guard.config.save();

        if !enabled {
            guard.effective_kelvin = warmth::NEUTRAL_KELVIN;
            guard.warmth_active = false;
            warmth::reset_gamma();
        } else {
            guard.tick();
            if !guard.warmth_active {
                warmth::reset_gamma();
            }
        }
    }
    emit_state(&app, &state.runtime);
    state.runtime.lock().snapshot()
}

#[tauri::command]
fn preview_warmth(
    app: AppHandle,
    state: State<'_, AppState>,
    kelvin: Option<u32>,
) -> SolaceStateSnapshot {
    {
        let mut guard = state.runtime.lock();
        guard.preview_kelvin = kelvin;
        let compat = guard.config.warmth.compatibility_mode;
        match kelvin {
            Some(k) => {
                let _ = warmth::apply_kelvin(k, compat);
            }
            None => {
                if guard.warmth_active && guard.config.warmth.enabled {
                    let eff = guard.effective_kelvin;
                    let _ = warmth::apply_kelvin(eff, compat);
                } else {
                    warmth::reset_gamma();
                }
            }
        }
    }
    emit_state(&app, &state.runtime);
    state.runtime.lock().snapshot()
}

#[tauri::command]
async fn set_weather_enabled(
    app: AppHandle,
    state: State<'_, AppState>,
    enabled: bool,
) -> Result<SolaceStateSnapshot, String> {
    {
        let mut guard = state.runtime.lock();
        guard.config.weather.enabled = enabled;
        if enabled {
            guard.config.manual_override = None;
            guard.config.weather.keep_light_date = None;
        }
        let _ = guard.config.save();
    }
    if enabled {
        let rt = state.runtime.clone();
        if let Some(cloud_pct) =
            scheduler::refresh_weather_async(rt, "Weather enabled".to_string()).await
        {
            let _ = app
                .notification()
                .builder()
                .title("Solace — Overcast Day")
                .body(format!(
                    "Cloud cover is {:.0}%. Switched to Dark Mode for today.",
                    cloud_pct
                ))
                .show();
        }
    } else {
        let mut guard = state.runtime.lock();
        guard.tick();
    }
    emit_state(&app, &state.runtime);
    Ok(state.runtime.lock().snapshot())
}

#[tauri::command]
fn set_weather_sensitivity(
    app: AppHandle,
    state: State<'_, AppState>,
    sensitivity: WeatherSensitivity,
) -> SolaceStateSnapshot {
    {
        let mut guard = state.runtime.lock();
        guard.config.weather.sensitivity = sensitivity;
        guard.config.weather.keep_light_date = None;
        guard.config.manual_override = None;
        guard.evaluate_weather_decision("Sensitivity changed");
        guard.tick();
        let _ = guard.config.save();
    }
    emit_state(&app, &state.runtime);
    state.runtime.lock().snapshot()
}

#[tauri::command]
fn toggle_temperature_unit(app: AppHandle, state: State<'_, AppState>) -> SolaceStateSnapshot {
    {
        let mut guard = state.runtime.lock();
        guard.config.weather.unit = match guard.config.weather.unit {
            TemperatureUnit::Celsius => TemperatureUnit::Fahrenheit,
            TemperatureUnit::Fahrenheit => TemperatureUnit::Celsius,
        };
        let _ = guard.config.save();
    }
    emit_state(&app, &state.runtime);
    state.runtime.lock().snapshot()
}

#[tauri::command]
async fn refresh_weather(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<SolaceStateSnapshot, String> {
    let rt = state.runtime.clone();
    scheduler::refresh_weather_async(rt, "Manual refresh".to_string()).await;
    emit_state(&app, &state.runtime);
    Ok(state.runtime.lock().snapshot())
}

#[tauri::command]
fn update_shortcut(
    app: AppHandle,
    state: State<'_, AppState>,
    shortcut: String,
) -> Result<SolaceStateSnapshot, String> {
    register_global_shortcut(&app, &shortcut)?;
    {
        let mut guard = state.runtime.lock();
        guard.config.shortcut = shortcut;
        let _ = guard.config.save();
    }
    emit_state(&app, &state.runtime);
    Ok(state.runtime.lock().snapshot())
}

#[tauri::command]
fn complete_onboarding(app: AppHandle, state: State<'_, AppState>) -> SolaceStateSnapshot {
    {
        let mut guard = state.runtime.lock();
        guard.config.onboarding_completed = true;
        let _ = guard.config.save();
        guard.tick();
    }
    emit_state(&app, &state.runtime);
    state.runtime.lock().snapshot()
}

#[tauri::command]
fn hide_panel(app: AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
}

#[tauri::command]
fn quit_app(app: AppHandle) {
    warmth::reset_gamma();
    app.exit(0);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "solace_rs_lib=info".into()),
        )
        .init();

    let runtime: SharedRuntime = Arc::new(Mutex::new(SolaceRuntime::new()));

    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--hidden"]),
        ))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_opener::init())
        .manage(AppState {
            runtime: runtime.clone(),
        })
        .setup(move |app| {
            // Keep Solace in the menu bar and out of the macOS Dock
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let initial_mode = runtime.lock().current_mode;
            let initial_shortcut = runtime.lock().config.shortcut.clone();
            let onboarding_done = runtime.lock().config.onboarding_completed;

            // Hide window on startup if launched with --hidden (autostart) or already onboarded
            let launched_hidden = std::env::args().any(|a| a == "--hidden");
            if launched_hidden && onboarding_done {
                if let Some(win) = app.get_webview_window("main") {
                    let _ = win.hide();
                }
            }

            // Build tray context menu
            let toggle_item = MenuItem::with_id(app, "toggle", "Toggle Light / Dark Mode", true, None::<&str>)?;
            let open_item = MenuItem::with_id(app, "open", "Open Solace Panel", true, None::<&str>)?;
            let settings_item = MenuItem::with_id(app, "settings", "Settings…", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit Solace", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&toggle_item, &open_item, &settings_item, &quit_item])?;

            let icon_bytes = match initial_mode {
                ThemeMode::Light => SUN_ICON_BYTES,
                ThemeMode::Dark => MOON_ICON_BYTES,
            };
            let tray_img = Image::from_bytes(icon_bytes)?;

            let mut tray_builder = TrayIconBuilder::with_id(TRAY_ID)
                .icon(tray_img)
                .tooltip(format!("Solace — {}", initial_mode.label()))
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "toggle" => {
                        let state: State<'_, AppState> = app.state();
                        {
                            let mut guard = state.runtime.lock();
                            guard.manual_toggle("Menu bar");
                        }
                        emit_state(app, &state.runtime);
                    }
                    "open" => {
                        show_panel_window(app);
                        let _ = app.emit("solace://navigate", "main");
                    }
                    "settings" => {
                        show_panel_window(app);
                        let _ = app.emit("solace://navigate", "settings");
                    }
                    "quit" => {
                        warmth::reset_gamma();
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        position,
                        ..
                    } = event
                    {
                        toggle_panel_window(tray.app_handle(), Some(position));
                    }
                });

            #[cfg(target_os = "macos")]
            {
                tray_builder = tray_builder.icon_as_template(true);
            }

            tray_builder.build(app)?;

            // Register global shortcut (non-fatal if another app claimed it)
            if let Err(e) = register_global_shortcut(app.handle(), &initial_shortcut) {
                tracing::warn!("{e}");
            }

            // Spawn background scheduler loop (15s tick + hourly weather refresh)
            let app_handle = app.handle().clone();
            let bg_runtime = runtime.clone();
            tauri::async_runtime::spawn(async move {
                // Initial tick on startup
                {
                    let mut guard = bg_runtime.lock();
                    guard.tick();
                }
                emit_state(&app_handle, &bg_runtime);

                // Initial weather fetch if enabled
                let should_fetch_init = {
                    let g = bg_runtime.lock();
                    g.config.weather.enabled && g.config.location.is_some()
                };
                if should_fetch_init {
                    if let Some(cloud_pct) = scheduler::refresh_weather_async(
                        bg_runtime.clone(),
                        "Startup forecast".to_string(),
                    )
                    .await
                    {
                        let _ = app_handle
                            .notification()
                            .builder()
                            .title("Solace — Overcast Day")
                            .body(format!(
                                "Cloud cover is {:.0}%. Switched to Dark Mode for today.",
                                cloud_pct
                            ))
                            .show();
                    }
                    emit_state(&app_handle, &bg_runtime);
                }

                let mut tick_count: u64 = 0;
                let mut interval = tokio::time::interval(std::time::Duration::from_secs(15));
                loop {
                    interval.tick().await;
                    tick_count += 1;

                    let changed = {
                        let mut guard = bg_runtime.lock();
                        guard.tick()
                    };
                    if changed {
                        emit_state(&app_handle, &bg_runtime);
                    }

                    // Every ~60 minutes (240 ticks * 15s), refresh weather if enabled
                    if tick_count % 240 == 0 {
                        let weather_on = {
                            let g = bg_runtime.lock();
                            g.config.weather.enabled && g.config.location.is_some()
                        };
                        if weather_on {
                            if let Some(cloud_pct) = scheduler::refresh_weather_async(
                                bg_runtime.clone(),
                                "Scheduled forecast".to_string(),
                            )
                            .await
                            {
                                let _ = app_handle
                                    .notification()
                                    .builder()
                                    .title("Solace — Overcast Day")
                                    .body(format!(
                                        "Cloud cover is {:.0}%. Switched to Dark Mode for today.",
                                        cloud_pct
                                    ))
                                    .show();
                            }
                            emit_state(&app_handle, &bg_runtime);
                        }
                    }
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_state,
            toggle_mode,
            respond_first_switch,
            keep_light_today,
            set_wallpapers_enabled,
            set_wallpaper_path,
            set_schedule_enabled,
            set_schedule_offsets,
            set_location,
            search_cities,
            auto_detect_location,
            set_warmth_config,
            preview_warmth,
            set_weather_enabled,
            set_weather_sensitivity,
            toggle_temperature_unit,
            refresh_weather,
            update_shortcut,
            complete_onboarding,
            hide_panel,
            quit_app,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Solace");
}
