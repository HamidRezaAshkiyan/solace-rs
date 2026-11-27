import { useState, useRef } from "react";
import { Sun, Moon, MapPin, Compass, Search, X } from "lucide-react";
import { useSolaceStore } from "../store/useSolaceStore";
import type { CitySearchResult } from "../types/solace";

const OFFSET_PRESETS = [-120, -90, -60, -45, -30, -15, 0, 15, 30, 45, 60, 90, 120];

function formatOffset(min: number): string {
  if (min === 0) return "Exact";
  const sign = min < 0 ? "Earlier" : "Later";
  const abs = Math.abs(min);
  const h = Math.floor(abs / 60);
  const m = abs % 60;
  const dur = h > 0 ? (m > 0 ? `${h}h ${m}m` : `${h}h`) : `${m}m`;
  return `${dur} ${sign}`;
}

export function ScheduleCard() {
  const {
    state,
    setScheduleEnabled,
    setScheduleOffsets,
    setLocation,
    searchCities,
    autoDetectLocation,
  } = useSolaceStore();

  const [editingMarker, setEditingMarker] = useState<"sunrise" | "sunset" | null>(null);
  const [locationMenuOpen, setLocationMenuOpen] = useState(false);
  const [cityQuery, setCityQuery] = useState("");
  const [cityResults, setCityResults] = useState<CitySearchResult[]>([]);
  const [searching, setSearching] = useState(false);
  const barRef = useRef<HTMLDivElement>(null);

  if (!state) return null;

  const isDark = state.currentMode === "dark";
  const sched = state.config.schedule;
  const solar = state.solar;
  const loc = state.config.location;

  const handleCitySearch = async (q: string) => {
    setCityQuery(q);
    if (q.trim().length < 2) {
      setCityResults([]);
      return;
    }
    setSearching(true);
    try {
      const res = await searchCities(q);
      setCityResults(res);
    } finally {
      setSearching(false);
    }
  };

  const handleDragMarker = (marker: "sunrise" | "sunset", clientX: number) => {
    if (!barRef.current || !solar) return;
    const rect = barRef.current.getBoundingClientRect();
    const ratio = Math.max(0, Math.min(1, (clientX - rect.left) / rect.width));
    const targetMin = Math.round(ratio * 1440);
    const baseMin = marker === "sunrise" ? solar.sunriseMin : solar.sunsetMin;
    let delta = Math.max(-120, Math.min(120, targetMin - baseMin));
    // Snap to 15-minute increments
    delta = Math.round(delta / 15) * 15;
    if (marker === "sunrise") {
      void setScheduleOffsets(delta, sched.sunsetOffsetMin);
    } else {
      void setScheduleOffsets(sched.sunriseOffsetMin, delta);
    }
  };

  const srPct = solar ? (solar.effectiveSunriseMin / 1440) * 100 : 28;
  const ssPct = solar ? (solar.effectiveSunsetMin / 1440) * 100 : 75;
  const nowPct = solar ? (solar.currentMinute / 1440) * 100 : 50;

  return (
    <div
      className={`rounded-2xl p-3.5 border transition-colors ${
        isDark
          ? "bg-[#1b1d24] border-white/8"
          : "bg-white border-black/7 shadow-[0_1px_3px_rgba(0,0,0,0.03)]"
      }`}
    >
      {/* Header */}
      <div className="flex items-center justify-between mb-2.5">
        <div className="flex items-center gap-2">
          <Sun className={`w-3.5 h-3.5 ${isDark ? "text-amber-300" : "text-amber-500"}`} />
          <span
            className={`text-[12.5px] font-semibold ${
              isDark ? "text-zinc-100" : "text-zinc-800"
            }`}
          >
            Schedule
          </span>
        </div>

        {!sched.enabled ? (
          <button
            onClick={() => void setScheduleEnabled(true)}
            className={`px-2.5 py-0.5 rounded-full text-[11px] font-medium cursor-pointer ${
              isDark
                ? "bg-white/8 text-zinc-400 hover:bg-white/12"
                : "bg-black/5 text-zinc-600 hover:bg-black/10"
            }`}
          >
            Enable
          </button>
        ) : (
          <button
            onClick={() => setLocationMenuOpen((v) => !v)}
            className={`flex items-center gap-1 px-2 py-0.5 rounded-full text-[11px] font-medium cursor-pointer transition-colors ${
              isDark
                ? "bg-white/8 text-zinc-300 hover:bg-white/14"
                : "bg-black/5 text-zinc-700 hover:bg-black/10"
            }`}
          >
            <MapPin className="w-3 h-3 opacity-70" />
            <span className="max-w-[120px] truncate">{loc ? loc.city : "Set City"}</span>
          </button>
        )}
      </div>

      {/* Location Picker Popover */}
      {(locationMenuOpen || (sched.enabled && !loc)) && (
        <div
          className={`mb-3 p-2.5 rounded-xl border space-y-2 ${
            isDark ? "bg-black/25 border-white/10" : "bg-zinc-50 border-black/8"
          }`}
        >
          <div className="flex items-center gap-1.5">
            <div className="relative flex-1">
              <Search className="w-3.5 h-3.5 absolute left-2 top-1/2 -translate-y-1/2 opacity-50" />
              <input
                type="text"
                value={cityQuery}
                onChange={(e) => void handleCitySearch(e.target.value)}
                placeholder="Search any city…"
                className={`w-full pl-7 pr-2 py-1 text-[11.5px] rounded-lg border outline-none ${
                  isDark
                    ? "bg-white/6 border-white/10 text-white placeholder:text-zinc-500"
                    : "bg-white border-black/10 text-zinc-900 placeholder:text-zinc-400"
                }`}
              />
            </div>
            <button
              onClick={() => void autoDetectLocation().then(() => setLocationMenuOpen(false))}
              title="Auto-detect approximate city"
              className={`px-2 py-1 rounded-lg text-[11px] font-medium flex items-center gap-1 cursor-pointer ${
                isDark
                  ? "bg-indigo-500/20 text-indigo-300 hover:bg-indigo-500/30"
                  : "bg-amber-500/15 text-amber-700 hover:bg-amber-500/25"
              }`}
            >
              <Compass className="w-3 h-3" />
              Auto
            </button>
            {loc && (
              <button
                onClick={() => setLocationMenuOpen(false)}
                className="p-1 opacity-60 hover:opacity-100 cursor-pointer"
              >
                <X className="w-3.5 h-3.5" />
              </button>
            )}
          </div>

          {searching && <div className="text-[11px] opacity-60 px-1">Searching…</div>}

          {cityResults.length > 0 && (
            <div className="space-y-1 max-h-28 overflow-y-auto solace-scroll">
              {cityResults.map((c, idx) => (
                <button
                  key={`${c.name}-${idx}`}
                  onClick={() => {
                    void setLocation({
                      city: c.name,
                      country: c.country,
                      latitude: c.latitude,
                      longitude: c.longitude,
                    });
                    setCityResults([]);
                    setCityQuery("");
                    setLocationMenuOpen(false);
                  }}
                  className={`w-full text-left px-2 py-1 rounded-lg text-[11.5px] cursor-pointer ${
                    isDark ? "hover:bg-white/10 text-zinc-200" : "hover:bg-black/6 text-zinc-800"
                  }`}
                >
                  {c.name}
                  {c.admin1 ? `, ${c.admin1}` : ""} ·{" "}
                  <span className="opacity-65">{c.country}</span>
                </button>
              ))}
            </div>
          )}

          {sched.enabled && loc && (
            <div className="pt-1 border-t border-current/10 flex justify-end">
              <button
                onClick={() => {
                  void setScheduleEnabled(false);
                  setLocationMenuOpen(false);
                }}
                className="text-[11px] text-rose-400 hover:text-rose-300 font-medium cursor-pointer"
              >
                Turn Schedule Off
              </button>
            </div>
          )}
        </div>
      )}

      {/* 24-Hour Solar Timeline */}
      {sched.enabled && solar ? (
        <div>
          <div className="relative pt-2 pb-1">
            <div
              ref={barRef}
              className="relative h-3 rounded-full overflow-hidden border border-black/10"
              style={{
                background: `linear-gradient(90deg, #1e2238 0%, #2a2f52 ${Math.max(
                  0,
                  srPct - 4
                )}%, #fbbf24 ${srPct}%, #fde68a ${(srPct + ssPct) / 2}%, #fb923c ${ssPct}%, #2a2f52 ${Math.min(
                  100,
                  ssPct + 4
                )}%, #1e2238 100%)`,
              }}
            >
              {/* Current time needle */}
              <div
                className="absolute top-0 bottom-0 w-0.5 bg-white shadow-[0_0_6px_rgba(255,255,255,0.9)]"
                style={{ left: `${nowPct}%` }}
              />
            </div>

            {/* Draggable Sunrise Marker */}
            <div
              onPointerDown={(e) => {
                const move = (ev: PointerEvent) => handleDragMarker("sunrise", ev.clientX);
                const up = () => {
                  window.removeEventListener("pointermove", move);
                  window.removeEventListener("pointerup", up);
                };
                window.addEventListener("pointermove", move);
                window.addEventListener("pointerup", up);
                handleDragMarker("sunrise", e.clientX);
              }}
              title="Drag to offset sunrise (±2h)"
              style={{ left: `${srPct}%` }}
              className="absolute top-1 -translate-x-1/2 w-5 h-5 rounded-full bg-amber-400 text-amber-950 shadow-md ring-2 ring-white flex items-center justify-center cursor-ew-resize"
            >
              <Sun className="w-3 h-3" />
            </div>

            {/* Draggable Sunset Marker */}
            <div
              onPointerDown={(e) => {
                const move = (ev: PointerEvent) => handleDragMarker("sunset", ev.clientX);
                const up = () => {
                  window.removeEventListener("pointermove", move);
                  window.removeEventListener("pointerup", up);
                };
                window.addEventListener("pointermove", move);
                window.addEventListener("pointerup", up);
                handleDragMarker("sunset", e.clientX);
              }}
              title="Drag to offset sunset (±2h)"
              style={{ left: `${ssPct}%` }}
              className="absolute top-1 -translate-x-1/2 w-5 h-5 rounded-full bg-indigo-500 text-white shadow-md ring-2 ring-white flex items-center justify-center cursor-ew-resize"
            >
              <Moon className="w-3 h-3" />
            </div>
          </div>

          {/* Clickable Sunrise / Sunset Time Pills */}
          <div className="flex items-center justify-between mt-2 text-[11px]">
            <button
              onClick={() =>
                setEditingMarker(editingMarker === "sunrise" ? null : "sunrise")
              }
              className={`px-2 py-1 rounded-lg transition-colors cursor-pointer ${
                editingMarker === "sunrise"
                  ? "bg-amber-500/20 text-amber-500 font-semibold"
                  : isDark
                    ? "text-zinc-300 hover:bg-white/8"
                    : "text-zinc-700 hover:bg-black/5"
              }`}
            >
              <span className="font-medium">{solar.effectiveSunriseFormatted}</span>
              {sched.sunriseOffsetMin !== 0 && (
                <span className="ml-1 text-[10px] opacity-70">
                  ({formatOffset(sched.sunriseOffsetMin)})
                </span>
              )}
            </button>

            <span className="text-[10px] opacity-45">12:00 PM</span>

            <button
              onClick={() =>
                setEditingMarker(editingMarker === "sunset" ? null : "sunset")
              }
              className={`px-2 py-1 rounded-lg transition-colors cursor-pointer ${
                editingMarker === "sunset"
                  ? "bg-indigo-500/20 text-indigo-400 font-semibold"
                  : isDark
                    ? "text-zinc-300 hover:bg-white/8"
                    : "text-zinc-700 hover:bg-black/5"
              }`}
            >
              <span className="font-medium">{solar.effectiveSunsetFormatted}</span>
              {sched.sunsetOffsetMin !== 0 && (
                <span className="ml-1 text-[10px] opacity-70">
                  ({formatOffset(sched.sunsetOffsetMin)})
                </span>
              )}
            </button>
          </div>

          {/* Inline Offset Picker */}
          {editingMarker && (
            <div
              className={`mt-2 p-2 rounded-xl border ${
                isDark ? "bg-black/25 border-white/10" : "bg-zinc-50 border-black/8"
              }`}
            >
              <div className="flex items-center justify-between mb-1.5 text-[10.5px] opacity-75">
                <span>
                  {editingMarker === "sunrise" ? "Sunrise offset" : "Sunset offset"}
                </span>
                <button
                  onClick={() => setEditingMarker(null)}
                  className="font-medium hover:underline cursor-pointer"
                >
                  Done
                </button>
              </div>
              <div className="flex flex-wrap gap-1">
                {OFFSET_PRESETS.map((val) => {
                  const active =
                    (editingMarker === "sunrise"
                      ? sched.sunriseOffsetMin
                      : sched.sunsetOffsetMin) === val;
                  return (
                    <button
                      key={val}
                      onClick={() => {
                        if (editingMarker === "sunrise") {
                          void setScheduleOffsets(val, sched.sunsetOffsetMin);
                        } else {
                          void setScheduleOffsets(sched.sunriseOffsetMin, val);
                        }
                      }}
                      className={`px-1.5 py-0.5 rounded-md text-[10px] cursor-pointer transition-colors ${
                        active
                          ? "bg-amber-500 text-white font-semibold"
                          : isDark
                            ? "bg-white/6 text-zinc-300 hover:bg-white/12"
                            : "bg-white border border-black/8 text-zinc-700 hover:bg-black/5"
                      }`}
                    >
                      {val === 0
                        ? "Exact"
                        : `${val > 0 ? "+" : ""}${val}m`}
                    </button>
                  );
                })}
              </div>
            </div>
          )}
        </div>
      ) : (
        <p className={`text-[11px] ${isDark ? "text-zinc-500" : "text-zinc-500"}`}>
          {sched.enabled
            ? "Choose a city to calculate local sunrise and sunset offline."
            : "Automatic solar switching is paused."}
        </p>
      )}
    </div>
  );
}
