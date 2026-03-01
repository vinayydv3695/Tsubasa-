// Tsubasa (翼) — Theme Store

import { create } from "zustand";
import { getSetting, setSetting } from "@/lib/tauri";

export type Theme = "black" | "rose" | "light";

interface ThemeState {
  theme: Theme;
  setTheme: (theme: Theme) => void;
  loadTheme: () => Promise<void>;
}

function applyTheme(theme: Theme) {
  document.documentElement.setAttribute("data-theme", theme);
  localStorage.setItem("tsubasa-theme", theme);
}

export const useThemeStore = create<ThemeState>((set) => ({
  theme: "black",

  setTheme: (theme: Theme) => {
    applyTheme(theme);
    set({ theme });
    setSetting("theme", theme).catch((err) => {
      console.error("Failed to persist theme:", err);
    });
  },

  loadTheme: async () => {
    try {
      const saved = await getSetting("theme");
      if (saved === "black" || saved === "rose" || saved === "light") {
        applyTheme(saved);
        set({ theme: saved });
      } else {
        applyTheme("black");
      }
    } catch {
      applyTheme("black");
    }
  },
}));
