export type ThemeMode = "light" | "dark";
export type TemperatureUnit = "celsius" | "fahrenheit";
export type WeatherSensitivity = "cloudy" | "overcast" | "gloomy";

export type WarmthSchedule =
  | { type: "sunset" }
  | { type: "fixed_time"; hour: number; minute: number }
  | { type: "always" };

export interface LocationConfig {
  city: string;
  country: string;
  latitude: number;
  longitude: number;
}

export interface ManualOverride {
  mode: ThemeMode;
  untilTransition: string;
  reason: string;
  setAt: string;
}

export interface WeatherDecisionRecord {
  date: string;
  decidedAt: string;
  avgCloudPercent: number;
  thresholdPercent: number;
  sensitivity: WeatherSensitivity;
  outcomeMode: ThemeMode;
  trigger: string;
}

export interface WallpaperConfig {
  enabled: boolean;
  lightPath: string | null;
  darkPath: string | null;
  originalPath: string | null;
}

export interface ScheduleConfig {
  enabled: boolean;
  sunriseOffsetMin: number;
  sunsetOffsetMin: number;
}

export interface WarmthConfig {
  enabled: boolean;
  kelvin: number;
  schedule: WarmthSchedule;
  compatibilityMode: boolean;
}

export interface WeatherConfig {
  enabled: boolean;
  sensitivity: WeatherSensitivity;
  unit: TemperatureUnit;
  lastDecisionDate: string | null;
  todayDarkOverride: boolean;
  keepLightDate: string | null;
}

export interface SolaceConfig {
  onboardingCompleted: boolean;
  firstSwitchConfirmed: boolean;
  location: LocationConfig | null;
  schedule: ScheduleConfig;
  wallpapers: WallpaperConfig;
  warmth: WarmthConfig;
  weather: WeatherConfig;
  shortcut: string;
  manualOverride: ManualOverride | null;
  weatherDecisions: WeatherDecisionRecord[];
}

export interface SolarTimes {
  sunriseMin: number;
  sunsetMin: number;
  effectiveSunriseMin: number;
  effectiveSunsetMin: number;
  sunriseFormatted: string;
  sunsetFormatted: string;
  effectiveSunriseFormatted: string;
  effectiveSunsetFormatted: string;
  currentMinute: number;
  isDaytime: boolean;
}

export interface WeatherSnapshot {
  fetchedAt: string;
  temperatureC: number;
  weatherCode: number;
  conditionLabel: string;
  conditionIcon: string;
  currentCloudPercent: number;
  avgDaylightCloudPercent: number;
  avgRainProbability: number;
  maxUvIndex: number;
  sourceNote: string;
}

export interface FirstSwitchPrompt {
  targetMode: ThemeMode;
  reason: string;
}

export interface SolaceStateSnapshot {
  config: SolaceConfig;
  currentMode: ThemeMode;
  modeSubtitle: string;
  solar: SolarTimes | null;
  weather: WeatherSnapshot | null;
  weatherError: string | null;
  weatherTriggeredDark: boolean;
  effectiveKelvin: number;
  warmthActive: boolean;
  lightWallpaperMissing: boolean;
  darkWallpaperMissing: boolean;
  firstSwitchPrompt: FirstSwitchPrompt | null;
  permissionWarning: string | null;
  platform: string;
  version: string;
}

export interface CitySearchResult {
  name: string;
  admin1: string | null;
  country: string;
  latitude: number;
  longitude: number;
}
