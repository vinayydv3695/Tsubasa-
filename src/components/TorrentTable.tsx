// Tsubasa — Torrent Table Component
// Uses @tanstack/react-table for sortable columns.
// Includes right-click context menu for torrent actions.

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

// ─── Helpers ────────────────────────────────────────────

function stateIcon(state: TorrentState) {
  const style: React.CSSProperties = { flexShrink: 0 };
  switch (state) {
    case "downloading": return <Download size={13} style={{ ...style, color: "var(--green)" }} />;
    case "seeding": return <Upload size={13} style={{ ...style, color: "var(--green)" }} />;
    case "paused": return <Pause size={13} style={{ ...style, color: "var(--fg-3)" }} />;
    case "completed": return <CheckCircle size={13} style={{ ...style, color: "var(--green)" }} />;
    case "errored": return <AlertCircle size={13} style={{ ...style, color: "var(--red)" }} />;
    case "pending":
    case "checking": return <Loader size={13} style={{ ...style, color: "var(--amber)", animation: "spin 1s linear infinite" }} />;
    case "queued": return <Clock size={13} style={{ ...style, color: "var(--fg-3)" }} />;
    case "stopped": return <StopCircle size={13} style={{ ...style, color: "var(--fg-3)" }} />;
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

function ProgressBar({ progress, state }: { progress: number; state: TorrentState }) {
  const isError = state === "errored";
  const isDone = state === "completed" || state === "seeding";
  const isIdle = state === "paused" || state === "stopped";
  const isActive = state === "downloading";

  const bg = isError ? "var(--red)"
    : isDone ? "var(--green)"
      : isIdle ? "var(--fg-muted)"
        : "var(--accent)";

  const glow = isError ? "0 0 5px var(--red-glow)"
    : isDone ? "0 0 5px var(--green-glow)"
      : isActive ? "0 0 6px var(--accent-glow)"
        : "none";

  return (
    <div
      style={{
        width: "100%",
        height: 3,
        borderRadius: 99,
        background: "var(--muted)",
        overflow: "visible",
        marginTop: 4,
      }}
    >
      <div
        style={{
          height: "100%",
          width: `${Math.min(progress * 100, 100)}%`,
          borderRadius: 99,
          background: bg,
          boxShadow: glow,
          transition: "width 300ms ease",
        }}
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
        <div style={{ display: "flex", alignItems: "flex-start", gap: 8 }}>
          <div style={{ paddingTop: 1, flexShrink: 0 }}>{stateIcon(t.state)}</div>
          <div style={{ minWidth: 0, flex: 1 }}>
            <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
              <span style={{
                fontSize: 12,
                fontWeight: 500,
                color: "var(--fg)",
                overflow: "hidden",
                textOverflow: "ellipsis",
                whiteSpace: "nowrap",
              }}>
                {t.name}
              </span>
              {t.policy !== "local_only" && (
                <span className="badge badge-blue" style={{ flexShrink: 0 }}>
                  <Cloud size={9} />
                  {t.policy === "cloud_only" ? "Cloud" : "Hybrid"}
                </span>
              )}
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
    cell: (info) => {
      const v = info.getValue();
      return (
        <span style={{ fontSize: 11, color: "var(--fg-2)", fontFamily: "'JetBrains Mono', monospace", fontVariantNumeric: "tabular-nums" }}>
          {v > 0 ? formatBytes(v) : "—"}
        </span>
      );
    },
  }),
  columnHelper.accessor("progress", {
    header: "Done",
    size: 60,
    cell: (info) => (
      <span style={{ fontSize: 11, color: "var(--fg-2)", fontFamily: "'JetBrains Mono', monospace", fontVariantNumeric: "tabular-nums" }}>
        {formatProgress(info.getValue())}
      </span>
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
      <span style={{ fontSize: 11, color: info.getValue() > 0 ? "var(--green)" : "var(--fg-3)", fontFamily: "'JetBrains Mono', monospace", fontVariantNumeric: "tabular-nums" }}>
        {info.getValue() > 0 ? formatSpeed(info.getValue()) : "—"}
      </span>
    ),
  }),
  columnHelper.accessor("upload_speed", {
    header: "Up",
    size: 80,
    cell: (info) => (
      <span style={{ fontSize: 11, color: info.getValue() > 0 ? "var(--accent)" : "var(--fg-3)", fontFamily: "'JetBrains Mono', monospace", fontVariantNumeric: "tabular-nums" }}>
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
      cell: (info) => (
        <span style={{ fontSize: 11, color: "var(--fg-2)", fontFamily: "'JetBrains Mono', monospace", fontVariantNumeric: "tabular-nums" }}>
          {info.getValue()}
        </span>
      ),
    }
  ),
  columnHelper.accessor("eta_seconds", {
    header: "ETA",
    size: 64,
    cell: (info) => (
      <span style={{ fontSize: 11, color: "var(--fg-2)", fontFamily: "'JetBrains Mono', monospace", fontVariantNumeric: "tabular-nums" }}>
        {formatEta(info.getValue())}
      </span>
    ),
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
      <span style={{ fontSize: 11, color: "var(--fg-3)" }}>
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
    <div
      ref={menuRef}
      style={{
        position: "fixed",
        zIndex: 30,
        left: x,
        top: y,
        minWidth: 180,
        background: "var(--surface)",
        border: "1px solid var(--line-strong)",
        borderRadius: 8,
        boxShadow: "var(--shadow-lg)",
        padding: 4,
        backdropFilter: "blur(12px) saturate(160%)",
      }}
    >
      {items.map((item, i) => {
        if (item === null) {
          return <div key={`sep-${i}`} style={{ margin: "4px 0", height: 1, background: "var(--line)" }} />;
        }
        return (
          <button
            key={item.label}
            onClick={() => { item.action(); onClose(); }}
            className="transition-colors-fast"
            style={{
              width: "100%",
              display: "flex",
              alignItems: "center",
              gap: 8,
              padding: "6px 12px",
              border: "none",
              background: "transparent",
              cursor: "pointer",
              fontSize: 12,
              color: item.danger ? "var(--red)" : "var(--fg-2)",
              textAlign: "left",
            }}
            onMouseEnter={(e) => {
              (e.currentTarget as HTMLButtonElement).style.background = item.danger ? "var(--red-soft)" : "var(--muted)";
              (e.currentTarget as HTMLButtonElement).style.color = item.danger ? "var(--red)" : "var(--fg)";
            }}
            onMouseLeave={(e) => {
              (e.currentTarget as HTMLButtonElement).style.background = "transparent";
              (e.currentTarget as HTMLButtonElement).style.color = item.danger ? "var(--red)" : "var(--fg-2)";
            }}
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
    <div style={{ flex: 1, display: "flex", flexDirection: "column", alignItems: "center", justifyContent: "center", gap: 12 }}>
      <div style={{
        width: 56,
        height: 56,
        borderRadius: 14,
        background: "var(--overlay)",
        border: "1px solid var(--line)",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
      }}>
        <Download size={22} color="var(--fg-muted)" />
      </div>
      <div style={{ textAlign: "center" }}>
        <p style={{ fontSize: 13, fontWeight: 500, color: "var(--fg-2)" }}>No torrents yet</p>
        <p style={{ fontSize: 12, color: "var(--fg-3)", marginTop: 4 }}>
          Click <strong style={{ color: "var(--fg-2)" }}>Add</strong> to add a magnet link or torrent file
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

    // Category filter: cat:Movies, cat:TV Shows, etc.
    if (filter.startsWith("cat:")) {
      const catName = filter.slice(4);
      if (catName === "Other") {
        return list.filter((t) => getCategoryForTorrent(t.name) === null);
      }
      return list.filter((t) => {
        const cat = getCategoryForTorrent(t.name);
        return cat?.name === catName;
      });
    }

    // Tag filter: tag:SomeTag
    if (filter.startsWith("tag:")) {
      const tagName = filter.slice(4);
      if (tagName === "(Untagged)") return list.filter((t) => !torrentTags[t.id]);
      return list.filter((t) => torrentTags[t.id] === tagName);
    }

    // Tracker filter: tracker:SomeHost (not yet fully implemented—falls through to all)
    if (filter.startsWith("tracker:")) return list;

    // Fallback: treat as TorrentState
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
    <div style={{ flex: 1, overflow: "auto", position: "relative" }}>
      <table style={{ width: "100%", borderCollapse: "collapse" }}>
        {/* Sticky header */}
        <thead style={{ position: "sticky", top: 0, zIndex: 10, background: "var(--surface)" }}>
          {table.getHeaderGroups().map((headerGroup) => (
            <tr
              key={headerGroup.id}
              style={{ borderBottom: "1px solid var(--line)" }}
            >
              {headerGroup.headers.map((header) => (
                <th
                  key={header.id}
                  style={{
                    padding: "8px 12px",
                    textAlign: "left",
                    width: header.getSize() === 999 ? undefined : header.getSize(),
                    userSelect: "none",
                  }}
                >
                  {!header.isPlaceholder && (
                    <button
                      style={{
                        display: "flex",
                        alignItems: "center",
                        gap: 3,
                        background: "none",
                        border: "none",
                        cursor: "pointer",
                        fontSize: 10,
                        fontWeight: 600,
                        color: "var(--fg-3)",
                        textTransform: "uppercase",
                        letterSpacing: "0.5px",
                        padding: 0,
                      }}
                      onClick={header.column.getToggleSortingHandler()}
                      onMouseEnter={(e) => { (e.currentTarget as HTMLButtonElement).style.color = "var(--fg-2)"; }}
                      onMouseLeave={(e) => { (e.currentTarget as HTMLButtonElement).style.color = "var(--fg-3)"; }}
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
                onClick={() => selectTorrent(torrent.id)}
                onDoubleClick={() => { selectTorrent(torrent.id); setDetailPanelOpen(true); }}
                onContextMenu={(e) => handleContextMenu(e, torrent)}
                style={{
                  cursor: "pointer",
                  borderBottom: "1px solid var(--line-subtle)",
                  background: isSelected ? "var(--accent-soft)" : "transparent",
                  transition: "background-color 100ms ease",
                  position: "relative",
                }}
                onMouseEnter={(e) => {
                  if (!isSelected) (e.currentTarget as HTMLTableRowElement).style.background = "var(--muted)";
                }}
                onMouseLeave={(e) => {
                  if (!isSelected) (e.currentTarget as HTMLTableRowElement).style.background = "transparent";
                }}
              >
                {row.getVisibleCells().map((cell) => (
                  <td key={cell.id} style={{ padding: "8px 12px", verticalAlign: "middle" }}>
                    {flexRender(cell.column.columnDef.cell, cell.getContext())}
                  </td>
                ))}
              </tr>
            );
          })}
        </tbody>
      </table>

      {/* Context menu */}
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
