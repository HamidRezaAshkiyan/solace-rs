import { useState } from "react";
import { Compass, MapPin, Search, Sun, Moon, ShieldCheck } from "lucide-react";
import { useSolaceStore } from "../store/useSolaceStore";
import type { CitySearchResult } from "../types/solace";

export function WelcomeView() {
  const { state, searchCities, setLocation, autoDetectLocation, completeOnboarding } =
    useSolaceStore();
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<CitySearchResult[]>([]);
  const [detecting, setDetecting] = useState(false);

  if (!state) return null;
  const isDark = state.currentMode === "dark";

  const handleSearch = async (val: string) => {
    setQuery(val);
    if (val.trim().length < 2) {
      setResults([]);
      return;
    }
    const list = await searchCities(val);
    setResults(list);
  };

  const handleAuto = async () => {
    setDetecting(true);
    try {
      await autoDetectLocation();
    } finally {
      setDetecting(false);
    }
  };

  return (
    <div className="flex flex-col h-full justify-between py-2">
      <div className="space-y-4">
        {/* Hero */}
        <div className="text-center space-y-2 pt-3">
          <div className="mx-auto w-14 h-14 rounded-2xl bg-gradient-to-b from-amber-300 via-orange-400 to-indigo-900 shadow-lg flex items-center justify-center">
            <div className="flex items-center gap-0.5 text-white">
              <Sun className="w-4 h-4" />
              <Moon className="w-4 h-4" />
            </div>
          </div>
          <h1 className="text-[19px] font-bold tracking-tight">Welcome to Solace</h1>
          <p className="text-[12px] opacity-70 leading-relaxed px-2">
            Your display, in tune with the world around you. Solar dark mode, paired wallpapers,
            evening warmth, and weather-aware switching.
          </p>
        </div>

        {/* Location Setup Card */}
        <div
          className={`rounded-2xl p-3.5 border space-y-2.5 ${
            isDark ? "bg-[#1b1d24] border-white/8" : "bg-white border-black/8"
          }`}
        >
          <div className="flex items-center justify-between">
            <span className="text-[12px] font-semibold flex items-center gap-1.5">
              <MapPin className="w-3.5 h-3.5 text-amber-400" />
              Your Location (Optional)
            </span>
            <button
              onClick={() => void handleAuto()}
              disabled={detecting}
              className="px-2 py-0.5 rounded-lg text-[11px] font-medium bg-amber-500/15 text-amber-500 hover:bg-amber-500/25 flex items-center gap-1 cursor-pointer"
            >
              <Compass className="w-3 h-3" />
              {detecting ? "Detecting…" : "Auto-Detect"}
            </button>
          </div>

          {state.config.location ? (
            <div className="px-2.5 py-2 rounded-xl bg-emerald-500/12 border border-emerald-500/25 text-[12px] flex items-center justify-between">
              <span className="font-medium">
                {state.config.location.city}, {state.config.location.country}
              </span>
              <span className="text-[10.5px] text-emerald-400 font-semibold">Ready</span>
            </div>
          ) : (
            <div className="relative">
              <Search className="w-3.5 h-3.5 absolute left-2.5 top-1/2 -translate-y-1/2 opacity-50" />
              <input
                type="text"
                value={query}
                onChange={(e) => void handleSearch(e.target.value)}
                placeholder="Type your city (e.g. London, Tehran)…"
                className={`w-full pl-8 pr-3 py-1.5 rounded-xl text-[12px] border outline-none ${
                  isDark
                    ? "bg-white/6 border-white/10 text-white placeholder:text-zinc-500"
                    : "bg-zinc-50 border-black/10 text-zinc-900 placeholder:text-zinc-400"
                }`}
              />
            </div>
          )}

          {results.length > 0 && (
            <div className="space-y-1 max-h-28 overflow-y-auto solace-scroll">
              {results.map((c, idx) => (
                <button
                  key={`${c.name}-${idx}`}
                  onClick={() => {
                    void setLocation({
                      city: c.name,
                      country: c.country,
                      latitude: c.latitude,
                      longitude: c.longitude,
                    });
                    setResults([]);
                    setQuery("");
                  }}
                  className={`w-full text-left px-2.5 py-1.5 rounded-lg text-[11.5px] cursor-pointer ${
                    isDark ? "hover:bg-white/10" : "hover:bg-black/5"
                  }`}
                >
                  {c.name}
                  {c.admin1 ? `, ${c.admin1}` : ""} · <span className="opacity-65">{c.country}</span>
                </button>
              ))}
            </div>
          )}

          <div className="flex items-center gap-1.5 text-[10.5px] opacity-60">
            <ShieldCheck className="w-3.5 h-3.5 text-emerald-400 shrink-0" />
            <span>Solar times are calculated 100% locally. Zero telemetry.</span>
          </div>
        </div>
      </div>

      {/* CTA */}
      <div className="space-y-2">
        <button
          onClick={() => void completeOnboarding()}
          className="w-full py-2.5 rounded-xl bg-amber-500 hover:bg-amber-400 text-white font-semibold text-[13px] shadow-md transition-colors cursor-pointer"
        >
          Open Solace
        </button>
        <p className="text-center text-[10.5px] opacity-55">
          Solace lives quietly in your menu bar / system tray as a sun or moon icon.
        </p>
      </div>
    </div>
  );
}
