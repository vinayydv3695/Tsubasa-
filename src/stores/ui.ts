// Tsubasa (翼) — UI Store
// Global UI state (panels, theme, etc.)

import { create } from "zustand";

interface UIStore {
  // State
  sidebarCollapsed: boolean;
  detailPanelOpen: boolean;
  detailPanelTab: string;
  engineReady: boolean;
  globalDownloadSpeed: number;
  globalUploadSpeed: number;
  activeTorrents: number;
  totalPeers: number;

  // Actions
  toggleSidebar: () => void;
  setDetailPanelOpen: (open: boolean) => void;
  setDetailPanelTab: (tab: string) => void;
  setEngineReady: (ready: boolean) => void;
  updateGlobalStats: (stats: {
    total_download_speed: number;
    total_upload_speed: number;
    active_torrents: number;
    total_peers: number;
  }) => void;
}

export const useUIStore = create<UIStore>((set) => ({
  sidebarCollapsed: false,
  detailPanelOpen: false,
  detailPanelTab: "general",
  engineReady: false,
  globalDownloadSpeed: 0,
  globalUploadSpeed: 0,
  activeTorrents: 0,
  totalPeers: 0,

  toggleSidebar: () =>
    set((state) => ({ sidebarCollapsed: !state.sidebarCollapsed })),

  setDetailPanelOpen: (open) => set({ detailPanelOpen: open }),
  setDetailPanelTab: (tab) => set({ detailPanelTab: tab }),
  setEngineReady: (ready) => set({ engineReady: ready }),

  updateGlobalStats: (stats) =>
    set({
      globalDownloadSpeed: stats.total_download_speed,
      globalUploadSpeed: stats.total_upload_speed,
      activeTorrents: stats.active_torrents,
      totalPeers: stats.total_peers,
    }),
}));
