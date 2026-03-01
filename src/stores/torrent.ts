// Tsubasa (翼) — Torrent Store
// Zustand store for torrent state management.

import { create } from "zustand";
import type { TorrentSummary, TorrentState } from "@/types";
import * as api from "@/lib/tauri";

interface TorrentStore {
  // State
  torrents: Map<string, TorrentSummary>;
  selectedTorrentId: string | null;
  filter: string; // TorrentState | "all" | "cloud" | "cat:X" | "tag:X" | "tracker:X"
  isLoading: boolean;

  // Actions
  loadTorrents: () => Promise<void>;
  addTorrent: (source: string, savePath?: string) => Promise<string>;
  pauseTorrent: (id: string) => Promise<void>;
  resumeTorrent: (id: string) => Promise<void>;
  removeTorrent: (id: string, deleteFiles?: boolean) => Promise<void>;
  selectTorrent: (id: string | null) => void;
  setFilter: (filter: string) => void;

  // Event handlers (called by EventBridge)
  updateProgress: (payload: {
    id: string;
    downloaded_bytes: number;
    total_bytes: number;
    download_speed: number;
    upload_speed: number;
    peers_connected: number;
    seeds_connected: number;
  }) => void;
  updateState: (payload: { id: string; from: string; to: string }) => void;
  handleTorrentAdded: (payload: {
    id: string;
    name: string;
    info_hash: string;
  }) => void;
  handleTorrentRemoved: (payload: { id: string }) => void;
}

export const useTorrentStore = create<TorrentStore>((set, get) => ({
  torrents: new Map(),
  selectedTorrentId: null,
  filter: "all",
  isLoading: false,

  loadTorrents: async () => {
    set({ isLoading: true });
    try {
      const summaries = await api.getTorrents();
      const map = new Map<string, TorrentSummary>();
      for (const t of summaries) {
        map.set(t.id, t);
      }
      set({ torrents: map, isLoading: false });
    } catch (err) {
      console.error("Failed to load torrents:", err);
      set({ isLoading: false });
    }
  },

  addTorrent: async (source, savePath) => {
    const response = await api.addTorrent({ source, save_path: savePath });
    // The backend will emit TorrentAdded event, which updates the store
    return response.id;
  },

  pauseTorrent: async (id) => {
    await api.pauseTorrent(id);
  },

  resumeTorrent: async (id) => {
    await api.resumeTorrent(id);
  },

  removeTorrent: async (id, deleteFiles = false) => {
    await api.removeTorrent(id, deleteFiles);
  },

  selectTorrent: (id) => {
    set({ selectedTorrentId: id });
  },

  setFilter: (filter) => {
    set({ filter });
  },

  updateProgress: (payload) => {
    const { torrents } = get();
    const torrent = torrents.get(payload.id);
    if (!torrent) return;

    const updated = new Map(torrents);
    updated.set(payload.id, {
      ...torrent,
      downloaded_bytes: payload.downloaded_bytes,
      total_bytes: payload.total_bytes,
      download_speed: payload.download_speed,
      upload_speed: payload.upload_speed,
      peers_connected: payload.peers_connected,
      seeds_connected: payload.seeds_connected,
      progress:
        payload.total_bytes > 0
          ? payload.downloaded_bytes / payload.total_bytes
          : 0,
      eta_seconds:
        payload.download_speed > 0
          ? Math.floor(
            (payload.total_bytes - payload.downloaded_bytes) /
            payload.download_speed
          )
          : null,
    });
    set({ torrents: updated });
  },

  updateState: (payload) => {
    const { torrents } = get();
    const torrent = torrents.get(payload.id);
    if (!torrent) return;

    const updated = new Map(torrents);
    updated.set(payload.id, {
      ...torrent,
      state: payload.to as TorrentState,
    });
    set({ torrents: updated });
  },

  handleTorrentAdded: (_payload) => {
    // Reload from backend to get the full record
    get().loadTorrents();
  },

  handleTorrentRemoved: (payload) => {
    const { torrents, selectedTorrentId } = get();
    const updated = new Map(torrents);
    updated.delete(payload.id);
    set({
      torrents: updated,
      selectedTorrentId:
        selectedTorrentId === payload.id ? null : selectedTorrentId,
    });
  },
}));
