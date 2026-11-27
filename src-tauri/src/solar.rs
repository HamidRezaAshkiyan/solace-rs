use chrono::{Datelike, Local, NaiveDate, Offset};
use serde::{Deserialize, Serialize};

/// Exact local solar calculation (NOAA Solar Calculations algorithm).
/// Runs 100% locally on-device with zero network calls.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SolarTimes {
    /// Exact astronomical sunrise in minutes from local midnight (0..1440)
    pub sunrise_min: i32,
    /// Exact astronomical sunset in minutes from local midnight (0..1440)
    pub sunset_min: i32,
    /// Sunrise after applying user offset (-120..+120)
    pub effective_sunrise_min: i32,
    /// Sunset after applying user offset (-120..+120)
    pub effective_sunset_min: i32,
    pub sunrise_formatted: String,
    pub sunset_formatted: String,
    pub effective_sunrise_formatted: String,
    pub effective_sunset_formatted: String,
    pub current_minute: i32,
    pub is_daytime: bool,
}

pub fn format_minutes_12h(total_min: i32) -> String {
    let clamped = total_min.rem_euclid(1440);
    let h24 = clamped / 60;
    let m = clamped % 60;
    let suffix = if h24 < 12 { "AM" } else { "PM" };
    let h12 = match h24 % 12 {
        0 => 12,
        v => v,
    };
    format!("{h12}:{m:02} {suffix}")
}

/// Compute Julian Day Number at 0h UTC for a Gregorian date.
fn julian_day(date: NaiveDate) -> f64 {
    let mut y = date.year() as f64;
    let mut m = date.month() as f64;
    let d = date.day() as f64;
    if m <= 2.0 {
        y -= 1.0;
        m += 12.0;
    }
    let a = (y / 100.0).floor();
    let b = 2.0 - a + (a / 4.0).floor();
    (365.25 * (y + 4716.0)).floor() + (30.6001 * (m + 1.0)).floor() + d + b - 1524.5
}

/// Computes local sunrise & sunset minutes from midnight using NOAA solar equations.
/// `tz_offset_hours` is local UTC offset in hours (e.g. +3.5 for Iran, -5.0 for EST).
pub fn compute_solar_minutes(
    date: NaiveDate,
    latitude: f64,
    longitude: f64,
    tz_offset_hours: f64,
) -> (i32, i32) {
    // Evaluate at local noon for high accuracy
    let jd = julian_day(date) + 0.5 - tz_offset_hours / 24.0;
    let jc = (jd - 2451545.0) / 36525.0;

    let geom_mean_long_sun = (280.46646 + jc * (36000.76983 + jc * 0.0003032)).rem_euclid(360.0);
    let geom_mean_anom_sun = 357.52911 + jc * (35999.05029 - 0.0001537 * jc);
    let eccent_earth_orbit = 0.016708634 - jc * (0.000042037 + 0.0000001267 * jc);

    let anom_rad = geom_mean_anom_sun.to_radians();
    let sun_eq_of_ctr = anom_rad.sin() * (1.914602 - jc * (0.004817 + 0.000014 * jc))
        + (2.0 * anom_rad).sin() * (0.019993 - 0.000101 * jc)
        + (3.0 * anom_rad).sin() * 0.000289;

    let sun_true_long = geom_mean_long_sun + sun_eq_of_ctr;
    let omega = (125.04 - 1934.136 * jc).to_radians();
    let sun_app_long = sun_true_long - 0.00569 - 0.00478 * omega.sin();

    let mean_obliq_ecliptic =
        23.0 + (26.0 + ((21.448 - jc * (46.815 + jc * (0.00059 - jc * 0.001813))) / 60.0)) / 60.0;
    let obliq_corr = mean_obliq_ecliptic + 0.00256 * omega.cos();

    let sun_declin = (obliq_corr.to_radians().sin() * sun_app_long.to_radians().sin()).asin();

    let var_y = (obliq_corr.to_radians() / 2.0).tan().powi(2);
    let gml_rad = geom_mean_long_sun.to_radians();
    let eq_of_time_min = 4.0
        * (var_y * (2.0 * gml_rad).sin()
            - 2.0 * eccent_earth_orbit * anom_rad.sin()
            + 4.0 * eccent_earth_orbit * var_y * anom_rad.sin() * (2.0 * gml_rad).cos()
            - 0.5 * var_y * var_y * (4.0 * gml_rad).sin()
            - 1.25 * eccent_earth_orbit * eccent_earth_orbit * (2.0 * anom_rad).sin())
        .to_degrees();

    let lat_rad = latitude.to_radians();
    let zenith_rad = 90.833_f64.to_radians();
    let cos_ha = (zenith_rad.cos() / (lat_rad.cos() * sun_declin.cos()))
        - lat_rad.tan() * sun_declin.tan();

    // Solar noon in minutes from local midnight
    let solar_noon_min = 720.0 - 4.0 * longitude - eq_of_time_min + tz_offset_hours * 60.0;

    if cos_ha > 1.0 {
        // Polar night fallback
        return (720, 720);
    }
    if cos_ha < -1.0 {
        // Polar day fallback
        return (0, 1439);
    }

    let ha_deg = cos_ha.acos().to_degrees();
    let sunrise = (solar_noon_min - ha_deg * 4.0).round() as i32;
    let sunset = (solar_noon_min + ha_deg * 4.0).round() as i32;

    (sunrise.clamp(0, 1439), sunset.clamp(0, 1439))
}

pub fn compute_today_solar(
    latitude: f64,
    longitude: f64,
    sunrise_offset_min: i32,
    sunset_offset_min: i32,
) -> SolarTimes {
    let now = Local::now();
    let tz_offset_hours = now.offset().fix().local_minus_utc() as f64 / 3600.0;
    let (sunrise_min, sunset_min) =
        compute_solar_minutes(now.date_naive(), latitude, longitude, tz_offset_hours);

    let effective_sunrise_min = (sunrise_min + sunrise_offset_min.clamp(-120, 120)).clamp(0, 1439);
    let effective_sunset_min = (sunset_min + sunset_offset_min.clamp(-120, 120)).clamp(0, 1439);

    let current_minute = (now.format("%H").to_string().parse::<i32>().unwrap_or(0) * 60)
        + now.format("%M").to_string().parse::<i32>().unwrap_or(0);

    let is_daytime = current_minute >= effective_sunrise_min && current_minute < effective_sunset_min;

    SolarTimes {
        sunrise_min,
        sunset_min,
        effective_sunrise_min,
        effective_sunset_min,
        sunrise_formatted: format_minutes_12h(sunrise_min),
        sunset_formatted: format_minutes_12h(sunset_min),
        effective_sunrise_formatted: format_minutes_12h(effective_sunrise_min),
        effective_sunset_formatted: format_minutes_12h(effective_sunset_min),
        current_minute,
        is_daytime,
    }
}

/// Returns a deterministic key for the next scheduled transition (used to expire manual overrides).
pub fn next_transition_key(solar: &SolarTimes) -> String {
    let today = Local::now().date_naive();
    if solar.current_minute < solar.effective_sunrise_min {
        format!("{}-sunrise", today)
    } else if solar.current_minute < solar.effective_sunset_min {
        format!("{}-sunset", today)
    } else {
        let tomorrow = today.succ_opt().unwrap_or(today);
        format!("{}-sunrise", tomorrow)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn london_equinox_solar_times() {
        let date = NaiveDate::from_ymd_opt(2026, 3, 21).unwrap();
        let (sr, ss) = compute_solar_minutes(date, 51.5074, -0.1278, 0.0);
        // Around 6:00 AM (360) and 6:15 PM (1095)
        assert!((350..=375).contains(&sr), "sunrise was {sr}");
        assert!((1085..=1110).contains(&ss), "sunset was {ss}");
    }

    #[test]
    fn tehran_summer_solstice_solar_times() {
        let date = NaiveDate::from_ymd_opt(2026, 6, 21).unwrap();
        let (sr, ss) = compute_solar_minutes(date, 35.6892, 51.3890, 3.5);
        // Tehran summer solstice: ~4:50 AM (290) and ~7:25 PM (1165)
        assert!((275..=305).contains(&sr), "sunrise was {sr}");
        assert!((1150..=1185).contains(&ss), "sunset was {ss}");
    }

    #[test]
    fn format_12h_edge_cases() {
        assert_eq!(format_minutes_12h(0), "12:00 AM");
        assert_eq!(format_minutes_12h(720), "12:00 PM");
        assert_eq!(format_minutes_12h(479), "7:59 AM");
        assert_eq!(format_minutes_12h(988), "4:28 PM");
    }
}
