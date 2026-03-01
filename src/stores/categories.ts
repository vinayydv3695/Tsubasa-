// Tsubasa — Categories & Tags Store
// Client-side labels and categories for filtering the torrent list.
// Does NOT require a backend change — stored only in localStorage.

import { create } from "zustand";

export type Category = {
    name: string;
    color: string;  // CSS color token e.g. "var(--green)"
    filter: (name: string) => boolean;
};

// ─── Built-in categories (name heuristics) ──────────────

export const BUILT_IN_CATEGORIES: Category[] = [
    {
        name: "Movies",
        color: "var(--red)",
        filter: (n) => {
            const l = n.toLowerCase();
            return /\b(1080p|720p|2160p|4k|bluray|blu-ray|bdrip|dvdrip|webrip|web-dl|hdtv|x264|x265|hevc|avc|mkv|mp4)\b/.test(l)
                && !/\bs\d{2}e\d{2}\b/.test(l); // not a TV episode
        },
    },
    {
        name: "TV Shows",
        color: "var(--blue)",
        filter: (n) => {
            const l = n.toLowerCase();
            return /\bs\d{2}e\d{2}\b/.test(l) // S01E02 pattern
                || /\bseason\s*\d+\b/i.test(l)
                || /\bepisode\s*\d+\b/i.test(l);
        },
    },
    {
        name: "Music",
        color: "var(--green)",
        filter: (n) => {
            const l = n.toLowerCase();
            return /\b(mp3|flac|aac|ogg|wav|opus|320kbps|lossless|discography|album|soundtrack|ost)\b/.test(l);
        },
    },
    {
        name: "Games",
        color: "var(--amber)",
        filter: (n) => {
            const l = n.toLowerCase();
            return /\b(game|repack|fitgirl|dodi|gog|codex|plaza|skidrow|pc\s?game|switch|ps4|ps5|xbox|steamrip)\b/.test(l);
        },
    },
    {
        name: "Software",
        color: "var(--accent)",
        filter: (n) => {
            const l = n.toLowerCase();
            return /\b(software|app|application|crack|keygen|portable|installer|setup\.exe|winrar|adobe|windows)\b/.test(l);
        },
    },
];

// ─── Store ───────────────────────────────────────────────

interface CategoryStore {
    // User-created tags (simple string labels)
    tags: string[];
    // Per-torrent: map of torrentId → tag name
    torrentTags: Record<string, string | null>;

    addTag: (tag: string) => void;
    removeTag: (tag: string) => void;
    setTorrentTag: (id: string, tag: string | null) => void;
}

const STORAGE_KEY = "tsubasa-tags";

function loadPersistedTags(): { tags: string[]; torrentTags: Record<string, string | null> } {
    try {
        const raw = localStorage.getItem(STORAGE_KEY);
        if (raw) return JSON.parse(raw);
    } catch { /* ignore */ }
    return { tags: [], torrentTags: {} };
}

const initial = loadPersistedTags();

export const useCategoryStore = create<CategoryStore>((set, get) => ({
    tags: initial.tags,
    torrentTags: initial.torrentTags,

    addTag: (tag) => {
        if (get().tags.includes(tag)) return;
        const next = { tags: [...get().tags, tag], torrentTags: get().torrentTags };
        set(next);
        localStorage.setItem(STORAGE_KEY, JSON.stringify(next));
    },

    removeTag: (tag) => {
        const next = { tags: get().tags.filter((t) => t !== tag), torrentTags: get().torrentTags };
        set(next);
        localStorage.setItem(STORAGE_KEY, JSON.stringify(next));
    },

    setTorrentTag: (id, tag) => {
        const next = { tags: get().tags, torrentTags: { ...get().torrentTags, [id]: tag } };
        set(next);
        localStorage.setItem(STORAGE_KEY, JSON.stringify(next));
    },
}));

// ─── Utility ─────────────────────────────────────────────

export function getCategoryForTorrent(name: string): Category | null {
    return BUILT_IN_CATEGORIES.find((c) => c.filter(name)) ?? null;
}
