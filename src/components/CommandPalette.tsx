// Tsubasa (翼) — Command Palette Component
// Ctrl+K / ⌘+K to open. Quick access to all actions.
// Keyboard navigable, fuzzy-filtered, grouped by category.

import { useState, useEffect, useRef, useCallback, useMemo } from "react";
import {
    Search,
    Download,
    Upload,
    Pause,
    CheckCircle,
    Cloud,
    Settings,
    Moon,
    Sun,
    Palette,
    LayoutList,
    AlertCircle,
    BarChart3,
} from "lucide-react";
import { useTorrentStore } from "@/stores/torrent";
import { useUIStore } from "@/stores/ui";
import { useThemeStore } from "@/stores/theme";
import { useSearchStore } from "@/stores/search";
import "./CommandPalette.css";

interface CommandItem {
    id: string;
    label: string;
    icon: React.ReactNode;
    shortcut?: string;
    group: string;
    action: () => void;
}

interface CommandPaletteProps {
    open: boolean;
    onClose: () => void;
    onOpenSettings: () => void;
}

export function CommandPalette({ open, onClose, onOpenSettings }: CommandPaletteProps) {
    const [query, setQuery] = useState("");
    const [activeIndex, setActiveIndex] = useState(0);
    const inputRef = useRef<HTMLInputElement>(null);
    const listRef = useRef<HTMLDivElement>(null);

    const setFilter = useTorrentStore((s) => s.setFilter);
    const setDetailPanelOpen = useUIStore((s) => s.setDetailPanelOpen);
    const setSearchOpen = useSearchStore((s) => s.setOpen);
    const setTheme = useThemeStore((s) => s.setTheme);

    // All available commands
    const commands: CommandItem[] = useMemo(() => [
        // Navigation
        { id: "all", label: "Show All Torrents", icon: <LayoutList size={14} />, group: "Navigation", action: () => setFilter("all" as any) },
        { id: "downloading", label: "Show Downloading", icon: <Download size={14} />, group: "Navigation", action: () => setFilter("downloading" as any) },
        { id: "seeding", label: "Show Seeding", icon: <Upload size={14} />, group: "Navigation", action: () => setFilter("seeding" as any) },
        { id: "completed", label: "Show Completed", icon: <CheckCircle size={14} />, group: "Navigation", action: () => setFilter("completed" as any) },
        { id: "paused", label: "Show Paused", icon: <Pause size={14} />, group: "Navigation", action: () => setFilter("paused" as any) },
        { id: "errored", label: "Show Errors", icon: <AlertCircle size={14} />, group: "Navigation", action: () => setFilter("errored" as any) },
        { id: "cloud", label: "Show Cloud Torrents", icon: <Cloud size={14} />, group: "Navigation", action: () => setFilter("cloud" as any) },
        // Actions
        { id: "search", label: "Search Torrents", icon: <Search size={14} />, group: "Actions", action: () => setSearchOpen(true) },
        { id: "stats", label: "Open Stats Panel", icon: <BarChart3 size={14} />, group: "Actions", action: () => setDetailPanelOpen(true) },
        { id: "settings", label: "Open Settings", icon: <Settings size={14} />, group: "Actions", action: () => onOpenSettings() },
        // Theme
        { id: "theme-black", label: "Theme: Deep Black", icon: <Moon size={14} />, group: "Theme", action: () => setTheme("black") },
        { id: "theme-rose", label: "Theme: Rosé Pine", icon: <Palette size={14} />, group: "Theme", action: () => setTheme("rose") },
        { id: "theme-light", label: "Theme: Clean White", icon: <Sun size={14} />, group: "Theme", action: () => setTheme("light") },
    ], [setFilter, setDetailPanelOpen, setSearchOpen, setTheme, onOpenSettings]);

    // Filter by query
    const filtered = useMemo(() => {
        if (!query.trim()) return commands;
        const lower = query.toLowerCase();
        return commands.filter(
            (cmd) => cmd.label.toLowerCase().includes(lower) || cmd.group.toLowerCase().includes(lower)
        );
    }, [commands, query]);

    // Group filtered results
    const grouped = useMemo(() => {
        const groups: Record<string, CommandItem[]> = {};
        for (const cmd of filtered) {
            if (!groups[cmd.group]) groups[cmd.group] = [];
            groups[cmd.group].push(cmd);
        }
        return groups;
    }, [filtered]);

    // Reset on open
    useEffect(() => {
        if (open) {
            setQuery("");
            setActiveIndex(0);
            setTimeout(() => inputRef.current?.focus(), 50);
        }
    }, [open]);

    // Reset active index when filtered results change
    useEffect(() => {
        setActiveIndex(0);
    }, [filtered.length]);

    const runCommand = useCallback((cmd: CommandItem) => {
        cmd.action();
        onClose();
    }, [onClose]);

    // Keyboard navigation
    const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
        if (e.key === "ArrowDown") {
            e.preventDefault();
            setActiveIndex((i) => Math.min(i + 1, filtered.length - 1));
        } else if (e.key === "ArrowUp") {
            e.preventDefault();
            setActiveIndex((i) => Math.max(i - 1, 0));
        } else if (e.key === "Enter") {
            e.preventDefault();
            if (filtered[activeIndex]) runCommand(filtered[activeIndex]);
        } else if (e.key === "Escape") {
            e.preventDefault();
            onClose();
        }
    }, [filtered, activeIndex, runCommand, onClose]);

    if (!open) return null;

    return (
        <div className="cmd-palette__overlay" onClick={onClose}>
            <div className="cmd-palette" onClick={(e) => e.stopPropagation()}>
                {/* Search input */}
                <div className="cmd-palette__input-wrap">
                    <Search size={16} color="var(--fg-3)" />
                    <input
                        ref={inputRef}
                        type="text"
                        value={query}
                        onChange={(e) => setQuery(e.target.value)}
                        onKeyDown={handleKeyDown}
                        placeholder="Type a command…"
                        className="cmd-palette__input"
                    />
                    <span className="cmd-palette__kbd">Esc</span>
                </div>

                {/* Results */}
                <div className="cmd-palette__list" ref={listRef}>
                    {filtered.length === 0 ? (
                        <div className="cmd-palette__empty">No matching commands</div>
                    ) : (
                        Object.entries(grouped).map(([group, items]) => (
                            <div key={group}>
                                <div className="cmd-palette__group-label">{group}</div>
                                {items.map((cmd) => {
                                    const flatIndex = filtered.indexOf(cmd);
                                    return (
                                        <button
                                            key={cmd.id}
                                            className="cmd-palette__item"
                                            data-active={flatIndex === activeIndex}
                                            onClick={() => runCommand(cmd)}
                                            onMouseEnter={() => setActiveIndex(flatIndex)}
                                        >
                                            <span className="cmd-palette__item-icon">{cmd.icon}</span>
                                            {cmd.label}
                                            {cmd.shortcut && (
                                                <span className="cmd-palette__item-shortcut">{cmd.shortcut}</span>
                                            )}
                                        </button>
                                    );
                                })}
                            </div>
                        ))
                    )}
                </div>
            </div>
        </div>
    );
}
