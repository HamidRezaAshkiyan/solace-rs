import { useEffect, useState } from "react";
import {
  ArrowLeft,
  CheckCircle2,
  Cloud,
  ExternalLink,
  Power,
  ShieldCheck,
} from "lucide-react";
import { enable, disable, isEnabled } from "@tauri-apps/plugin-autostart";
import { openUrl } from "@tauri-apps/plugin-opener";
import { useSolaceStore } from "../store/useSolaceStore";

export function SettingsView() {
  const {
    state,
    view,
    setView,
    toggleTemperatureUnit,
    setWarmthConfig,
    quitApp,
  } = useSolaceStore();
  const [autostart, setAutostart] = useState(false);

  useEffect(() => {
    void isEnabled()
      .then(setAutostart)
      .catch(() => {});
  }, []);

  if (!state) return null;
  const isDark = state.currentMode === "dark";

  const toggleAutostart = async () => {
    try {
      if (autostart) {
        await disable();
        setAutostart(false);
      } else {
        await enable();
        setAutostart(true);
      }
    } catch {
      // ignore in dev mode if unsupported
    }
  };

  if (view === "decisions") {
    return (
      <div className="flex flex-col h-full">
        <div className="flex items-center gap-2 mb-3">
          <button
            onClick={() => setView("settings")}
            className={`p-1.5 rounded-lg cursor-pointer ${
              isDark ? "hover:bg-white/10 text-zinc-300" : "hover:bg-black/6 text-zinc-700"
            }`}
          >
            <ArrowLeft className="w-4 h-4" />
          </button>
          <h2 className="text-[14px] font-semibold">Weather Decisions</h2>
        </div>

        <p className="text-[11px] opacity-65 mb-3">
          The last 10 daily weather evaluations. Solace makes one morning decision per day so your
          screen never flickers.
        </p>

        <div className="flex-1 overflow-y-auto solace-scroll space-y-2 pr-0.5">
          {state.config.weatherDecisions.length === 0 ? (
            <div
              className={`rounded-xl p-4 text-center text-[11.5px] opacity-65 border ${
                isDark ? "bg-[#1b1d24] border-white/8" : "bg-white border-black/8"
              }`}
            >
              No weather decisions recorded yet. Enable Weather Mode to start logging.
            </div>
          ) : (
            state.config.weatherDecisions.map((rec, i) => (
              <div
                key={`${rec.date}-${i}`}
                className={`rounded-xl p-2.5 border text-[11px] space-y-1 ${
                  isDark ? "bg-[#1b1d24] border-white/8" : "bg-white border-black/8"
                }`}
              >
                <div className="flex items-center justify-between font-semibold">
                  <span>
                    {rec.date} · {rec.decidedAt}
                  </span>
                  <span
                    className={`px-2 py-0.5 rounded-full text-[10px] ${
                      rec.outcomeMode === "dark"
                        ? "bg-indigo-500/20 text-indigo-300"
                        : "bg-amber-500/20 text-amber-500"
                    }`}
                  >
                    {rec.outcomeMode === "dark" ? "Dark Mode" : "Light Mode"}
                  </span>
                </div>
                <div className="flex items-center justify-between opacity-75">
                  <span>
                    Cloud: {Math.round(rec.avgCloudPercent)}% vs {Math.round(rec.thresholdPercent)}%
                    ({rec.sensitivity})
                  </span>
                </div>
                <div className="text-[10px] opacity-55">{rec.trigger}</div>
              </div>
            ))
          )}
        </div>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="flex items-center gap-2 mb-3">
        <button
          onClick={() => setView("main")}
          className={`p-1.5 rounded-lg cursor-pointer ${
            isDark ? "hover:bg-white/10 text-zinc-300" : "hover:bg-black/6 text-zinc-700"
          }`}
        >
          <ArrowLeft className="w-4 h-4" />
        </button>
        <h2 className="text-[14px] font-semibold">Settings</h2>
      </div>

      <div className="flex-1 overflow-y-auto solace-scroll space-y-2.5 pr-0.5">
        {/* General Card */}
        <div
          className={`rounded-2xl p-3.5 border space-y-3 ${
            isDark ? "bg-[#1b1d24] border-white/8" : "bg-white border-black/7"
          }`}
        >
          <div className="text-[11px] font-semibold uppercase tracking-wider opacity-55">
            General
          </div>

          <div className="flex items-center justify-between text-[12px]">
            <span>Launch at Login</span>
            <button
              onClick={() => void toggleAutostart()}
              className={`px-2.5 py-0.5 rounded-full text-[11px] font-medium cursor-pointer ${
                autostart
                  ? "bg-emerald-500/20 text-emerald-400"
                  : isDark
                    ? "bg-white/8 text-zinc-400"
                    : "bg-black/6 text-zinc-600"
              }`}
            >
              {autostart ? "Enabled" : "Off"}
            </button>
          </div>

          <div className="flex items-center justify-between text-[12px]">
            <span>Temperature Unit</span>
            <button
              onClick={() => void toggleTemperatureUnit()}
              className={`px-2.5 py-0.5 rounded-full text-[11px] font-medium cursor-pointer ${
                isDark ? "bg-white/10 text-zinc-200" : "bg-black/6 text-zinc-800"
              }`}
            >
              {state.config.weather.unit === "celsius" ? "Celsius (°C)" : "Fahrenheit (°F)"}
            </button>
          </div>

          <div className="flex items-center justify-between text-[12px]">
            <div>
              <div>Warmth Compatibility Mode</div>
              <div className="text-[10px] opacity-60">For HDR / XDR displays</div>
            </div>
            <button
              onClick={() =>
                void setWarmthConfig(
                  state.config.warmth.enabled,
                  state.config.warmth.kelvin,
                  state.config.warmth.schedule,
                  !state.config.warmth.compatibilityMode
                )
              }
              className={`px-2.5 py-0.5 rounded-full text-[11px] font-medium cursor-pointer ${
                state.config.warmth.compatibilityMode
                  ? "bg-orange-500/20 text-orange-400"
                  : isDark
                    ? "bg-white/8 text-zinc-400"
                    : "bg-black/6 text-zinc-600"
              }`}
            >
              {state.config.warmth.compatibilityMode ? "On" : "Off"}
            </button>
          </div>
        </div>

        {/* Support & Diagnostics */}
        <div
          className={`rounded-2xl p-3.5 border space-y-2.5 ${
            isDark ? "bg-[#1b1d24] border-white/8" : "bg-white border-black/7"
          }`}
        >
          <div className="text-[11px] font-semibold uppercase tracking-wider opacity-55">
            Support
          </div>

          <button
            onClick={() => setView("decisions")}
            className={`w-full flex items-center justify-between text-[12px] py-1 px-2 -mx-2 rounded-lg cursor-pointer ${
              isDark ? "hover:bg-white/8" : "hover:bg-black/5"
            }`}
          >
            <span className="flex items-center gap-2">
              <Cloud className="w-3.5 h-3.5 text-sky-400" />
              Weather Decisions
            </span>
            <span className="text-[11px] opacity-60">
              {state.config.weatherDecisions.length} logged →
            </span>
          </button>

          <button
            onClick={() => void openUrl("https://github.com/HamidRezaAshkiyan/solace-rs")}
            className={`w-full flex items-center justify-between text-[12px] py-1 px-2 -mx-2 rounded-lg cursor-pointer ${
              isDark ? "hover:bg-white/8" : "hover:bg-black/5"
            }`}
          >
            <span className="flex items-center gap-2">
              <ExternalLink className="w-3.5 h-3.5 opacity-70" />
              GitHub Repository
            </span>
            <span className="text-[11px] opacity-60">solace-rs</span>
          </button>
        </div>

        {/* Privacy & About */}
        <div
          className={`rounded-2xl p-3.5 border space-y-2 ${
            isDark ? "bg-[#1b1d24] border-white/8" : "bg-white border-black/7"
          }`}
        >
          <div className="flex items-center gap-2 text-[12px] font-semibold">
            <ShieldCheck className="w-4 h-4 text-emerald-400" />
            <span>Private by design</span>
          </div>
          <p className="text-[11px] opacity-65 leading-relaxed">
            No account, zero telemetry, and zero analytics. Solar sunrise and sunset are calculated
            100% locally on your device.
          </p>
          <div className="flex items-center justify-between pt-1 text-[11px] opacity-60">
            <span className="flex items-center gap-1 text-emerald-400 font-medium">
              <CheckCircle2 className="w-3.5 h-3.5" />
              Open Source (MIT)
            </span>
            <span>v{state.version}</span>
          </div>
        </div>
      </div>

      {/* Quit Button */}
      <div className="pt-2.5 mt-2 border-t border-current/10">
        <button
          onClick={() => void quitApp()}
          className="w-full py-2 rounded-xl text-[12px] font-semibold flex items-center justify-center gap-1.5 bg-rose-500/15 hover:bg-rose-500/25 text-rose-400 transition-colors cursor-pointer"
        >
          <Power className="w-3.5 h-3.5" />
          Quit Solace
        </button>
      </div>
    </div>
  );
}
