import {
  Cloud,
  CloudDrizzle,
  CloudFog,
  CloudLightning,
  CloudRain,
  CloudSnow,
  CloudSun,
  Droplets,
  RefreshCw,
  Sun,
} from "lucide-react";
import { useState } from "react";
import { useSolaceStore } from "../store/useSolaceStore";
import type { WeatherSensitivity } from "../types/solace";

function WeatherIcon({ name, className }: { name: string; className?: string }) {
  switch (name) {
    case "sun":
      return <Sun className={className} />;
    case "sun-cloud":
    case "cloud-sun":
      return <CloudSun className={className} />;
    case "cloud-fog":
      return <CloudFog className={className} />;
    case "cloud-drizzle":
      return <CloudDrizzle className={className} />;
    case "cloud-rain":
      return <CloudRain className={className} />;
    case "cloud-snow":
      return <CloudSnow className={className} />;
    case "cloud-lightning":
      return <CloudLightning className={className} />;
    default:
      return <Cloud className={className} />;
  }
}

const SENSITIVITY_OPTIONS: Array<{
  id: WeatherSensitivity;
  label: string;
  tip: string;
}> = [
  { id: "cloudy", label: "Cloudy", tip: "Dark mode from 50% cloud cover" },
  { id: "overcast", label: "Overcast", tip: "Dark mode from 70% cloud cover (default)" },
  { id: "gloomy", label: "Gloomy", tip: "Dark mode only from 90% cloud cover" },
];

export function WeatherCard() {
  const {
    state,
    setWeatherEnabled,
    setWeatherSensitivity,
    toggleTemperatureUnit,
    refreshWeather,
  } = useSolaceStore();
  const [refreshing, setRefreshing] = useState(false);

  if (!state) return null;

  const isDark = state.currentMode === "dark";
  const wCfg = state.config.weather;
  const snap = state.weather;

  const formatTemp = (tempC: number) => {
    if (wCfg.unit === "fahrenheit") {
      return `${Math.round((tempC * 9) / 5 + 32)}°F`;
    }
    return `${Math.round(tempC)}°C`;
  };

  const handleRefresh = async () => {
    setRefreshing(true);
    try {
      await refreshWeather();
    } finally {
      setRefreshing(false);
    }
  };

  return (
    <div
      className={`rounded-2xl p-3.5 border transition-colors ${
        isDark
          ? "bg-[#1b1d24] border-white/8"
          : "bg-white border-black/7 shadow-[0_1px_3px_rgba(0,0,0,0.03)]"
      }`}
    >
      {/* Header */}
      <div className="flex items-center justify-between mb-2">
        <div className="flex items-center gap-2">
          <Cloud className="w-3.5 h-3.5 text-sky-400" />
          <span
            className={`text-[12.5px] font-semibold ${
              isDark ? "text-zinc-100" : "text-zinc-800"
            }`}
          >
            Weather
          </span>
        </div>

        <div className="flex items-center gap-1.5">
          {wCfg.enabled && (
            <button
              onClick={() => void handleRefresh()}
              title="Refresh forecast"
              className={`p-1 rounded-md cursor-pointer ${
                isDark ? "text-zinc-400 hover:bg-white/8" : "text-zinc-500 hover:bg-black/5"
              }`}
            >
              <RefreshCw className={`w-3 h-3 ${refreshing ? "animate-spin" : ""}`} />
            </button>
          )}
          <button
            onClick={() => void setWeatherEnabled(!wCfg.enabled)}
            className={`px-2.5 py-0.5 rounded-full text-[11px] font-medium transition-colors cursor-pointer ${
              wCfg.enabled
                ? isDark
                  ? "bg-sky-500/20 text-sky-300 ring-1 ring-sky-400/30"
                  : "bg-sky-500/15 text-sky-700 ring-1 ring-sky-500/25"
                : isDark
                  ? "bg-white/8 text-zinc-400 hover:bg-white/12"
                  : "bg-black/5 text-zinc-600 hover:bg-black/10"
            }`}
          >
            {wCfg.enabled ? "Active" : "Enable"}
          </button>
        </div>
      </div>

      {wCfg.enabled ? (
        <div className="space-y-2.5">
          {/* Current Conditions Row */}
          {!state.config.location ? (
            <p className="text-[11px] opacity-65">
              Set a city in Schedule above to enable weather-aware switching.
            </p>
          ) : state.weatherError ? (
            <p className="text-[11px] text-amber-500">{state.weatherError}</p>
          ) : snap ? (
            <>
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <WeatherIcon
                    name={snap.conditionIcon}
                    className={`w-5 h-5 ${
                      snap.conditionIcon === "sun" ? "text-amber-400" : "text-sky-400"
                    }`}
                  />
                  <div>
                    <div className="flex items-center gap-1.5 text-[12.5px] font-semibold">
                      <span>{snap.conditionLabel}</span>
                      <span className="opacity-40">·</span>
                      <button
                        onClick={() => void toggleTemperatureUnit()}
                        title="Click to toggle °C / °F"
                        className="hover:underline cursor-pointer tabular-nums"
                      >
                        {formatTemp(snap.temperatureC)}
                      </button>
                    </div>
                    <div className="text-[10.5px] opacity-65">
                      {state.weatherTriggeredDark
                        ? "Switched to dark mode"
                        : "No change expected"}
                    </div>
                  </div>
                </div>

                {/* Day averages chips */}
                <div className="flex items-center gap-2 text-[10.5px] opacity-75 tabular-nums">
                  <span title="Average daylight cloud cover" className="flex items-center gap-0.5">
                    <Cloud className="w-3 h-3" />
                    {Math.round(snap.avgDaylightCloudPercent)}%
                  </span>
                  <span title="Rain probability" className="flex items-center gap-0.5">
                    <Droplets className="w-3 h-3" />
                    {Math.round(snap.avgRainProbability)}%
                  </span>
                  <span title="Max UV Index">UV {snap.maxUvIndex.toFixed(0)}</span>
                </div>
              </div>

              {/* Sensitivity Segmented Control */}
              <div
                className={`grid grid-cols-3 gap-1 p-1 rounded-xl ${
                  isDark ? "bg-black/30" : "bg-zinc-100"
                }`}
              >
                {SENSITIVITY_OPTIONS.map((opt) => {
                  const active = wCfg.sensitivity === opt.id;
                  return (
                    <button
                      key={opt.id}
                      title={opt.tip}
                      onClick={() => void setWeatherSensitivity(opt.id)}
                      className={`py-1 rounded-lg text-[11px] font-medium transition-all cursor-pointer ${
                        active
                          ? isDark
                            ? "bg-white/14 text-white shadow-sm"
                            : "bg-white text-zinc-900 shadow-sm"
                          : isDark
                            ? "text-zinc-400 hover:text-zinc-200"
                            : "text-zinc-600 hover:text-zinc-900"
                      }`}
                    >
                      {opt.label}
                    </button>
                  );
                })}
              </div>
            </>
          ) : (
            <p className="text-[11px] opacity-60">Checking today’s forecast…</p>
          )}
        </div>
      ) : (
        <p className="text-[11px] opacity-55">
          Automatically switch to Dark Mode on gloomy, overcast days.
        </p>
      )}
    </div>
  );
}
