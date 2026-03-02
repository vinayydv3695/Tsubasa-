// Tsubasa (翼) — Search Aggregator Store
// Zustand store for plugin-based torrent search (no API key required).

import { create } from "zustand";
import type { PluginSearchResult, SearchPluginInfo } from "@/types";
import { aggregatorSearch, getSearchPlugins } from "@/lib/tauri";

interface AggregatorStore {
    // State
    query: string;
    results: PluginSearchResult[];
    plugins: SearchPluginInfo[];
    enabledPlugins: string[];
    category: string | null;
    loading: boolean;
    error: string | null;
    pluginsLoaded: boolean;

    // Actions
    setQuery: (q: string) => void;
    setCategory: (c: string | null) => void;
    togglePlugin: (id: string) => void;
    search: (queryOverride?: string) => Promise<void>;
    loadPlugins: () => Promise<void>;
    clearResults: () => void;
}

export const useAggregatorStore = create<AggregatorStore>((set, get) => ({
    query: "",
    results: [],
    plugins: [],
    enabledPlugins: [],
    category: null,
    loading: false,
    error: null,
    pluginsLoaded: false,

    setQuery: (query) => set({ query }),
    setCategory: (category) => set({ category }),

    togglePlugin: (id) => {
        const current = get().enabledPlugins;
        if (current.includes(id)) {
            set({ enabledPlugins: current.filter((p) => p !== id) });
        } else {
            set({ enabledPlugins: [...current, id] });
        }
    },

    search: async (queryOverride) => {
        const q = (queryOverride ?? get().query).trim();
        if (!q) return;

        set({ loading: true, error: null, query: q });

        try {
            const results = await aggregatorSearch(
                q,
                get().category ?? undefined,
                get().enabledPlugins.length > 0 ? get().enabledPlugins : undefined,
            );
            set({ results, loading: false });
        } catch (err) {
            const msg = err instanceof Error ? err.message : String(err);
            set({ error: msg, loading: false, results: [] });
        }
    },

    loadPlugins: async () => {
        try {
            const plugins = await getSearchPlugins();
            set({
                plugins,
                pluginsLoaded: true,
                // Enable all by default
                enabledPlugins: plugins.map((p) => p.id),
            });
        } catch {
            set({ plugins: [], pluginsLoaded: true });
        }
    },

    clearResults: () => set({ results: [], error: null }),
}));
