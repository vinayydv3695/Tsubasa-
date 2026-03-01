// Tsubasa (翼) — Torrent Table Component (v3 — Manifesto Redesign)
// @tanstack/react-table, sortable, context menu, CSS classes.
// Source badges, inline progress bars, density-aware rows.

import React, { useState, useCallback, useRef, useEffect } from "react";
import {
  useReactTable,
  getCoreRowModel,
  getSortedRowModel,
  flexRender,
  createColumnHelper,
  type SortingState,
} from "@tanstack/react-table";
import {
  Download,
  Upload,
  Pause,
  CheckCircle,
  AlertCircle,
  Clock,
  Loader,
  StopCircle,
  Cloud,
  Play,
  Trash2,
  Copy,
  FolderOpen,
  ChevronUp,
  ChevronDown,
  ChevronsUpDown,
  Magnet,
} from "lucide-react";
import { useTorrentStore } from "@/stores/torrent";
import { useUIStore } from "@/stores/ui";
import { useCategoryStore, getCategoryForTorrent } from "@/stores/categories";
import type { TorrentSummary, TorrentState } from "@/types";
import {
  formatBytes,
  formatSpeed,
  formatEta,
  formatProgress,
  formatRelativeTime,
} from "@/lib/utils";
import "./TorrentTable.css";

// ─── Helpers ────────────────────────────────────────────

function stateIcon(state: TorrentState) {
  const size = 13;
  const sw = 1.5;
  switch (state) {
    case "downloading": return <Download size={size} strokeWidth={sw} style={{ color: "var(--green)" }} />;
    case "seeding": return <Upload size={size} strokeWidth={sw} style={{ color: "var(--green)" }} />;
    case "paused": return <Pause size={size} strokeWidth={sw} style={{ color: "var(--fg-3)" }} />;
    case "completed": return <CheckCircle size={size} strokeWidth={sw} style={{ color: "var(--green)" }} />;
    case "errored": return <AlertCircle size={size} strokeWidth={sw} style={{ color: "var(--red)" }} />;
    case "pending":
    case "checking": return <Loader size={size} strokeWidth={sw} style={{ color: "var(--amber)", animation: "spin 1s linear infinite" }} />;
    case "queued": return <Clock size={size} strokeWidth={sw} style={{ color: "var(--fg-3)" }} />;
    case "stopped": return <StopCircle size={size} strokeWidth={sw} style={{ color: "var(--fg-3)" }} />;
  }
}

type BadgeVariant = "green" | "amber" | "red" | "muted" | "accent";

function stateBadge(state: TorrentState) {
  const map: Record<TorrentState, { label: string; variant: BadgeVariant }> = {
    downloading: { label: "Downloading", variant: "accent" },
    seeding: { label: "Seeding", variant: "green" },
    completed: { label: "Completed", variant: "green" },
    paused: { label: "Paused", variant: "muted" },
    stopped: { label: "Stopped", variant: "muted" },
    queued: { label: "Queued", variant: "muted" },
    checking: { label: "Checking", variant: "amber" },
    pending: { label: "Pending", variant: "amber" },
    errored: { label: "Error", variant: "red" },
  };
  const { label, variant } = map[state] ?? { label: state, variant: "muted" };
  return <span className={`badge badge-${variant}`}>{label}</span>;
}

function sourceBadge(policy: string) {
  if (policy === "cloud_only") return <span className="badge badge-cloud"><Cloud size={9} /> Cloud</span>;
  if (policy === "hybrid") return <span className="badge badge-accent"><Cloud size={9} /> Hybrid</span>;
  return null;
}

function ProgressBar({ progress, state }: { progress: number; state: TorrentState }) {
  const isError = state === "errored";
  const isDone = state === "completed" || state === "seeding";
  const isIdle = state === "paused" || state === "stopped";

  const variant = isError ? "red" : isDone ? "green" : isIdle ? "muted" : "accent";

  return (
    <div className="torrent-table__progress">
      <div
        className={`torrent-table__progress-fill torrent-table__progress-fill--${variant}`}
        style={{ width: `${Math.min(progress * 100, 100)}%` }}
      />
    </div>
  );
}

function SortIcon({ sorted }: { sorted: false | "asc" | "desc" }) {
  if (!sorted) return <ChevronsUpDown size={11} style={{ color: "var(--fg-muted)", flexShrink: 0 }} />;
  return sorted === "asc"
    ? <ChevronUp size={11} style={{ color: "var(--accent)", flexShrink: 0 }} />
    : <ChevronDown size={11} style={{ color: "var(--accent)", flexShrink: 0 }} />;
}

// ─── Column Definitions ─────────────────────────────────

const columnHelper = createColumnHelper<TorrentSummary>();

const columns = [
  columnHelper.accessor("name", {
    header: "Name",
    size: 999,
    cell: (info) => {
      const t = info.row.original;
      return (
        <div className="torrent-table__name-cell">
          <div className="torrent-table__name-icon">{stateIcon(t.state)}</div>
          <div className="torrent-table__name-content">
            <div className="torrent-table__name-row">
              <span className="torrent-table__name-text">{t.name}</span>
              {sourceBadge(t.policy)}
            </div>
            <ProgressBar progress={t.progress} state={t.state} />
          </div>
        </div>
      );
    },
  }),
  columnHelper.accessor("total_bytes", {
    header: "Size",
    size: 80,
    cell: (info) => (
      <span className="torrent-table__mono">
        {info.getValue() > 0 ? formatBytes(info.getValue()) : "—"}
      </span>
    ),
  }),
  columnHelper.accessor("progress", {
    header: "Done",
    size: 60,
    cell: (info) => (
      <span className="torrent-table__mono">{formatProgress(info.getValue())}</span>
    ),
  }),
  columnHelper.accessor("state", {
    header: "Status",
    size: 96,
    cell: (info) => stateBadge(info.getValue()),
  }),
  columnHelper.accessor("download_speed", {
    header: "Down",
    size: 80,
    cell: (info) => (
      <span className={`torrent-table__mono ${info.getValue() > 0 ? "torrent-table__mono--green" : "torrent-table__mono--muted"}`}>
        {info.getValue() > 0 ? formatSpeed(info.getValue()) : "—"}
      </span>
    ),
  }),
  columnHelper.accessor("upload_speed", {
    header: "Up",
    size: 80,
    cell: (info) => (
      <span className={`torrent-table__mono ${info.getValue() > 0 ? "torrent-table__mono--accent" : "torrent-table__mono--muted"}`}>
        {info.getValue() > 0 ? formatSpeed(info.getValue()) : "—"}
      </span>
    ),
  }),
  columnHelper.accessor(
    (row) => `${row.seeds_connected}/${row.peers_connected}`,
    {
      id: "peers",
      header: "Peers",
      size: 60,
      cell: (info) => <span className="torrent-table__mono">{info.getValue()}</span>,
    }
  ),
  columnHelper.accessor("eta_seconds", {
    header: "ETA",
    size: 64,
    cell: (info) => <span className="torrent-table__mono">{formatEta(info.getValue())}</span>,
    sortingFn: (rowA, rowB) => {
      const a = rowA.original.eta_seconds ?? Infinity;
      const b = rowB.original.eta_seconds ?? Infinity;
      return a - b;
    },
  }),
  columnHelper.accessor("added_at", {
    header: "Added",
    size: 80,
    cell: (info) => (
      <span className="torrent-table__mono torrent-table__mono--muted">
        {formatRelativeTime(info.getValue())}
      </span>
    ),
  }),
];

// ─── Context Menu ───────────────────────────────────────

interface ContextMenuProps {
  x: number;
  y: number;
  torrent: TorrentSummary;
  onClose: () => void;
}

function ContextMenu({ x, y, torrent, onClose }: ContextMenuProps) {
  const pauseTorrent = useTorrentStore((s) => s.pauseTorrent);
  const resumeTorrent = useTorrentStore((s) => s.resumeTorrent);
  const removeTorrent = useTorrentStore((s) => s.removeTorrent);
  const menuRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    function handleClick(e: MouseEvent) {
      if (menuRef.current && !menuRef.current.contains(e.target as Node)) onClose();
    }
    function handleEsc(e: KeyboardEvent) { if (e.key === "Escape") onClose(); }
    document.addEventListener("mousedown", handleClick);
    document.addEventListener("keydown", handleEsc);
    return () => { document.removeEventListener("mousedown", handleClick); document.removeEventListener("keydown", handleEsc); };
  }, [onClose]);

  const isPausable = torrent.state === "downloading" || torrent.state === "seeding";
  const isResumable = torrent.state === "paused" || torrent.state === "stopped" || torrent.state === "errored";

  const items = [
    isPausable && { label: "Pause", icon: <Pause size={13} />, action: () => pauseTorrent(torrent.id), danger: false },
    isResumable && { label: "Resume", icon: <Play size={13} />, action: () => resumeTorrent(torrent.id), danger: false },
    { label: "Copy Info Hash", icon: <Copy size={13} />, action: () => navigator.clipboard.writeText(torrent.info_hash), danger: false },
    { label: "Copy Magnet Link", icon: <Magnet size={13} />, action: () => navigator.clipboard.writeText(`magnet:?xt=urn:btih:${torrent.info_hash}`), danger: false },
    { label: "Open Save Path", icon: <FolderOpen size={13} />, action: () => { }, danger: false },
    null,
    { label: "Remove", icon: <Trash2 size={13} />, action: () => removeTorrent(torrent.id, false), danger: true },
    { label: "Remove + Delete Files", icon: <Trash2 size={13} />, action: () => removeTorrent(torrent.id, true), danger: true },
  ].filter(Boolean) as Array<{ label: string; icon: React.ReactNode; action: () => void; danger: boolean } | null>;

  return (
    <div ref={menuRef} className="context-menu" style={{ left: x, top: y }}>
      {items.map((item, i) => {
        if (item === null) return <div key={`sep-${i}`} className="context-menu__separator" />;
        return (
          <button
            key={item.label}
            onClick={() => { item.action(); onClose(); }}
            className={`context-menu__item ${item.danger ? "context-menu__item--danger" : ""}`}
          >
            {item.icon}
            {item.label}
          </button>
        );
      })}
    </div>
  );
}

// ─── Empty State ────────────────────────────────────────

function EmptyState() {
  return (
    <div className="torrent-table__empty">
      <div className="torrent-table__empty-icon">
        <Download size={22} color="var(--fg-muted)" />
      </div>
      <div style={{ textAlign: "center" }}>
        <p className="torrent-table__empty-title">Add your first torrent</p>
        <p className="torrent-table__empty-subtitle">
          Paste a magnet link or drop a .torrent file
        </p>
      </div>
    </div>
  );
}

// ─── Main Table Component ───────────────────────────────

export function TorrentTable() {
  const torrents = useTorrentStore((s) => s.torrents);
  const filter = useTorrentStore((s) => s.filter);
  const selectedTorrentId = useTorrentStore((s) => s.selectedTorrentId);
  const selectTorrent = useTorrentStore((s) => s.selectTorrent);
  const setDetailPanelOpen = useUIStore((s) => s.setDetailPanelOpen);

  const [sorting, setSorting] = useState<SortingState>([]);
  const [contextMenu, setContextMenu] = useState<{ x: number; y: number; torrent: TorrentSummary } | null>(null);

  // Filter torrents
  const torrentTags = useCategoryStore((s) => s.torrentTags);
  const data = React.useMemo(() => {
    const list = Array.from(torrents.values());

    if (filter === "all") return list;
    if (filter === "downloading") return list.filter((t) => t.state === "downloading" || t.state === "pending" || t.state === "checking" || t.state === "queued");
    if (filter === "paused") return list.filter((t) => t.state === "paused");
    if (filter === "completed") return list.filter((t) => t.state === "completed" || t.state === "stopped");
    if (filter === "seeding") return list.filter((t) => t.state === "seeding");
    if (filter === "errored") return list.filter((t) => t.state === "errored");
    if (filter === "cloud") return list.filter((t) => t.policy === "cloud_only" || t.policy === "hybrid");

    // Category filter
    if (filter.startsWith("cat:")) {
      const catName = filter.slice(4);
      if (catName === "Other") return list.filter((t) => getCategoryForTorrent(t.name) === null);
      return list.filter((t) => getCategoryForTorrent(t.name)?.name === catName);
    }

    // Tag filter
    if (filter.startsWith("tag:")) {
      const tagName = filter.slice(4);
      if (tagName === "(Untagged)") return list.filter((t) => !torrentTags[t.id]);
      return list.filter((t) => torrentTags[t.id] === tagName);
    }

    if (filter.startsWith("tracker:")) return list;

    return list.filter((t) => t.state === filter);
  }, [torrents, filter, torrentTags]);

  const table = useReactTable({
    data,
    columns,
    state: { sorting },
    onSortingChange: setSorting,
    getCoreRowModel: getCoreRowModel(),
    getSortedRowModel: getSortedRowModel(),
  });

  const handleContextMenu = useCallback((e: React.MouseEvent, torrent: TorrentSummary) => {
    e.preventDefault();
    selectTorrent(torrent.id);
    setContextMenu({ x: e.clientX, y: e.clientY, torrent });
  }, [selectTorrent]);

  const closeContextMenu = useCallback(() => setContextMenu(null), []);

  if (data.length === 0) return <EmptyState />;

  return (
    <div className="torrent-table">
      <table>
        <thead>
          {table.getHeaderGroups().map((headerGroup) => (
            <tr key={headerGroup.id}>
              {headerGroup.headers.map((header) => (
                <th
                  key={header.id}
                  style={{ width: header.getSize() === 999 ? undefined : header.getSize() }}
                >
                  {!header.isPlaceholder && (
                    <button
                      className="torrent-table__header-btn"
                      onClick={header.column.getToggleSortingHandler()}
                    >
                      {flexRender(header.column.columnDef.header, header.getContext())}
                      <SortIcon sorted={header.column.getIsSorted()} />
                    </button>
                  )}
                </th>
              ))}
            </tr>
          ))}
        </thead>

        <tbody>
          {table.getRowModel().rows.map((row) => {
            const torrent = row.original;
            const isSelected = selectedTorrentId === torrent.id;

            return (
              <tr
                key={row.id}
                data-selected={isSelected}
                onClick={() => selectTorrent(torrent.id)}
                onDoubleClick={() => { selectTorrent(torrent.id); setDetailPanelOpen(true); }}
                onContextMenu={(e) => handleContextMenu(e, torrent)}
              >
                {row.getVisibleCells().map((cell) => (
                  <td key={cell.id}>
                    {flexRender(cell.column.columnDef.cell, cell.getContext())}
                  </td>
                ))}
              </tr>
            );
          })}
        </tbody>
      </table>

      {contextMenu && (
        <ContextMenu
          x={contextMenu.x}
          y={contextMenu.y}
          torrent={contextMenu.torrent}
          onClose={closeContextMenu}
        />
      )}
    </div>
  );
}
