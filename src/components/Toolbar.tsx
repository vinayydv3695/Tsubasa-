// Tsubasa (翼) — Toolbar Component (v3 — Manifesto Redesign)
// Add torrent, pause/resume all, search, global speed display.
// Settings button moved to sidebar per manifesto.

import React, { useState, useCallback } from "react";
import {
  Plus,
  ArrowDown,
  ArrowUp,
  PauseCircle,
  PlayCircle,
  FileUp,
  Search,
  X,
  Command,
} from "lucide-react";
import { open as dialogOpen } from "@tauri-apps/plugin-dialog";
import { useTorrentStore } from "@/stores/torrent";
import { useUIStore } from "@/stores/ui";
import { useSearchStore } from "@/stores/search";
import { formatSpeed } from "@/lib/utils";
import { SettingsPanel } from "@/components/SettingsPanel";
import { SearchPanel } from "@/components/SearchPanel";
import "./Toolbar.css";

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
      <div className="toolbar">
        {/* Left: Actions */}
        <div className="toolbar__left">
          {/* Add torrent — primary */}
          <button
            onClick={() => setShowAddDialog(!showAddDialog)}
            className="btn-primary"
            style={{ height: 30, padding: "0 12px" }}
          >
            <Plus size={13} strokeWidth={2.5} />
            <span className="sidebar-label">Add</span>
          </button>

          {/* Open .torrent file */}
          <button
            onClick={handleAddFile}
            className="btn-icon"
            title="Open .torrent file"
          >
            <FileUp size={15} strokeWidth={1.5} />
          </button>

          <div className="toolbar__separator" />

          {/* Search */}
          <button
            onClick={() => setSearchOpen(true)}
            className="btn-icon"
            title="Search torrents"
          >
            <Search size={15} strokeWidth={1.5} />
          </button>

          <div className="toolbar__separator" />

          {/* Pause all */}
          <button
            onClick={handlePauseAll}
            disabled={!hasActive}
            className="btn-icon"
            title="Pause all"
            style={{ opacity: hasActive ? 1 : 0.3 }}
          >
            <PauseCircle size={15} strokeWidth={1.5} />
          </button>

          {/* Resume all */}
          <button
            onClick={handleResumeAll}
            disabled={!hasPaused}
            className="btn-icon"
            title="Resume all"
            style={{ opacity: hasPaused ? 1 : 0.3 }}
          >
            <PlayCircle size={15} strokeWidth={1.5} />
          </button>

          {/* Inline magnet input */}
          {showAddDialog && (
            <>
              <div className="toolbar__separator" />
              <div style={{ position: "relative", display: "flex", alignItems: "center" }}>
                <input
                  type="text"
                  value={magnetInput}
                  onChange={(e) => setMagnetInput(e.target.value)}
                  onKeyDown={handleKeyDown}
                  placeholder="Paste magnet link or URL…"
                  autoFocus
                  className="toolbar__magnet-input"
                />
                {magnetInput && (
                  <button onClick={() => setMagnetInput("")} className="toolbar__magnet-clear">
                    <X size={11} />
                  </button>
                )}
              </div>
              <button
                onClick={handleAddMagnet}
                disabled={!magnetInput.trim()}
                className="btn-primary"
                style={{ height: 30, padding: "0 12px", opacity: magnetInput.trim() ? 1 : 0.4 }}
              >
                Add
              </button>
            </>
          )}
        </div>

        {/* Right: Speed pill + density toggle + Ctrl+K */}
        <div className="toolbar__right">
          {/* Command palette hint */}
          <button
            className="btn-icon"
            title="Command palette (Ctrl+K)"
            onClick={() => {/* TODO: open command palette */ }}
          >
            <Command size={14} strokeWidth={1.5} />
          </button>

          {/* Speed display */}
          <div className="toolbar__speed-pill">
            <div className="toolbar__speed-group">
              <span className="toolbar__speed-dot toolbar__speed-dot--down" />
              <ArrowDown size={11} color="var(--green)" />
              <span className="toolbar__speed-value">
                {formatSpeed(globalDownloadSpeed)}
              </span>
            </div>
            <div className="toolbar__speed-divider" />
            <div className="toolbar__speed-group">
              <span className="toolbar__speed-dot toolbar__speed-dot--up" />
              <ArrowUp size={11} color="var(--accent)" />
              <span className="toolbar__speed-value">
                {formatSpeed(globalUploadSpeed)}
              </span>
            </div>
          </div>
        </div>
      </div>

      {/* Settings Panel */}
      {showSettings && <SettingsPanel onClose={() => setShowSettings(false)} />}

      {/* Search Panel */}
      {searchOpen && <SearchPanel onClose={() => setSearchOpen(false)} />}
    </>
  );
}
