import { useEffect, useState } from "react";
import { Info, Keyboard } from "lucide-react";
import { useSolaceStore } from "../store/useSolaceStore";

function formatShortcutDisplay(shortcut: string, platform: string): string {
  if (!shortcut) return "Set shortcut";
  if (platform === "macos") {
    return shortcut
      .replace(/CommandOrControl|Command|Super/gi, "⌘")
      .replace(/Shift/gi, "⇧")
      .replace(/Alt|Option/gi, "⌥")
      .replace(/Control|Ctrl/gi, "⌃")
      .replace(/\+/g, "");
  }
  return shortcut.replace(/CommandOrControl/gi, "Ctrl");
}

export function PanelFooter() {
  const { state, setView, updateShortcut } = useSolaceStore();
  const [recording, setRecording] = useState(false);

  useEffect(() => {
    if (!recording) return;
    const onKeyDown = (e: KeyboardEvent) => {
      e.preventDefault();
      e.stopPropagation();
      if (e.key === "Escape") {
        setRecording(false);
        return;
      }
      // Ignore bare modifier presses
      if (["Meta", "Control", "Shift", "Alt"].includes(e.key)) return;

      const parts: string[] = [];
      if (e.metaKey) parts.push("Command");
      if (e.ctrlKey) parts.push("Control");
      if (e.altKey) parts.push("Alt");
      if (e.shiftKey) parts.push("Shift");
      if (parts.length === 0) return;

      const keyName =
        e.key.length === 1 ? e.key.toUpperCase() : e.key.replace(/^Arrow/, "");
      parts.push(keyName);
      const combo = parts.join("+");
      setRecording(false);
      void updateShortcut(combo);
    };
    window.addEventListener("keydown", onKeyDown, true);
    return () => window.removeEventListener("keydown", onKeyDown, true);
  }, [recording, updateShortcut]);

  if (!state) return null;
  const isDark = state.currentMode === "dark";

  return (
    <div
      className={`pt-2.5 mt-1 border-t flex items-center justify-between ${
        isDark ? "border-white/8" : "border-black/7"
      }`}
    >
      {/* Global Shortcut Recorder */}
      <button
        onClick={() => setRecording((r) => !r)}
        title="Click to record a new global keyboard shortcut (Esc to cancel)"
        className={`flex items-center gap-1.5 px-2.5 py-1 rounded-lg text-[11px] font-medium transition-colors cursor-pointer ${
          recording
            ? "bg-amber-500/20 text-amber-400 ring-1 ring-amber-400/40"
            : isDark
              ? "bg-white/6 text-zinc-300 hover:bg-white/12"
              : "bg-black/5 text-zinc-700 hover:bg-black/10"
        }`}
      >
        <Keyboard className="w-3.5 h-3.5 opacity-70" />
        <span>
          {recording
            ? "Press keys…"
            : formatShortcutDisplay(state.config.shortcut, state.platform)}
        </span>
      </button>

      {/* Settings / Info Button */}
      <button
        onClick={() => setView("settings")}
        title="Settings & Weather Decisions"
        className={`p-1.5 rounded-lg transition-colors cursor-pointer ${
          isDark
            ? "text-zinc-400 hover:text-white hover:bg-white/10"
            : "text-zinc-500 hover:text-zinc-900 hover:bg-black/6"
        }`}
      >
        <Info className="w-4 h-4" />
      </button>
    </div>
  );
}
