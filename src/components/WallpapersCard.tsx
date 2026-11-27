import { Image as ImageIcon, Sun, Moon, AlertCircle } from "lucide-react";
import { open } from "@tauri-apps/plugin-dialog";
import { convertFileSrc } from "@tauri-apps/api/core";
import { useSolaceStore } from "../store/useSolaceStore";

function basename(path: string | null): string {
  if (!path) return "Choose image…";
  const parts = path.split(/[\\/]/);
  return parts[parts.length - 1] || path;
}

export function WallpapersCard() {
  const { state, setWallpapersEnabled, setWallpaperPath } = useSolaceStore();
  if (!state) return null;

  const isDark = state.currentMode === "dark";
  const wp = state.config.wallpapers;

  const pickWallpaper = async (slot: "light" | "dark") => {
    const selected = await open({
      multiple: false,
      title: slot === "light" ? "Choose Light Mode Wallpaper" : "Choose Dark Mode Wallpaper",
      filters: [
        {
          name: "Images",
          extensions: ["png", "jpg", "jpeg", "webp", "heic", "bmp", "tiff"],
        },
      ],
    });
    if (typeof selected === "string" && selected.length > 0) {
      await setWallpaperPath(slot, selected);
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
      <div className="flex items-center justify-between mb-2.5">
        <div className="flex items-center gap-2">
          <ImageIcon
            className={`w-3.5 h-3.5 ${isDark ? "text-zinc-400" : "text-zinc-500"}`}
          />
          <span
            className={`text-[12.5px] font-semibold ${
              isDark ? "text-zinc-100" : "text-zinc-800"
            }`}
          >
            Wallpapers
          </span>
        </div>

        <button
          onClick={() => void setWallpapersEnabled(!wp.enabled)}
          className={`px-2.5 py-0.5 rounded-full text-[11px] font-medium transition-colors cursor-pointer ${
            wp.enabled
              ? isDark
                ? "bg-emerald-500/20 text-emerald-300 ring-1 ring-emerald-400/30"
                : "bg-emerald-500/15 text-emerald-700 ring-1 ring-emerald-500/25"
              : isDark
                ? "bg-white/8 text-zinc-400 hover:bg-white/12"
                : "bg-black/5 text-zinc-600 hover:bg-black/10"
          }`}
        >
          {wp.enabled ? "Active" : "Enable"}
        </button>
      </div>

      <div className="grid grid-cols-2 gap-2.5">
        {/* Light Wallpaper Slot */}
        <button
          onClick={() => void pickWallpaper("light")}
          className={`group relative aspect-[16/10] rounded-xl overflow-hidden border text-left transition-all cursor-pointer ${
            state.currentMode === "light" && wp.enabled
              ? "ring-2 ring-amber-400 border-transparent"
              : isDark
                ? "border-white/10 hover:border-white/25"
                : "border-black/10 hover:border-black/25"
          }`}
        >
          {wp.lightPath && !state.lightWallpaperMissing ? (
            <img
              src={convertFileSrc(wp.lightPath)}
              alt="Light wallpaper"
              className="w-full h-full object-cover transition-transform duration-300 group-hover:scale-105"
            />
          ) : (
            <div className="w-full h-full bg-gradient-to-br from-amber-100 via-sky-100 to-blue-200 flex items-center justify-center">
              <Sun className="w-5 h-5 text-amber-500/70" />
            </div>
          )}
          <div className="absolute inset-x-0 bottom-0 bg-gradient-to-t from-black/75 via-black/40 to-transparent px-2 py-1.5 flex items-center justify-between">
            <span className="text-[10.5px] font-medium text-white flex items-center gap-1 truncate">
              <Sun className="w-3 h-3 text-amber-300 shrink-0" />
              Light
            </span>
            <span className="text-[9.5px] text-white/75 truncate max-w-[78px]">
              {basename(wp.lightPath)}
            </span>
          </div>
        </button>

        {/* Dark Wallpaper Slot */}
        <button
          onClick={() => void pickWallpaper("dark")}
          className={`group relative aspect-[16/10] rounded-xl overflow-hidden border text-left transition-all cursor-pointer ${
            state.currentMode === "dark" && wp.enabled
              ? "ring-2 ring-indigo-400 border-transparent"
              : isDark
                ? "border-white/10 hover:border-white/25"
                : "border-black/10 hover:border-black/25"
          }`}
        >
          {wp.darkPath && !state.darkWallpaperMissing ? (
            <img
              src={convertFileSrc(wp.darkPath)}
              alt="Dark wallpaper"
              className="w-full h-full object-cover transition-transform duration-300 group-hover:scale-105"
            />
          ) : (
            <div className="w-full h-full bg-gradient-to-br from-slate-800 via-indigo-950 to-zinc-900 flex items-center justify-center">
              <Moon className="w-5 h-5 text-indigo-300/70" />
            </div>
          )}
          <div className="absolute inset-x-0 bottom-0 bg-gradient-to-t from-black/75 via-black/40 to-transparent px-2 py-1.5 flex items-center justify-between">
            <span className="text-[10.5px] font-medium text-white flex items-center gap-1 truncate">
              <Moon className="w-3 h-3 text-indigo-300 shrink-0" />
              Dark
            </span>
            <span className="text-[9.5px] text-white/75 truncate max-w-[78px]">
              {basename(wp.darkPath)}
            </span>
          </div>
        </button>
      </div>

      {(state.lightWallpaperMissing || state.darkWallpaperMissing) && (
        <div className="mt-2 flex items-center gap-1.5 text-[11px] text-amber-500">
          <AlertCircle className="w-3.5 h-3.5 shrink-0" />
          <span>Wallpaper not found — click the slot above to re-pick.</span>
        </div>
      )}
    </div>
  );
}
