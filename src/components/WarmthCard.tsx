import { useState } from "react";
import { Flame, Settings2 } from "lucide-react";
import { useSolaceStore } from "../store/useSolaceStore";
import type { WarmthSchedule } from "../types/solace";

const FIXED_TIMES: Array<{ hour: number; minute: number; label: string }> = [
  { hour: 15, minute: 0, label: "3:00 PM" },
  { hour: 16, minute: 0, label: "4:00 PM" },
  { hour: 17, minute: 0, label: "5:00 PM" },
  { hour: 18, minute: 0, label: "6:00 PM" },
  { hour: 19, minute: 0, label: "7:00 PM" },
  { hour: 20, minute: 0, label: "8:00 PM" },
  { hour: 21, minute: 0, label: "9:00 PM" },
  { hour: 22, minute: 0, label: "10:00 PM" },
  { hour: 23, minute: 0, label: "11:00 PM" },
  { hour: 0, minute: 0, label: "12:00 AM" },
];

function formatScheduleLabel(s: WarmthSchedule): string {
  if (s.type === "sunset") return "Starts at Sunset";
  if (s.type === "always") return "Always On";
  const suffix = s.hour < 12 ? "AM" : "PM";
  const h12 = s.hour % 12 === 0 ? 12 : s.hour % 12;
  return `Starts at ${h12}:${String(s.minute).padStart(2, "0")} ${suffix}`;
}

export function WarmthCard() {
  const { state, setWarmthConfig, previewWarmth } = useSolaceStore();
  const [schedulePickerOpen, setSchedulePickerOpen] = useState(false);
  const [compatOpen, setCompatOpen] = useState(false);
  const [draggingKelvin, setDraggingKelvin] = useState<number | null>(null);

  if (!state) return null;

  const isDark = state.currentMode === "dark";
  const warmth = state.config.warmth;
  const displayKelvin = draggingKelvin ?? warmth.kelvin;

  const commit = (
    enabled: boolean,
    kelvin: number,
    schedule: WarmthSchedule,
    compat: boolean
  ) => {
    void setWarmthConfig(enabled, kelvin, schedule, compat);
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
        <div className="flex items-center gap-1.5">
          <Flame className="w-3.5 h-3.5 text-orange-400" />
          <span
            className={`text-[12.5px] font-semibold ${
              isDark ? "text-zinc-100" : "text-zinc-800"
            }`}
          >
            Evening Warmth
          </span>
          <button
            onClick={() => setCompatOpen((v) => !v)}
            title="Warmth Compatibility Mode"
            className={`p-1 rounded-md transition-colors cursor-pointer ${
              warmth.compatibilityMode
                ? "text-orange-400 bg-orange-500/15"
                : isDark
                  ? "text-zinc-500 hover:text-zinc-300"
                  : "text-zinc-400 hover:text-zinc-700"
            }`}
          >
            <Settings2 className="w-3 h-3" />
          </button>
        </div>

        <div className="flex items-center gap-1.5">
          <button
            onClick={() => setSchedulePickerOpen((v) => !v)}
            className={`px-2 py-0.5 rounded-full text-[10.5px] font-medium cursor-pointer transition-colors ${
              isDark
                ? "bg-white/8 text-zinc-300 hover:bg-white/14"
                : "bg-black/5 text-zinc-700 hover:bg-black/10"
            }`}
          >
            {formatScheduleLabel(warmth.schedule)}
          </button>

          <button
            onClick={() =>
              commit(!warmth.enabled, warmth.kelvin, warmth.schedule, warmth.compatibilityMode)
            }
            className={`px-2.5 py-0.5 rounded-full text-[11px] font-medium transition-colors cursor-pointer ${
              warmth.enabled
                ? "bg-orange-500/20 text-orange-400 ring-1 ring-orange-400/30"
                : isDark
                  ? "bg-white/8 text-zinc-400 hover:bg-white/12"
                  : "bg-black/5 text-zinc-600 hover:bg-black/10"
            }`}
          >
            {warmth.enabled ? "On" : "Off"}
          </button>
        </div>
      </div>

      {/* Compatibility Mode Popover */}
      {compatOpen && (
        <div
          className={`mb-2.5 p-2 rounded-xl border flex items-center justify-between gap-2 text-[11px] ${
            isDark ? "bg-black/25 border-white/10" : "bg-zinc-50 border-black/8"
          }`}
        >
          <div>
            <div className="font-semibold">Compatibility Mode</div>
            <div className="text-[10px] opacity-65">
              Alternative colour path for XDR / HDR displays
            </div>
          </div>
          <button
            onClick={() =>
              commit(warmth.enabled, warmth.kelvin, warmth.schedule, !warmth.compatibilityMode)
            }
            className={`px-2 py-0.5 rounded-full text-[10.5px] font-semibold cursor-pointer ${
              warmth.compatibilityMode
                ? "bg-orange-500 text-white"
                : isDark
                  ? "bg-white/10 text-zinc-300"
                  : "bg-black/10 text-zinc-700"
            }`}
          >
            {warmth.compatibilityMode ? "Enabled" : "Off"}
          </button>
        </div>
      )}

      {/* Schedule Picker */}
      {schedulePickerOpen && (
        <div
          className={`mb-2.5 p-2 rounded-xl border space-y-1.5 ${
            isDark ? "bg-black/25 border-white/10" : "bg-zinc-50 border-black/8"
          }`}
        >
          <div className="grid grid-cols-2 gap-1.5">
            <button
              onClick={() => {
                commit(true, warmth.kelvin, { type: "sunset" }, warmth.compatibilityMode);
                setSchedulePickerOpen(false);
              }}
              className={`py-1 rounded-lg text-[11px] font-medium cursor-pointer ${
                warmth.schedule.type === "sunset"
                  ? "bg-orange-500 text-white"
                  : isDark
                    ? "bg-white/8 text-zinc-200 hover:bg-white/12"
                    : "bg-white border border-black/8 text-zinc-700"
              }`}
            >
              At Sunset
            </button>
            <button
              onClick={() => {
                commit(true, warmth.kelvin, { type: "always" }, warmth.compatibilityMode);
                setSchedulePickerOpen(false);
              }}
              className={`py-1 rounded-lg text-[11px] font-medium cursor-pointer ${
                warmth.schedule.type === "always"
                  ? "bg-orange-500 text-white"
                  : isDark
                    ? "bg-white/8 text-zinc-200 hover:bg-white/12"
                    : "bg-white border border-black/8 text-zinc-700"
              }`}
            >
              Always On
            </button>
          </div>
          <div className="flex flex-wrap gap-1 pt-1">
            {FIXED_TIMES.map((ft) => {
              const active =
                warmth.schedule.type === "fixed_time" &&
                warmth.schedule.hour === ft.hour &&
                warmth.schedule.minute === ft.minute;
              return (
                <button
                  key={ft.label}
                  onClick={() => {
                    commit(
                      true,
                      warmth.kelvin,
                      { type: "fixed_time", hour: ft.hour, minute: ft.minute },
                      warmth.compatibilityMode
                    );
                    setSchedulePickerOpen(false);
                  }}
                  className={`px-1.5 py-0.5 rounded-md text-[10px] cursor-pointer ${
                    active
                      ? "bg-orange-500 text-white font-semibold"
                      : isDark
                        ? "bg-white/6 text-zinc-300 hover:bg-white/12"
                        : "bg-white border border-black/8 text-zinc-700"
                  }`}
                >
                  {ft.label}
                </button>
              );
            })}
          </div>
        </div>
      )}

      {/* Kelvin Slider */}
      <div className="space-y-1.5">
        <input
          type="range"
          min={1900}
          max={6500}
          step={100}
          value={displayKelvin}
          onChange={(e) => {
            const val = Number(e.target.value);
            setDraggingKelvin(val);
            void previewWarmth(val);
          }}
          onPointerUp={() => {
            if (draggingKelvin !== null) {
              const val = draggingKelvin;
              setDraggingKelvin(null);
              commit(true, val, warmth.schedule, warmth.compatibilityMode);
            }
          }}
          className="warmth-slider"
        />
        <div className="flex items-center justify-between text-[10.5px]">
          <span className="text-orange-400 font-medium">Warm</span>
          <span
            className={`font-semibold tabular-nums ${
              isDark ? "text-zinc-200" : "text-zinc-800"
            }`}
          >
            {displayKelvin}K
            {state.warmthActive && warmth.enabled ? " · Active" : ""}
          </span>
          <span className="text-sky-400 font-medium">Cool</span>
        </div>
      </div>
    </div>
  );
}
