// Tsubasa (翼) — Search Store
// State management for torrent search and search history.

import { create } from "zustand";
import type { SearchHistoryEntry, SearchResult } from "@/types";
import {
  searchTorrents,
  saveSearchHistory,
  getSearchHistory,
  clearSearchHistory,
} from "@/lib/tauri";

interface SearchStore {
  // State
  query: string;
  results: SearchResult[];
  loading: boolean;
  error: string | null;
  history: SearchHistoryEntry[];
  historyLoaded: boolean;
  checkCache: boolean;
  isOpen: boolean;

  // Actions
  setQuery: (query: string) => void;
  setCheckCache: (checkCache: boolean) => void;
  setOpen: (open: boolean) => void;
  search: (query?: string) => Promise<void>;
  loadHistory: () => Promise<void>;
  clearHistory: () => Promise<void>;
  clearResults: () => void;
}

export const useSearchStore = create<SearchStore>((set, get) => ({
  query: "",
  results: [],
  loading: false,
  error: null,
  history: [],
  historyLoaded: false,
  checkCache: false,
  isOpen: false,

  setQuery: (query) => set({ query }),
  setCheckCache: (checkCache) => set({ checkCache }),
  setOpen: (open) => set({ isOpen: open }),

  search: async (queryOverride?: string) => {
    const q = queryOverride ?? get().query;
    const trimmed = q.trim();
    if (!trimmed) return;

    set({ loading: true, error: null, query: trimmed });

    try {
      const results = await searchTorrents(trimmed, get().checkCache);
      set({ results, loading: false });

      // Save to history (fire and forget)
      saveSearchHistory(trimmed).catch(() => {});

      // Refresh history in background
      getSearchHistory()
        .then((history) => set({ history, historyLoaded: true }))
        .catch(() => {});
    } catch (err) {
      const message =
        err instanceof Error ? err.message : typeof err === "string" ? err : "Search failed";
      set({ error: message, loading: false, results: [] });
    }
  },

  loadHistory: async () => {
    try {
      const history = await getSearchHistory();
      set({ history, historyLoaded: true });
    } catch {
      set({ history: [], historyLoaded: true });
    }
  },

  clearHistory: async () => {
    try {
      await clearSearchHistory();
      set({ history: [] });
    } catch {
      // Silently fail
    }
  },

  clearResults: () => set({ results: [], error: null }),
}));
