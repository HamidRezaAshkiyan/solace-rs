import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type {
  CitySearchResult,
  LocationConfig,
  SolaceStateSnapshot,
  WarmthSchedule,
  WeatherSensitivity,
} from "../types/solace";

export type PanelView = "main" | "settings" | "decisions";

interface SolaceStore {
  state: SolaceStateSnapshot | null;
  view: PanelView;
  loading: boolean;
  error: string | null;

  init: () => Promise<() => void>;
  setView: (view: PanelView) => void;
  clearError: () => void;

  toggleMode: () => Promise<void>;
  respondFirstSwitch: (accept: boolean) => Promise<void>;
  keepLightToday: () => Promise<void>;

  setWallpapersEnabled: (enabled: boolean) => Promise<void>;
  setWallpaperPath: (slot: "light" | "dark", path: string) => Promise<void>;

  setScheduleEnabled: (enabled: boolean) => Promise<void>;
  setScheduleOffsets: (sunriseOffsetMin: number, sunsetOffsetMin: number) => Promise<void>;
  setLocation: (location: LocationConfig) => Promise<void>;
  searchCities: (query: string) => Promise<CitySearchResult[]>;
  autoDetectLocation: () => Promise<void>;

  setWarmthConfig: (
    enabled: boolean,
    kelvin: number,
    schedule: WarmthSchedule,
    compatibilityMode: boolean
  ) => Promise<void>;
  previewWarmth: (kelvin: number | null) => Promise<void>;

  setWeatherEnabled: (enabled: boolean) => Promise<void>;
  setWeatherSensitivity: (sensitivity: WeatherSensitivity) => Promise<void>;
  toggleTemperatureUnit: () => Promise<void>;
  refreshWeather: () => Promise<void>;

  updateShortcut: (shortcut: string) => Promise<void>;
  completeOnboarding: () => Promise<void>;
  hidePanel: () => Promise<void>;
  quitApp: () => Promise<void>;
}

export const useSolaceStore = create<SolaceStore>((set) => ({
  state: null,
  view: "main",
  loading: true,
  error: null,

  init: async () => {
    try {
      const snap = await invoke<SolaceStateSnapshot>("get_state");
      set({ state: snap, loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }

    const unlistenState = await listen<SolaceStateSnapshot>(
      "solace://state-changed",
      (event) => {
        set({ state: event.payload });
      }
    );

    const unlistenNav = await listen<string>("solace://navigate", (event) => {
      if (event.payload === "settings") set({ view: "settings" });
      else if (event.payload === "main") set({ view: "main" });
    });

    return () => {
      unlistenState();
      unlistenNav();
    };
  },

  setView: (view) => set({ view }),
  clearError: () => set({ error: null }),

  toggleMode: async () => {
    const snap = await invoke<SolaceStateSnapshot>("toggle_mode");
    set({ state: snap });
  },

  respondFirstSwitch: async (accept) => {
    const snap = await invoke<SolaceStateSnapshot>("respond_first_switch", { accept });
    set({ state: snap });
  },

  keepLightToday: async () => {
    const snap = await invoke<SolaceStateSnapshot>("keep_light_today");
    set({ state: snap });
  },

  setWallpapersEnabled: async (enabled) => {
    const snap = await invoke<SolaceStateSnapshot>("set_wallpapers_enabled", { enabled });
    set({ state: snap });
  },

  setWallpaperPath: async (slot, path) => {
    const snap = await invoke<SolaceStateSnapshot>("set_wallpaper_path", { slot, path });
    set({ state: snap });
  },

  setScheduleEnabled: async (enabled) => {
    const snap = await invoke<SolaceStateSnapshot>("set_schedule_enabled", { enabled });
    set({ state: snap });
  },

  setScheduleOffsets: async (sunriseOffsetMin, sunsetOffsetMin) => {
    const snap = await invoke<SolaceStateSnapshot>("set_schedule_offsets", {
      sunriseOffsetMin,
      sunsetOffsetMin,
    });
    set({ state: snap });
  },

  setLocation: async (location) => {
    try {
      const snap = await invoke<SolaceStateSnapshot>("set_location", { location });
      set({ state: snap, error: null });
    } catch (e) {
      set({ error: String(e) });
    }
  },

  searchCities: async (query) => {
    return invoke<CitySearchResult[]>("search_cities", { query });
  },

  autoDetectLocation: async () => {
    try {
      const snap = await invoke<SolaceStateSnapshot>("auto_detect_location");
      set({ state: snap, error: null });
    } catch (e) {
      set({ error: String(e) });
    }
  },

  setWarmthConfig: async (enabled, kelvin, schedule, compatibilityMode) => {
    const snap = await invoke<SolaceStateSnapshot>("set_warmth_config", {
      enabled,
      kelvin,
      schedule,
      compatibilityMode,
    });
    set({ state: snap });
  },

  previewWarmth: async (kelvin) => {
    const snap = await invoke<SolaceStateSnapshot>("preview_warmth", { kelvin });
    set({ state: snap });
  },

  setWeatherEnabled: async (enabled) => {
    try {
      const snap = await invoke<SolaceStateSnapshot>("set_weather_enabled", { enabled });
      set({ state: snap, error: null });
    } catch (e) {
      set({ error: String(e) });
    }
  },

  setWeatherSensitivity: async (sensitivity) => {
    const snap = await invoke<SolaceStateSnapshot>("set_weather_sensitivity", { sensitivity });
    set({ state: snap });
  },

  toggleTemperatureUnit: async () => {
    const snap = await invoke<SolaceStateSnapshot>("toggle_temperature_unit");
    set({ state: snap });
  },

  refreshWeather: async () => {
    try {
      const snap = await invoke<SolaceStateSnapshot>("refresh_weather");
      set({ state: snap, error: null });
    } catch (e) {
      set({ error: String(e) });
    }
  },

  updateShortcut: async (shortcut) => {
    try {
      const snap = await invoke<SolaceStateSnapshot>("update_shortcut", { shortcut });
      set({ state: snap, error: null });
    } catch (e) {
      set({ error: String(e) });
    }
  },

  completeOnboarding: async () => {
    const snap = await invoke<SolaceStateSnapshot>("complete_onboarding");
    set({ state: snap });
  },

  hidePanel: async () => {
    await invoke("hide_panel");
  },

  quitApp: async () => {
    await invoke("quit_app");
  },
}));
