// Tsubasa — Toolbar Component
// Add torrent (magnet or file), pause/resume all, search, settings access.

import React, { useState, useCallback } from "react";
import {
  Plus,
  ArrowDown,
  ArrowUp,
  PauseCircle,
  PlayCircle,
  Settings,
  FileUp,
  Search,
  X,
} from "lucide-react";
import { open as dialogOpen } from "@tauri-apps/plugin-dialog";
import { useTorrentStore } from "@/stores/torrent";
import { useUIStore } from "@/stores/ui";
import { useSearchStore } from "@/stores/search";
import { formatSpeed } from "@/lib/utils";
import { SettingsPanel } from "@/components/SettingsPanel";
import { SearchPanel } from "@/components/SearchPanel";

export function Toolbar() {
  const [showAddDialog, setShowAddDialog] = useState(false);
  const [showSettings, setShowSettings] = useState(false);
  const [magnetInput, setMagnetInput] = useState("");
  const addTorrent = useTorrentStore((s) => s.addTorrent);
  const torrents = useTorrentStore((s) => s.torrents);
  const pauseTorrent = useTorrentStore((s) => s.pauseTorrent);
  const resumeTorrent = useTorrentStore((s) => s.resumeTorrent);
  const globalDownloadSpeed = useUIStore((s) => s.globalDownloadSpeed);
  const globalUploadSpeed = useUIStore((s) => s.globalUploadSpeed);

  const searchOpen = useSearchStore((s) => s.isOpen);
  const setSearchOpen = useSearchStore((s) => s.setOpen);

  const handleAddMagnet = async () => {
    const source = magnetInput.trim();
    if (!source) return;
    try {
      await addTorrent(source);
      setMagnetInput("");
      setShowAddDialog(false);
    } catch (err) {
      console.error("Failed to add torrent:", err);
    }
  };

  const handleAddFile = useCallback(async () => {
    try {
      const selected = await dialogOpen({
        multiple: true,
        filters: [{ name: "Torrent Files", extensions: ["torrent"] }],
      });
      if (!selected) return;
      const paths = Array.isArray(selected) ? selected : [selected];
      for (const path of paths) {
        try { await addTorrent(path); }
        catch (err) { console.error(`Failed to add torrent from file: ${path}`, err); }
      }
    } catch (err) {
      console.error("Failed to open file dialog:", err);
    }
  }, [addTorrent]);

  const handlePauseAll = useCallback(async () => {
    const active = Array.from(torrents.values()).filter(
      (t) => t.state === "downloading" || t.state === "seeding"
    );
    for (const t of active) {
      try { await pauseTorrent(t.id); }
      catch (err) { console.error(`Failed to pause ${t.id}:`, err); }
    }
  }, [torrents, pauseTorrent]);

  const handleResumeAll = useCallback(async () => {
    const paused = Array.from(torrents.values()).filter(
      (t) => t.state === "paused" || t.state === "stopped"
    );
    for (const t of paused) {
      try { await resumeTorrent(t.id); }
      catch (err) { console.error(`Failed to resume ${t.id}:`, err); }
    }
  }, [torrents, resumeTorrent]);

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter") handleAddMagnet();
    if (e.key === "Escape") { setShowAddDialog(false); setMagnetInput(""); }
  };

  const hasActive = Array.from(torrents.values()).some(
    (t) => t.state === "downloading" || t.state === "seeding"
  );
  const hasPaused = Array.from(torrents.values()).some(
    (t) => t.state === "paused" || t.state === "stopped"
  );

  return (
    <>
      <div
        data-tauri-drag-region
        style={{
          height: 44,
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
          padding: "0 12px",
          borderBottom: "1px solid var(--line)",
          background: "var(--surface)",
          flexShrink: 0,
          gap: 8,
        }}
      >
        {/* Left: Actions */}
        <div style={{ display: "flex", alignItems: "center", gap: 4 }}>

          {/* Add torrent — primary */}
          <button
            onClick={() => setShowAddDialog(!showAddDialog)}
            className="btn-primary transition-colors-fast"
            style={{ height: 30, padding: "0 12px", fontSize: 12 }}
          >
            <Plus size={13} strokeWidth={2.5} />
            Add
          </button>

          {/* Open .torrent file */}
          <button
            onClick={handleAddFile}
            className="btn-ghost transition-colors-fast"
            title="Open .torrent file"
            style={{ height: 30, padding: "0 9px", fontSize: 12 }}
          >
            <FileUp size={13} />
          </button>

          {/* Separator */}
          <div style={{ width: 1, height: 18, background: "var(--line)", margin: "0 2px" }} />

          {/* Search */}
          <button
            onClick={() => setSearchOpen(true)}
            className="btn-ghost transition-colors-fast"
            title="Search torrents (Torbox)"
            style={{ height: 30, padding: "0 9px" }}
          >
            <Search size={13} />
          </button>

          {/* Separator */}
          <div style={{ width: 1, height: 18, background: "var(--line)", margin: "0 2px" }} />

          {/* Pause all */}
          <button
            onClick={handlePauseAll}
            disabled={!hasActive}
            className="btn-ghost transition-colors-fast"
            title="Pause all"
            style={{ height: 30, padding: "0 9px", opacity: hasActive ? 1 : 0.3 }}
          >
            <PauseCircle size={13} />
          </button>

          {/* Resume all */}
          <button
            onClick={handleResumeAll}
            disabled={!hasPaused}
            className="btn-ghost transition-colors-fast"
            title="Resume all"
            style={{ height: 30, padding: "0 9px", opacity: hasPaused ? 1 : 0.3 }}
          >
            <PlayCircle size={13} />
          </button>

          {/* Inline magnet input */}
          {showAddDialog && (
            <>
              <div style={{ width: 1, height: 18, background: "var(--line)", margin: "0 2px" }} />
              <div style={{ position: "relative", display: "flex", alignItems: "center" }}>
                <input
                  type="text"
                  value={magnetInput}
                  onChange={(e) => setMagnetInput(e.target.value)}
                  onKeyDown={handleKeyDown}
                  placeholder="Paste magnet link or URL…"
                  autoFocus
                  style={{
                    width: 300,
                    height: 30,
                    padding: "0 30px 0 10px",
                    borderRadius: 6,
                    border: "1px solid var(--line)",
                    background: "var(--overlay)",
                    color: "var(--fg)",
                    fontSize: 12,
                    outline: "none",
                    transition: "border-color 150ms, box-shadow 150ms",
                  }}
                  onFocus={(e) => {
                    e.currentTarget.style.borderColor = "var(--accent)";
                    e.currentTarget.style.boxShadow = "0 0 0 3px var(--accent-soft)";
                  }}
                  onBlur={(e) => {
                    e.currentTarget.style.borderColor = "var(--line)";
                    e.currentTarget.style.boxShadow = "none";
                  }}
                />
                {magnetInput && (
                  <button
                    onClick={() => setMagnetInput("")}
                    style={{
                      position: "absolute",
                      right: 8,
                      background: "none",
                      border: "none",
                      cursor: "pointer",
                      color: "var(--fg-3)",
                      display: "flex",
                      padding: 2,
                    }}
                  >
                    <X size={11} />
                  </button>
                )}
              </div>
              <button
                onClick={handleAddMagnet}
                disabled={!magnetInput.trim()}
                className="btn-primary transition-colors-fast"
                style={{ height: 30, padding: "0 12px", fontSize: 12, opacity: magnetInput.trim() ? 1 : 0.4 }}
              >
                Add
              </button>
            </>
          )}
        </div>

        {/* Right: Global speeds + settings */}
        <div style={{ display: "flex", alignItems: "center", gap: 12 }}>
          {/* Speed display */}
          <div
            style={{
              display: "flex",
              alignItems: "center",
              gap: 10,
              padding: "4px 10px",
              borderRadius: 6,
              background: "var(--overlay)",
              border: "1px solid var(--line)",
            }}
          >
            <div style={{ display: "flex", alignItems: "center", gap: 4 }}>
              <div style={{ width: 5, height: 5, borderRadius: "50%", background: "var(--green)", boxShadow: "0 0 4px var(--green-glow)" }} />
              <ArrowDown size={11} color="var(--green)" />
              <span style={{ fontSize: 11, fontFamily: "'JetBrains Mono', monospace", fontVariantNumeric: "tabular-nums", color: "var(--fg-2)" }}>
                {formatSpeed(globalDownloadSpeed)}
              </span>
            </div>
            <div style={{ width: 1, height: 12, background: "var(--line)" }} />
            <div style={{ display: "flex", alignItems: "center", gap: 4 }}>
              <div style={{ width: 5, height: 5, borderRadius: "50%", background: "var(--accent)", boxShadow: "0 0 4px var(--accent-glow)" }} />
              <ArrowUp size={11} color="var(--accent)" />
              <span style={{ fontSize: 11, fontFamily: "'JetBrains Mono', monospace", fontVariantNumeric: "tabular-nums", color: "var(--fg-2)" }}>
                {formatSpeed(globalUploadSpeed)}
              </span>
            </div>
          </div>

          {/* Settings */}
          <button
            onClick={() => setShowSettings(!showSettings)}
            className="transition-colors-fast"
            title="Settings"
            style={{
              width: 30,
              height: 30,
              borderRadius: 6,
              border: "none",
              background: showSettings ? "var(--muted)" : "transparent",
              cursor: "pointer",
              color: "var(--fg-3)",
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
            }}
            onMouseEnter={(e) => {
              (e.currentTarget as HTMLButtonElement).style.background = "var(--muted)";
              (e.currentTarget as HTMLButtonElement).style.color = "var(--fg-2)";
            }}
            onMouseLeave={(e) => {
              (e.currentTarget as HTMLButtonElement).style.background = showSettings ? "var(--muted)" : "transparent";
              (e.currentTarget as HTMLButtonElement).style.color = "var(--fg-3)";
            }}
          >
            <Settings size={15} />
          </button>
        </div>
      </div>

      {/* Settings Panel */}
      {showSettings && <SettingsPanel onClose={() => setShowSettings(false)} />}

      {/* Search Panel */}
      {searchOpen && <SearchPanel onClose={() => setSearchOpen(false)} />}
    </>
  );
}
