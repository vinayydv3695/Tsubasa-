// Tsubasa (翼) — Settings Store (v2)
// Zustand store for the grouped settings system.
// Loads all settings on init, allows per-group updates.

import { create } from "zustand";
import type {
    AllSettings,
    BehaviorSettings,
    BitTorrentSettings,
    CloudSettingsV2,
    ConnectionSettings,
    DownloadSettings,
    QueueSettings,
    SeedingSettings,
    SpeedSettings,
} from "@/types";
import {
    getAllSettings,
    updateBehaviorSettings,
    updateBitTorrentSettings,
    updateCloudSettingsV2,
    updateConnectionSettings,
    updateDownloadSettings,
    updateQueueSettings,
    updateSeedingSettings,
    updateSpeedSettings,
} from "@/lib/tauri";

interface SettingsStore {
    // State
    settings: AllSettings | null;
    loading: boolean;
    error: string | null;
    dirty: boolean;

    // Actions
    load: () => Promise<void>;
    updateBehavior: (s: BehaviorSettings) => Promise<void>;
    updateDownloads: (s: DownloadSettings) => Promise<void>;
    updateConnections: (s: ConnectionSettings) => Promise<void>;
    updateSpeed: (s: SpeedSettings) => Promise<void>;
    updateBitTorrent: (s: BitTorrentSettings) => Promise<void>;
    updateQueue: (s: QueueSettings) => Promise<void>;
    updateSeeding: (s: SeedingSettings) => Promise<void>;
    updateCloud: (s: CloudSettingsV2) => Promise<void>;
}

export const useSettingsStore = create<SettingsStore>((set, get) => ({
    settings: null,
    loading: false,
    error: null,
    dirty: false,

    load: async () => {
        set({ loading: true, error: null });
        try {
            const settings = await getAllSettings();
            set({ settings, loading: false });
        } catch (err) {
            const msg = err instanceof Error ? err.message : String(err);
            set({ error: msg, loading: false });
        }
    },

    updateBehavior: async (s) => {
        try {
            await updateBehaviorSettings(s);
            const current = get().settings;
            if (current) set({ settings: { ...current, behavior: s } });
        } catch (err) {
            set({ error: err instanceof Error ? err.message : String(err) });
        }
    },

    updateDownloads: async (s) => {
        try {
            await updateDownloadSettings(s);
            const current = get().settings;
            if (current) set({ settings: { ...current, downloads: s } });
        } catch (err) {
            set({ error: err instanceof Error ? err.message : String(err) });
        }
    },

    updateConnections: async (s) => {
        try {
            await updateConnectionSettings(s);
            const current = get().settings;
            if (current) set({ settings: { ...current, connections: s } });
        } catch (err) {
            set({ error: err instanceof Error ? err.message : String(err) });
        }
    },

    updateSpeed: async (s) => {
        try {
            await updateSpeedSettings(s);
            const current = get().settings;
            if (current) set({ settings: { ...current, speed: s } });
        } catch (err) {
            set({ error: err instanceof Error ? err.message : String(err) });
        }
    },

    updateBitTorrent: async (s) => {
        try {
            await updateBitTorrentSettings(s);
            const current = get().settings;
            if (current) set({ settings: { ...current, bittorrent: s } });
        } catch (err) {
            set({ error: err instanceof Error ? err.message : String(err) });
        }
    },

    updateQueue: async (s) => {
        try {
            await updateQueueSettings(s);
            const current = get().settings;
            if (current) set({ settings: { ...current, queue: s } });
        } catch (err) {
            set({ error: err instanceof Error ? err.message : String(err) });
        }
    },

    updateSeeding: async (s) => {
        try {
            await updateSeedingSettings(s);
            const current = get().settings;
            if (current) set({ settings: { ...current, seeding: s } });
        } catch (err) {
            set({ error: err instanceof Error ? err.message : String(err) });
        }
    },

    updateCloud: async (s) => {
        try {
            await updateCloudSettingsV2(s);
            const current = get().settings;
            if (current) set({ settings: { ...current, cloud: s } });
        } catch (err) {
            set({ error: err instanceof Error ? err.message : String(err) });
        }
    },
}));
