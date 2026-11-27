import { Sun, Moon, AlertTriangle, Sparkles } from "lucide-react";
import { useSolaceStore } from "../store/useSolaceStore";

export function ModeCard() {
  const { state, toggleMode, respondFirstSwitch, keepLightToday } = useSolaceStore();
  if (!state) return null;

  const isDark = state.currentMode === "dark";

  return (
    <div className="space-y-2">
      {/* Main Hero Mode Card */}
      <button
        onClick={() => void toggleMode()}
        className={`group w-full text-left rounded-2xl p-3.5 border transition-all duration-300 cursor-pointer relative overflow-hidden ${
          isDark
            ? "bg-gradient-to-br from-[#1f2233] via-[#1b1d2b] to-[#171923] border-white/10 hover:border-indigo-400/35 shadow-[0_8px_24px_rgba(0,0,0,0.35)]"
            : "bg-gradient-to-br from-[#fffdf7] via-[#fff9eb] to-[#fef3d6] border-amber-500/20 hover:border-amber-500/40 shadow-[0_4px_16px_rgba(245,158,11,0.10)]"
        }`}
      >
        <div className="flex items-center justify-between gap-3">
          <div className="flex items-center gap-3 min-w-0">
            <div
              className={`w-11 h-11 rounded-xl flex items-center justify-center shrink-0 transition-transform duration-300 group-hover:scale-105 ${
                isDark
                  ? "bg-indigo-500/20 text-indigo-300 ring-1 ring-indigo-400/30"
                  : "bg-amber-400/25 text-amber-600 ring-1 ring-amber-500/30"
              }`}
            >
              {isDark ? <Moon className="w-5 h-5" /> : <Sun className="w-5 h-5" />}
            </div>
            <div className="min-w-0">
              <div className="flex items-center gap-2">
                <h2
                  className={`text-[15px] font-semibold tracking-tight ${
                    isDark ? "text-white" : "text-zinc-900"
                  }`}
                >
                  {isDark ? "Dark Mode" : "Light Mode"}
                </h2>
                {state.config.manualOverride && (
                  <span
                    className={`text-[10px] font-medium px-1.5 py-0.5 rounded-full ${
                      isDark
                        ? "bg-white/10 text-zinc-300"
                        : "bg-black/6 text-zinc-600"
                    }`}
                  >
                    Manual
                  </span>
                )}
              </div>
              <p
                className={`text-[11.5px] truncate mt-0.5 ${
                  isDark ? "text-zinc-400" : "text-zinc-600"
                }`}
              >
                {state.modeSubtitle}
              </p>
            </div>
          </div>

          <div
            className={`text-[11px] font-medium px-2.5 py-1 rounded-lg shrink-0 transition-colors ${
              isDark
                ? "bg-white/8 text-zinc-300 group-hover:bg-white/14"
                : "bg-black/5 text-zinc-700 group-hover:bg-black/10"
            }`}
          >
            Switch
          </div>
        </div>
      </button>

      {/* First-Switch Respectful Prompt */}
      {state.firstSwitchPrompt && (
        <div
          className={`rounded-xl p-3 border flex items-center justify-between gap-2 ${
            isDark
              ? "bg-indigo-500/12 border-indigo-400/25 text-indigo-100"
              : "bg-amber-50 border-amber-300/60 text-amber-950"
          }`}
        >
          <div className="flex items-center gap-2 min-w-0">
            <Sparkles className="w-4 h-4 shrink-0 opacity-80" />
            <div className="text-[11.5px] leading-tight">
              <div className="font-semibold">
                Switch to{" "}
                {state.firstSwitchPrompt.targetMode === "dark" ? "Dark Mode" : "Light Mode"}?
              </div>
              <div className="opacity-75 text-[10.5px]">
                {state.firstSwitchPrompt.reason}
              </div>
            </div>
          </div>
          <div className="flex items-center gap-1.5 shrink-0">
            <button
              onClick={() => void respondFirstSwitch(false)}
              className={`px-2 py-1 rounded-lg text-[11px] font-medium cursor-pointer ${
                isDark
                  ? "bg-white/8 hover:bg-white/14 text-zinc-200"
                  : "bg-black/6 hover:bg-black/10 text-zinc-700"
              }`}
            >
              Keep Current
            </button>
            <button
              onClick={() => void respondFirstSwitch(true)}
              className="px-2.5 py-1 rounded-lg text-[11px] font-semibold bg-amber-500 hover:bg-amber-400 text-white cursor-pointer"
            >
              Switch
            </button>
          </div>
        </div>
      )}

      {/* Weather-Triggered Dark Mode Keep Light Banner */}
      {state.weatherTriggeredDark && isDark && (
        <div className="rounded-xl px-3 py-2 border border-sky-400/25 bg-sky-500/10 flex items-center justify-between gap-2">
          <span className="text-[11.5px] text-sky-200">
            Gloomy sky switched to Dark Mode today
          </span>
          <button
            onClick={() => void keepLightToday()}
            className="px-2.5 py-1 rounded-lg text-[11px] font-semibold bg-sky-500/25 hover:bg-sky-500/35 text-sky-100 cursor-pointer shrink-0"
          >
            Keep Light
          </button>
        </div>
      )}

      {/* OS Permission Warning */}
      {state.permissionWarning && (
        <div className="rounded-xl px-3 py-2 border border-amber-500/30 bg-amber-500/12 flex items-start gap-2 text-[11px] text-amber-500">
          <AlertTriangle className="w-3.5 h-3.5 shrink-0 mt-0.5" />
          <span className="leading-snug">{state.permissionWarning}</span>
        </div>
      )}
    </div>
  );
}
