import { useEffect } from "react";
import { useSolaceStore } from "./store/useSolaceStore";
import { ModeCard } from "./components/ModeCard";
import { WallpapersCard } from "./components/WallpapersCard";
import { ScheduleCard } from "./components/ScheduleCard";
import { WarmthCard } from "./components/WarmthCard";
import { WeatherCard } from "./components/WeatherCard";
import { PanelFooter } from "./components/PanelFooter";
import { SettingsView } from "./components/SettingsView";
import { WelcomeView } from "./components/WelcomeView";

export default function App() {
  const { state, view, loading, error, clearError, init, hidePanel, quitApp } =
    useSolaceStore();

  useEffect(() => {
    let cleanup: (() => void) | undefined;
    void init().then((fn) => {
      cleanup = fn;
    });

    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        void hidePanel();
      } else if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "q") {
        e.preventDefault();
        void quitApp();
      }
    };
    window.addEventListener("keydown", onKeyDown);

    return () => {
      cleanup?.();
      window.removeEventListener("keydown", onKeyDown);
    };
  }, [init, hidePanel, quitApp]);

  const isDark = state?.currentMode === "dark";

  return (
    <div
      className={`h-screen w-screen flex flex-col rounded-2xl border overflow-hidden transition-colors duration-300 ${
        isDark
          ? "bg-[#13151b]/96 text-zinc-100 border-white/12 shadow-[0_20px_50px_rgba(0,0,0,0.65)]"
          : "bg-[#f6f5f2]/96 text-zinc-900 border-black/10 shadow-[0_20px_50px_rgba(0,0,0,0.18)]"
      }`}
    >
      {/* Subtle draggable title handle */}
      <div
        data-tauri-drag-region
        className="h-3.5 w-full shrink-0 flex items-center justify-center cursor-grab active:cursor-grabbing"
      >
        <div
          className={`w-8 h-1 rounded-full ${isDark ? "bg-white/15" : "bg-black/12"}`}
        />
      </div>

      {/* Panel Content */}
      <div className="flex-1 px-3.5 pb-3 flex flex-col min-h-0">
        {loading || !state ? (
          <div className="flex-1 flex items-center justify-center text-[12px] opacity-60">
            Loading Solace…
          </div>
        ) : !state.config.onboardingCompleted ? (
          <WelcomeView />
        ) : view === "settings" || view === "decisions" ? (
          <SettingsView />
        ) : (
          <>
            {error && (
              <div className="mb-2 px-2.5 py-1.5 rounded-xl bg-rose-500/15 border border-rose-500/25 text-rose-400 text-[11px] flex items-center justify-between gap-2">
                <span className="truncate">{error}</span>
                <button
                  onClick={clearError}
                  className="font-semibold hover:underline cursor-pointer shrink-0"
                >
                  Dismiss
                </button>
              </div>
            )}

            <div className="flex-1 overflow-y-auto solace-scroll space-y-2.5 pr-0.5">
              <ModeCard />
              <WallpapersCard />
              <ScheduleCard />
              <WarmthCard />
              <WeatherCard />
            </div>

            <PanelFooter />
          </>
        )}
      </div>
    </div>
  );
}
