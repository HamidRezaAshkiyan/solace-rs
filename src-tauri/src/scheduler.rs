use crate::appearance;
use crate::config::{
    ManualOverride, SolaceConfig, ThemeMode, WeatherDecisionRecord,
};
use crate::solar::{self, SolarTimes};
use crate::wallpaper;
use crate::warmth;
use crate::weather::{self, WeatherSnapshot};
use chrono::Local;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};

/// Pending gentle prompt on fresh install before Solace's very first automatic switch.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FirstSwitchPrompt {
    pub target_mode: ThemeMode,
    pub reason: String,
}

/// Full UI snapshot sent to the React frontend via IPC and events.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SolaceStateSnapshot {
    pub config: SolaceConfig,
    pub current_mode: ThemeMode,
    pub mode_subtitle: String,
    pub solar: Option<SolarTimes>,
    pub weather: Option<WeatherSnapshot>,
    pub weather_error: Option<String>,
    pub weather_triggered_dark: bool,
    pub effective_kelvin: u32,
    pub warmth_active: bool,
    pub light_wallpaper_missing: bool,
    pub dark_wallpaper_missing: bool,
    pub first_switch_prompt: Option<FirstSwitchPrompt>,
    pub permission_warning: Option<String>,
    pub platform: String,
    pub version: String,
}

pub struct SolaceRuntime {
    pub config: SolaceConfig,
    pub current_mode: ThemeMode,
    pub weather: Option<WeatherSnapshot>,
    pub weather_error: Option<String>,
    pub effective_kelvin: u32,
    pub warmth_active: bool,
    pub preview_kelvin: Option<u32>,
    pub first_switch_prompt: Option<FirstSwitchPrompt>,
    pub permission_warning: Option<String>,
    pub last_applied_wallpaper: Option<String>,
}

impl SolaceRuntime {
    pub fn new() -> Self {
        let config = SolaceConfig::load();
        let current_mode = appearance::get_system_appearance();
        Self {
            config,
            current_mode,
            weather: None,
            weather_error: None,
            effective_kelvin: warmth::NEUTRAL_KELVIN,
            warmth_active: false,
            preview_kelvin: None,
            first_switch_prompt: None,
            permission_warning: None,
            last_applied_wallpaper: None,
        }
    }

    pub fn compute_solar(&self) -> Option<SolarTimes> {
        self.config.location.as_ref().map(|loc| {
            solar::compute_today_solar(
                loc.latitude,
                loc.longitude,
                self.config.schedule.sunrise_offset_min,
                self.config.schedule.sunset_offset_min,
            )
        })
    }

    pub fn snapshot(&self) -> SolaceStateSnapshot {
        let solar_times = self.compute_solar();
        let today_str = Local::now().format("%Y-%m-%d").to_string();
        let weather_triggered_dark = self.config.weather.enabled
            && self.config.weather.today_dark_override
            && self.config.weather.last_decision_date.as_deref() == Some(today_str.as_str())
            && self.config.weather.keep_light_date.as_deref() != Some(today_str.as_str())
            && solar_times.as_ref().map(|s| s.is_daytime).unwrap_or(true);

        let mode_subtitle = if let Some(ref ov) = self.config.manual_override {
            format!("{} · holds until next transition", ov.reason)
        } else if weather_triggered_dark && self.current_mode == ThemeMode::Dark {
            "Switched to dark for overcast weather".to_string()
        } else if self.config.schedule.enabled && solar_times.is_some() {
            let st = solar_times.as_ref().unwrap();
            if st.is_daytime {
                format!("Next switch at sunset ({})", st.effective_sunset_formatted)
            } else {
                format!("Next switch at sunrise ({})", st.effective_sunrise_formatted)
            }
        } else {
            "Manual control".to_string()
        };

        SolaceStateSnapshot {
            config: self.config.clone(),
            current_mode: self.current_mode,
            mode_subtitle,
            solar: solar_times,
            weather: self.weather.clone(),
            weather_error: self.weather_error.clone(),
            weather_triggered_dark,
            effective_kelvin: self.preview_kelvin.unwrap_or(self.effective_kelvin),
            warmth_active: self.preview_kelvin.is_some() || self.warmth_active,
            light_wallpaper_missing: !wallpaper::wallpaper_exists(&self.config.wallpapers.light_path),
            dark_wallpaper_missing: !wallpaper::wallpaper_exists(&self.config.wallpapers.dark_path),
            first_switch_prompt: self.first_switch_prompt.clone(),
            permission_warning: self.permission_warning.clone(),
            platform: std::env::consts::OS.to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    /// Apply mode to OS + paired wallpaper if enabled.
    pub fn apply_mode(&mut self, mode: ThemeMode, force_wallpaper: bool) {
        if self.current_mode != mode {
            match appearance::set_system_appearance(mode) {
                Ok(()) => {
                    self.current_mode = mode;
                    self.permission_warning = None;
                }
                Err(e) => {
                    tracing::warn!("Failed to switch OS appearance: {e}");
                    self.permission_warning = Some(e);
                }
            }
        }
        self.sync_wallpaper_for_mode(mode, force_wallpaper);
    }

    pub fn sync_wallpaper_for_mode(&mut self, mode: ThemeMode, force: bool) {
        if !self.config.wallpapers.enabled {
            return;
        }
        let target = match mode {
            ThemeMode::Light => self.config.wallpapers.light_path.clone(),
            ThemeMode::Dark => self.config.wallpapers.dark_path.clone(),
        };
        if let Some(path) = target {
            if force || self.last_applied_wallpaper.as_deref() != Some(path.as_str()) {
                if wallpaper::set_wallpaper(&path).is_ok() {
                    self.last_applied_wallpaper = Some(path);
                }
            }
        }
    }

    /// User manually toggled mode (via card, shortcut, or tray).
    pub fn manual_toggle(&mut self, reason: &str) -> ThemeMode {
        let next_mode = self.current_mode.toggled();
        self.set_manual_mode(next_mode, reason);
        next_mode
    }

    pub fn set_manual_mode(&mut self, mode: ThemeMode, reason: &str) {
        self.first_switch_prompt = None;
        self.config.first_switch_confirmed = true;

        let until_key = self
            .compute_solar()
            .map(|s| solar::next_transition_key(&s))
            .unwrap_or_else(|| "manual".to_string());

        self.config.manual_override = Some(ManualOverride {
            mode,
            until_transition: until_key,
            reason: reason.to_string(),
            set_at: Local::now(),
        });

        // If user switches to Light while weather had forced Dark today, record Keep Light
        if mode == ThemeMode::Light && self.config.weather.today_dark_override {
            self.config.weather.keep_light_date =
                Some(Local::now().format("%Y-%m-%d").to_string());
        }

        self.apply_mode(mode, false);
        let _ = self.config.save();
    }

    /// Evaluate today's weather decision from the latest snapshot.
    /// Returns `Some(avg_cloud)` if a fresh gloomy-day switch to Dark Mode was triggered.
    pub fn evaluate_weather_decision(&mut self, trigger: &str) -> Option<f64> {
        if !self.config.weather.enabled {
            return None;
        }
        let snap = self.weather.as_ref()?;
        let today = Local::now().format("%Y-%m-%d").to_string();
        let threshold = self.config.weather.sensitivity.threshold_percent();
        let should_dark = snap.avg_daylight_cloud_percent >= threshold;
        let outcome_mode = if should_dark {
            ThemeMode::Dark
        } else {
            ThemeMode::Light
        };

        let previously_dark = self.config.weather.today_dark_override
            && self.config.weather.last_decision_date.as_deref() == Some(today.as_str());

        self.config.weather.last_decision_date = Some(today.clone());
        self.config.weather.today_dark_override = should_dark;

        self.config.push_weather_decision(WeatherDecisionRecord {
            date: today.clone(),
            decided_at: Local::now().format("%H:%M").to_string(),
            avg_cloud_percent: snap.avg_daylight_cloud_percent,
            threshold_percent: threshold,
            sensitivity: self.config.weather.sensitivity,
            outcome_mode,
            trigger: format!("{} ({})", trigger, snap.source_note),
        });
        let _ = self.config.save();

        let keep_light = self.config.weather.keep_light_date.as_deref() == Some(today.as_str());
        if should_dark && !previously_dark && !keep_light {
            Some(snap.avg_daylight_cloud_percent)
        } else {
            None
        }
    }

    /// Periodic tick: sync external OS appearance, evaluate solar & weather schedule, and update warmth gamma.
    /// Returns `true` if state visibly changed (so UI & tray can update).
    pub fn tick(&mut self) -> bool {
        let mut changed = false;

        // 1. Detect external OS appearance changes (Control Center / System Settings)
        let os_mode = appearance::get_system_appearance();
        if os_mode != self.current_mode {
            self.current_mode = os_mode;
            self.sync_wallpaper_for_mode(os_mode, false);
            // Respect external switch until next scheduled transition
            if let Some(solar_times) = self.compute_solar() {
                self.config.manual_override = Some(ManualOverride {
                    mode: os_mode,
                    until_transition: solar::next_transition_key(&solar_times),
                    reason: "System change".to_string(),
                    set_at: Local::now(),
                });
                let _ = self.config.save();
            }
            changed = true;
        }

        // 2. Solar & Weather schedule evaluation
        let solar_times = self.compute_solar();
        let today = Local::now().format("%Y-%m-%d").to_string();

        // Expire manual override if the scheduled transition has passed
        if let (Some(ref ov), Some(ref st)) = (&self.config.manual_override, &solar_times) {
            let current_next_key = solar::next_transition_key(st);
            if ov.until_transition != current_next_key && ov.until_transition != "manual" {
                self.config.manual_override = None;
                let _ = self.config.save();
                changed = true;
            }
        }

        if self.config.manual_override.is_none() {
            let mut desired_mode: Option<(ThemeMode, String)> = None;

            if self.config.schedule.enabled {
                if let Some(ref st) = solar_times {
                    if st.is_daytime {
                        desired_mode = Some((ThemeMode::Light, "Solar sunrise".to_string()));
                    } else {
                        desired_mode = Some((ThemeMode::Dark, "Solar sunset".to_string()));
                    }
                }
            }

            // Weather-aware override during daytime
            let is_daytime = solar_times.as_ref().map(|s| s.is_daytime).unwrap_or(true);
            let keep_light =
                self.config.weather.keep_light_date.as_deref() == Some(today.as_str());
            if self.config.weather.enabled
                && is_daytime
                && self.config.weather.today_dark_override
                && self.config.weather.last_decision_date.as_deref() == Some(today.as_str())
                && !keep_light
            {
                desired_mode = Some((ThemeMode::Dark, "Overcast weather".to_string()));
            }

            if let Some((target, reason)) = desired_mode {
                if target != self.current_mode {
                    if !self.config.first_switch_confirmed {
                        // Gentle first-switch prompt instead of surprising the user
                        let prompt = FirstSwitchPrompt {
                            target_mode: target,
                            reason,
                        };
                        if self.first_switch_prompt.as_ref().map(|p| p.target_mode)
                            != Some(prompt.target_mode)
                        {
                            self.first_switch_prompt = Some(prompt);
                            changed = true;
                        }
                    } else {
                        self.apply_mode(target, false);
                        changed = true;
                    }
                }
            }
        }

        // 3. Evening Warmth smooth ramp & wake re-assertion
        if self.preview_kelvin.is_none() {
            let now = Local::now();
            let current_min = (now.format("%H").to_string().parse::<i32>().unwrap_or(0) * 60)
                + now.format("%M").to_string().parse::<i32>().unwrap_or(0);
            let (eff_k, active) =
                warmth::compute_effective_kelvin(&self.config.warmth, solar_times.as_ref(), current_min);

            if eff_k != self.effective_kelvin || active != self.warmth_active {
                self.effective_kelvin = eff_k;
                self.warmth_active = active;
                changed = true;
            }

            if active {
                // Re-assert gamma every tick so display wake / monitor plug never drops warmth
                let _ = warmth::apply_kelvin(eff_k, self.config.warmth.compatibility_mode);
            }
        }

        changed
    }
}

pub type SharedRuntime = std::sync::Arc<Mutex<SolaceRuntime>>;

/// Async helper to fetch latest weather and evaluate today's decision.
pub async fn refresh_weather_async(runtime: SharedRuntime, trigger: String) -> Option<f64> {
    let (lat, lon, sr_h, ss_h) = {
        let guard = runtime.lock();
        let loc = guard.config.location.clone()?;
        let solar_times = guard.compute_solar();
        let sr_h = solar_times
            .as_ref()
            .map(|s| (s.effective_sunrise_min / 60).clamp(0, 23) as u32)
            .unwrap_or(7);
        let ss_h = solar_times
            .as_ref()
            .map(|s| (s.effective_sunset_min / 60).clamp(0, 23) as u32)
            .unwrap_or(18);
        (loc.latitude, loc.longitude, sr_h, ss_h)
    };

    match weather::fetch_weather(lat, lon, sr_h, ss_h).await {
        Ok(snap) => {
            let mut guard = runtime.lock();
            guard.weather = Some(snap);
            guard.weather_error = None;
            let triggered_cloud = guard.evaluate_weather_decision(&trigger);
            guard.tick();
            triggered_cloud
        }
        Err(err) => {
            let mut guard = runtime.lock();
            guard.weather_error = Some(err);
            None
        }
    }
}
