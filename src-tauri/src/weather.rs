use crate::config::LocationConfig;
use chrono::Local;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WeatherSnapshot {
    pub fetched_at: String,
    pub temperature_c: f64,
    pub weather_code: u32,
    pub condition_label: String,
    pub condition_icon: String,
    pub current_cloud_percent: f64,
    /// Average cloud cover across today's daylight hours (used for the morning decision)
    pub avg_daylight_cloud_percent: f64,
    /// Average rain probability (%) across daylight hours
    pub avg_rain_probability: f64,
    /// Max UV index today
    pub max_uv_index: f64,
    pub source_note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CitySearchResult {
    pub name: String,
    pub admin1: Option<String>,
    pub country: String,
    pub latitude: f64,
    pub longitude: f64,
}

pub fn decode_wmo_code(code: u32) -> (&'static str, &'static str) {
    match code {
        0 => ("Clear", "sun"),
        1 => ("Mostly Clear", "sun-cloud"),
        2 => ("Partly Cloudy", "cloud-sun"),
        3 => ("Overcast", "cloud"),
        45 | 48 => ("Foggy", "cloud-fog"),
        51 | 53 | 55 | 56 | 57 => ("Drizzle", "cloud-drizzle"),
        61 | 63 | 65 | 66 | 67 => ("Rain", "cloud-rain"),
        71 | 73 | 75 | 77 => ("Snow", "cloud-snow"),
        80 | 81 | 82 => ("Rain Showers", "cloud-rain"),
        85 | 86 => ("Snow Showers", "cloud-snow"),
        95 | 96 | 99 => ("Thunderstorm", "cloud-lightning"),
        _ => ("Cloudy", "cloud"),
    }
}

#[derive(Deserialize)]
struct OpenMeteoResponse {
    current: Option<OpenMeteoCurrent>,
    hourly: Option<OpenMeteoHourly>,
}

#[derive(Deserialize)]
struct OpenMeteoCurrent {
    temperature_2m: f64,
    cloud_cover: f64,
    weather_code: u32,
}

#[derive(Deserialize)]
struct OpenMeteoHourly {
    time: Vec<String>,
    cloud_cover: Vec<f64>,
    precipitation_probability: Option<Vec<Option<f64>>>,
    uv_index: Option<Vec<Option<f64>>>,
}

/// Fetch today's weather & daylight cloud forecast from Open-Meteo (free, no API key required).
pub async fn fetch_weather(
    lat: f64,
    lon: f64,
    sunrise_hour: u32,
    sunset_hour: u32,
) -> Result<WeatherSnapshot, String> {
    let url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={lat:.4}&longitude={lon:.4}&current=temperature_2m,cloud_cover,weather_code&hourly=cloud_cover,precipitation_probability,uv_index&forecast_days=1&timezone=auto"
    );

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Weather unavailable: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("Weather service returned HTTP {}", resp.status()));
    }

    let data: OpenMeteoResponse = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse weather response: {e}"))?;

    let current = data
        .current
        .ok_or_else(|| "Weather response missing current conditions".to_string())?;
    let (condition_label, condition_icon) = decode_wmo_code(current.weather_code);

    let mut cloud_sum = 0.0;
    let mut cloud_cnt = 0.0;
    let mut rain_sum = 0.0;
    let mut rain_cnt = 0.0;
    let mut max_uv = 0.0_f64;

    if let Some(hourly) = data.hourly {
        let start_h = sunrise_hour.min(23);
        let end_h = sunset_hour.clamp(start_h, 23);
        for (i, t) in hourly.time.iter().enumerate() {
            // Format: "2026-11-27T08:00"
            let hour = t
                .split('T')
                .nth(1)
                .and_then(|hhmm| hhmm.split(':').next())
                .and_then(|h| h.parse::<u32>().ok())
                .unwrap_or(12);

            if hour >= start_h && hour <= end_h {
                if let Some(&cc) = hourly.cloud_cover.get(i) {
                    cloud_sum += cc;
                    cloud_cnt += 1.0;
                }
                if let Some(ref probs) = hourly.precipitation_probability {
                    if let Some(Some(p)) = probs.get(i) {
                        rain_sum += *p;
                        rain_cnt += 1.0;
                    }
                }
                if let Some(ref uvs) = hourly.uv_index {
                    if let Some(Some(u)) = uvs.get(i) {
                        if *u > max_uv {
                            max_uv = *u;
                        }
                    }
                }
            }
        }
    }

    let avg_daylight_cloud_percent = if cloud_cnt > 0.0 {
        (cloud_sum / cloud_cnt).round()
    } else {
        current.cloud_cover.round()
    };

    let avg_rain_probability = if rain_cnt > 0.0 {
        (rain_sum / rain_cnt).round()
    } else {
        0.0
    };

    let source_note = if cloud_cnt > 0.0 {
        "Daylight forecast".to_string()
    } else {
        "Current reading".to_string()
    };

    Ok(WeatherSnapshot {
        fetched_at: Local::now().format("%H:%M").to_string(),
        temperature_c: current.temperature_2m,
        weather_code: current.weather_code,
        condition_label: condition_label.to_string(),
        condition_icon: condition_icon.to_string(),
        current_cloud_percent: current.cloud_cover.round(),
        avg_daylight_cloud_percent,
        avg_rain_probability,
        max_uv_index: (max_uv * 10.0).round() / 10.0,
        source_note,
    })
}

#[derive(Deserialize)]
struct GeocodingResponse {
    results: Option<Vec<GeocodingItem>>,
}

#[derive(Deserialize)]
struct GeocodingItem {
    name: String,
    admin1: Option<String>,
    country: Option<String>,
    latitude: f64,
    longitude: f64,
}

/// Search cities by name using Open-Meteo Geocoding API (free, no API key).
pub async fn search_cities(query: &str) -> Result<Vec<CitySearchResult>, String> {
    let trimmed = query.trim();
    if trimmed.len() < 2 {
        return Ok(Vec::new());
    }
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(8))
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client
        .get("https://geocoding-api.open-meteo.com/v1/search")
        .query(&[("name", trimmed), ("count", "6"), ("language", "en"), ("format", "json")])
        .send()
        .await
        .map_err(|e| format!("City search failed: {e}"))?;

    let data: GeocodingResponse = resp.json().await.map_err(|e| e.to_string())?;
    let list = data
        .results
        .unwrap_or_default()
        .into_iter()
        .map(|item| CitySearchResult {
            name: item.name,
            admin1: item.admin1,
            country: item.country.unwrap_or_default(),
            latitude: item.latitude,
            longitude: item.longitude,
        })
        .collect();
    Ok(list)
}

#[derive(Deserialize)]
struct IpGeoResponse {
    city: Option<String>,
    country_name: Option<String>,
    latitude: Option<f64>,
    longitude: Option<f64>,
}

/// Optional one-click approximate city detection (only invoked when user clicks Auto-Detect).
pub async fn auto_detect_location() -> Result<LocationConfig, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(8))
        .user_agent("Solace/1.0")
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client
        .get("https://ipapi.co/json/")
        .send()
        .await
        .map_err(|e| format!("Auto-detect failed: {e}"))?;

    let geo: IpGeoResponse = resp.json().await.map_err(|e| e.to_string())?;
    let lat = geo.latitude.ok_or("No latitude returned")?;
    let lon = geo.longitude.ok_or("No longitude returned")?;
    Ok(LocationConfig {
        city: geo.city.unwrap_or_else(|| "Unknown City".to_string()),
        country: geo.country_name.unwrap_or_default(),
        latitude: lat,
        longitude: lon,
    })
}
