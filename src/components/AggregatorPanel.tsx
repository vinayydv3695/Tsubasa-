// Tsubasa (翼) — Search Aggregator Panel
// Multi-plugin torrent search with category filtering and source badges.
// Separate from the existing SearchPanel (which uses Torbox API).

import React, { useState, useEffect } from "react";
import {
    Search,
    Loader2,
    X,
    Download,
    ExternalLink,
    Filter,
    ChevronDown,
} from "lucide-react";
import { useAggregatorStore } from "@/stores/aggregator";
import type { PluginSearchResult } from "@/types";
import { motion, AnimatePresence } from "framer-motion";

function formatBytes(bytes: number): string {
    if (bytes === 0) return "0 B";
    const k = 1024;
    const units = ["B", "KB", "MB", "GB", "TB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${(bytes / Math.pow(k, i)).toFixed(i > 1 ? 1 : 0)} ${units[i]}`;
}

const CATEGORIES = [
    { value: null, label: "All" },
    { value: "movies", label: "Movies" },
    { value: "tv", label: "TV" },
    { value: "music", label: "Music" },
    { value: "games", label: "Games" },
    { value: "software", label: "Software" },
    { value: "anime", label: "Anime" },
    { value: "books", label: "Books" },
];

const SOURCE_COLORS: Record<string, { bg: string; fg: string }> = {
    piratebay: { bg: "rgba(99, 102, 241, 0.12)", fg: "#818cf8" },
    yts: { bg: "rgba(34, 197, 94, 0.12)", fg: "#22c55e" },
    leet: { bg: "rgba(245, 158, 11, 0.12)", fg: "#f59e0b" },
    nyaa: { bg: "rgba(236, 72, 153, 0.12)", fg: "#ec4899" },
    torrentgalaxy: { bg: "rgba(59, 130, 246, 0.12)", fg: "#3b82f6" },
};

const SOURCE_NAMES: Record<string, string> = {
    piratebay: "TPB",
    yts: "YTS",
    leet: "1337x",
    nyaa: "Nyaa",
    torrentgalaxy: "TGx",
};

interface AggregatorPanelProps {
    onAddTorrent: (magnet: string) => void;
    onClose?: () => void;
}

export function AggregatorPanel({ onAddTorrent, onClose }: AggregatorPanelProps) {
    const store = useAggregatorStore();
    const [showFilters, setShowFilters] = useState(false);

    useEffect(() => {
        if (!store.pluginsLoaded) store.loadPlugins();
    }, []);

    const handleSearch = (e: React.FormEvent) => {
        e.preventDefault();
        store.search();
    };

    return (
        <div style={{
            display: "flex", flexDirection: "column", height: "100%",
            background: "var(--surface)", borderRadius: 12,
            border: "1px solid var(--line-strong)",
            overflow: "hidden",
        }}>
            {/* Header */}
            <div style={{
                display: "flex", alignItems: "center", justifyContent: "space-between",
                padding: "12px 16px", borderBottom: "1px solid var(--line)",
            }}>
                <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
                    <Search size={14} color="var(--accent)" />
                    <span style={{ fontSize: 13, fontWeight: 600, color: "var(--fg)", letterSpacing: "-0.2px" }}>
                        Search Aggregator
                    </span>
                    <span style={{
                        padding: "2px 6px", borderRadius: 4, fontSize: 9, fontWeight: 600,
                        background: "var(--accent-soft)", color: "var(--accent)",
                    }}>
                        {store.plugins.length} plugins
                    </span>
                </div>
                {onClose && (
                    <button onClick={onClose} style={{
                        width: 28, height: 28, borderRadius: 6, border: "none",
                        background: "transparent", cursor: "pointer", color: "var(--fg-3)",
                        display: "flex", alignItems: "center", justifyContent: "center",
                    }}>
                        <X size={14} />
                    </button>
                )}
            </div>

            {/* Search bar */}
            <form onSubmit={handleSearch} style={{ padding: "12px 16px", borderBottom: "1px solid var(--line-subtle)" }}>
                <div style={{ display: "flex", gap: 8 }}>
                    <div style={{
                        flex: 1, display: "flex", alignItems: "center", gap: 8,
                        padding: "8px 12px", borderRadius: 8,
                        border: "1px solid var(--line)", background: "var(--overlay)",
                    }}>
                        <Search size={13} color="var(--fg-3)" />
                        <input
                            type="text"
                            value={store.query}
                            onChange={(e) => store.setQuery(e.target.value)}
                            placeholder="Search across all plugins…"
                            style={{
                                flex: 1, border: "none", background: "transparent",
                                color: "var(--fg)", fontSize: 12, outline: "none",
                            }}
                        />
                        {store.query && (
                            <button
                                type="button"
                                onClick={() => { store.setQuery(""); store.clearResults(); }}
                                style={{ border: "none", background: "transparent", cursor: "pointer", color: "var(--fg-3)", padding: 0, display: "flex" }}
                            >
                                <X size={12} />
                            </button>
                        )}
                    </div>

                    <button
                        type="button"
                        onClick={() => setShowFilters(!showFilters)}
                        style={{
                            padding: "8px 10px", borderRadius: 8,
                            border: "1px solid", borderColor: showFilters ? "var(--accent)" : "var(--line)",
                            background: showFilters ? "var(--accent-soft)" : "var(--overlay)",
                            color: showFilters ? "var(--accent)" : "var(--fg-2)",
                            cursor: "pointer", display: "flex", alignItems: "center", gap: 4, fontSize: 11,
                        }}
                    >
                        <Filter size={12} />
                        <ChevronDown size={10} style={{ transform: showFilters ? "rotate(180deg)" : "none", transition: "transform 150ms" }} />
                    </button>

                    <button
                        type="submit"
                        disabled={store.loading || !store.query.trim()}
                        style={{
                            padding: "8px 16px", borderRadius: 8, border: "none",
                            background: "var(--gradient-accent)", color: "#fff",
                            fontSize: 12, fontWeight: 500, cursor: "pointer",
                            opacity: store.loading || !store.query.trim() ? 0.5 : 1,
                            display: "flex", alignItems: "center", gap: 6,
                        }}
                    >
                        {store.loading ? <Loader2 size={12} style={{ animation: "spin 1s linear infinite" }} /> : <Search size={12} />}
                        Search
                    </button>
                </div>

                {/* Category filters */}
                <AnimatePresence>
                    {showFilters && (
                        <motion.div
                            initial={{ height: 0, opacity: 0 }}
                            animate={{ height: "auto", opacity: 1 }}
                            exit={{ height: 0, opacity: 0 }}
                            transition={{ duration: 0.15 }}
                            style={{ overflow: "hidden" }}
                        >
                            <div style={{ display: "flex", flexWrap: "wrap", gap: 6, paddingTop: 10 }}>
                                {CATEGORIES.map(({ value, label }) => (
                                    <button
                                        key={label}
                                        type="button"
                                        onClick={() => store.setCategory(value)}
                                        style={{
                                            padding: "4px 10px", borderRadius: 6, fontSize: 10, fontWeight: 500,
                                            border: "1px solid",
                                            borderColor: store.category === value ? "var(--accent)" : "var(--line)",
                                            background: store.category === value ? "var(--accent-soft)" : "transparent",
                                            color: store.category === value ? "var(--accent)" : "var(--fg-2)",
                                            cursor: "pointer", transition: "all 150ms ease",
                                        }}
                                    >
                                        {label}
                                    </button>
                                ))}
                            </div>
                        </motion.div>
                    )}
                </AnimatePresence>
            </form>

            {/* Results */}
            <div style={{ flex: 1, overflowY: "auto", padding: "0" }}>
                {store.error && (
                    <div style={{ margin: 16, padding: "10px 14px", borderRadius: 8, background: "var(--red-soft)", color: "var(--red)", fontSize: 11, border: "1px solid rgba(239,68,68,0.2)" }}>
                        {store.error}
                    </div>
                )}

                {store.results.length > 0 && (
                    <div style={{ padding: "8px 16px 4px", fontSize: 10, color: "var(--fg-3)" }}>
                        {store.results.length} results
                    </div>
                )}

                {store.results.map((result, i) => (
                    <ResultRow key={`${result.source}-${result.info_hash ?? i}`} result={result} onAdd={onAddTorrent} />
                ))}

                {!store.loading && store.results.length === 0 && store.query && !store.error && (
                    <div style={{ display: "flex", flexDirection: "column", alignItems: "center", justifyContent: "center", height: 120, color: "var(--fg-3)", fontSize: 12 }}>
                        No results found
                    </div>
                )}

                {!store.query && store.results.length === 0 && !store.loading && (
                    <div style={{ display: "flex", flexDirection: "column", alignItems: "center", justifyContent: "center", height: 120, color: "var(--fg-3)", fontSize: 12, gap: 6 }}>
                        <Search size={20} strokeWidth={1.5} />
                        Search across {store.plugins.length} torrent sites
                    </div>
                )}
            </div>
        </div>
    );
}

function ResultRow({ result, onAdd }: { result: PluginSearchResult; onAdd: (magnet: string) => void }) {
    const colors = SOURCE_COLORS[result.source] ?? { bg: "var(--muted)", fg: "var(--fg-2)" };
    const sourceName = SOURCE_NAMES[result.source] ?? result.source;

    return (
        <div
            style={{
                display: "flex", alignItems: "center", gap: 12,
                padding: "10px 16px", borderBottom: "1px solid var(--line-subtle)",
                transition: "background 100ms ease", cursor: "default",
            }}
            onMouseEnter={(e) => { e.currentTarget.style.background = "var(--overlay)"; }}
            onMouseLeave={(e) => { e.currentTarget.style.background = "transparent"; }}
        >
            {/* Source badge */}
            <span style={{
                padding: "3px 6px", borderRadius: 4, fontSize: 9, fontWeight: 700,
                background: colors.bg, color: colors.fg, flexShrink: 0, minWidth: 32, textAlign: "center",
                letterSpacing: "0.3px",
            }}>
                {sourceName}
            </span>

            {/* Title + meta */}
            <div style={{ flex: 1, minWidth: 0 }}>
                <div style={{ fontSize: 12, color: "var(--fg)", fontWeight: 500, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                    {result.title}
                </div>
                <div style={{ display: "flex", gap: 10, marginTop: 3, fontSize: 10, color: "var(--fg-3)" }}>
                    <span>{formatBytes(result.size_bytes)}</span>
                    {result.category && <span>{result.category}</span>}
                    {result.upload_date && <span>{result.upload_date}</span>}
                </div>
            </div>

            {/* Seeds / Leeches */}
            <div style={{ display: "flex", gap: 8, flexShrink: 0 }}>
                <span style={{ fontSize: 11, color: "var(--green)", fontFamily: "'JetBrains Mono', monospace", fontWeight: 600 }}>
                    ↑{result.seeders}
                </span>
                <span style={{ fontSize: 11, color: "var(--red)", fontFamily: "'JetBrains Mono', monospace" }}>
                    ↓{result.leechers}
                </span>
            </div>

            {/* Actions */}
            <div style={{ display: "flex", gap: 4, flexShrink: 0 }}>
                {result.magnet && (
                    <button
                        onClick={() => onAdd(result.magnet!)}
                        title="Add torrent"
                        style={{
                            width: 28, height: 28, borderRadius: 6, border: "1px solid var(--line)",
                            background: "var(--overlay)", cursor: "pointer", color: "var(--accent)",
                            display: "flex", alignItems: "center", justifyContent: "center",
                            transition: "all 150ms ease",
                        }}
                        onMouseEnter={(e) => { e.currentTarget.style.background = "var(--accent-soft)"; e.currentTarget.style.borderColor = "var(--accent)"; }}
                        onMouseLeave={(e) => { e.currentTarget.style.background = "var(--overlay)"; e.currentTarget.style.borderColor = "var(--line)"; }}
                    >
                        <Download size={12} />
                    </button>
                )}
                {result.source_url && (
                    <a
                        href={result.source_url}
                        target="_blank"
                        rel="noopener noreferrer"
                        title="Open source"
                        style={{
                            width: 28, height: 28, borderRadius: 6, border: "1px solid var(--line)",
                            background: "var(--overlay)", cursor: "pointer", color: "var(--fg-3)",
                            display: "flex", alignItems: "center", justifyContent: "center",
                            textDecoration: "none", transition: "all 150ms ease",
                        }}
                        onMouseEnter={(e) => { e.currentTarget.style.background = "var(--muted)"; e.currentTarget.style.color = "var(--fg)"; }}
                        onMouseLeave={(e) => { e.currentTarget.style.background = "var(--overlay)"; e.currentTarget.style.color = "var(--fg-3)"; }}
                    >
                        <ExternalLink size={12} />
                    </a>
                )}
            </div>
        </div>
    );
}
