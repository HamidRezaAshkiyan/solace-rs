use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
    Light,
    Dark,
}

impl ThemeMode {
    pub fn toggled(self) -> Self {
        match self {
            ThemeMode::Light => ThemeMode::Dark,
            ThemeMode::Dark => ThemeMode::Light,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            ThemeMode::Light => "Light Mode",
            ThemeMode::Dark => "Dark Mode",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TemperatureUnit {
    Celsius,
    Fahrenheit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WeatherSensitivity {
    /// >= 50% average daylight cloud cover
    Cloudy,
    /// >= 70% average daylight cloud cover (default)
    Overcast,
    /// >= 90% average daylight cloud cover
    Gloomy,
}

impl WeatherSensitivity {
    pub fn threshold_percent(self) -> f64 {
        match self {
            WeatherSensitivity::Cloudy => 50.0,
            WeatherSensitivity::Overcast => 70.0,
            WeatherSensitivity::Gloomy => 90.0,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            WeatherSensitivity::Cloudy => "Cloudy",
            WeatherSensitivity::Overcast => "Overcast",
            WeatherSensitivity::Gloomy => "Gloomy",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WarmthSchedule {
    Sunset,
    FixedTime { hour: u8, minute: u8 },
    Always,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocationConfig {
    pub city: String,
    pub country: String,
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManualOverride {
    pub mode: ThemeMode,
    /// Key identifying the next transition slot (e.g. "2026-11-27-sunset" or "2026-11-28-sunrise")
    /// so the override holds across restarts until that transition arrives.
    pub until_transition: String,
    pub reason: String,
    pub set_at: DateTime<Local>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WeatherDecisionRecord {
    pub date: String,
    pub decided_at: String,
    pub avg_cloud_percent: f64,
    pub threshold_percent: f64,
    pub sensitivity: WeatherSensitivity,
    pub outcome_mode: ThemeMode,
    pub trigger: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WallpaperConfig {
    pub enabled: bool,
    pub light_path: Option<String>,
    pub dark_path: Option<String>,
    /// Original wallpaper saved before Solace took over, restored if user disables wallpapers.
    pub original_path: Option<String>,
}

impl Default for WallpaperConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            light_path: None,
            dark_path: None,
            original_path: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleConfig {
    pub enabled: bool,
    /// Offset in minutes from exact sunrise (-120..=+120)
    pub sunrise_offset_min: i32,
    /// Offset in minutes from exact sunset (-120..=+120)
    pub sunset_offset_min: i32,
}

impl Default for ScheduleConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            sunrise_offset_min: 0,
            sunset_offset_min: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WarmthConfig {
    pub enabled: bool,
    /// Target color temperature in Kelvin (1900..=6500)
    pub kelvin: u32,
    pub schedule: WarmthSchedule,
    /// Compatibility mode for displays where hardware gamma LUT is clamped (e.g. XDR)
    pub compatibility_mode: bool,
}

impl Default for WarmthConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            kelvin: 3400,
            schedule: WarmthSchedule::FixedTime {
                hour: 20,
                minute: 0,
            },
            compatibility_mode: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WeatherConfig {
    pub enabled: bool,
    pub sensitivity: WeatherSensitivity,
    pub unit: TemperatureUnit,
    /// Date ("YYYY-MM-DD") for which today's morning decision was made
    pub last_decision_date: Option<String>,
    pub today_dark_override: bool,
    /// If the user clicked "Keep Light" after a gloomy weather switch today
    pub keep_light_date: Option<String>,
}

impl Default for WeatherConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            sensitivity: WeatherSensitivity::Overcast,
            unit: TemperatureUnit::Celsius,
            last_decision_date: None,
            today_dark_override: false,
            keep_light_date: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SolaceConfig {
    pub onboarding_completed: bool,
    /// On a fresh install, Solace asks before its very first automatic switch.
    pub first_switch_confirmed: bool,
    pub location: Option<LocationConfig>,
    pub schedule: ScheduleConfig,
    pub wallpapers: WallpaperConfig,
    pub warmth: WarmthConfig,
    pub weather: WeatherConfig,
    pub shortcut: String,
    pub manual_override: Option<ManualOverride>,
    /// Last 10 daily weather decisions for full transparency
    pub weather_decisions: Vec<WeatherDecisionRecord>,
}

impl Default for SolaceConfig {
    fn default() -> Self {
        #[cfg(target_os = "macos")]
        let default_shortcut = "Command+Shift+D".to_string();
        #[cfg(not(target_os = "macos"))]
        let default_shortcut = "Control+Shift+D".to_string();

        Self {
            onboarding_completed: false,
            first_switch_confirmed: false,
            location: None,
            schedule: ScheduleConfig::default(),
            wallpapers: WallpaperConfig::default(),
            warmth: WarmthConfig::default(),
            weather: WeatherConfig::default(),
            shortcut: default_shortcut,
            manual_override: None,
            weather_decisions: Vec::new(),
        }
    }
}

impl SolaceConfig {
    pub fn config_path() -> PathBuf {
        let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        base.join("solace-rs").join("config.json")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if let Ok(raw) = fs::read_to_string(&path) {
            if let Ok(cfg) = serde_json::from_str::<SolaceConfig>(&raw) {
                return cfg;
            }
        }
        Self::default()
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(&path, json).map_err(|e| e.to_string())
    }

    pub fn push_weather_decision(&mut self, record: WeatherDecisionRecord) {
        // Replace existing record for the same date if re-evaluated
        self.weather_decisions.retain(|r| r.date != record.date);
        self.weather_decisions.insert(0, record);
        self.weather_decisions.truncate(10);
    }
}
