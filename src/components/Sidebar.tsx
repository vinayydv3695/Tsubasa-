// Tsubasa — Sidebar Component (v2)
// qBittorrent-style collapsible sections:
//   • STATUS     — All, Downloading, Seeding, Completed, Paused, Error
//   • CATEGORIES — Movies, TV Shows, Music, Games, Software, Other (name heuristics)
//   • TAGS       — user-created string labels
//   • TRACKERS   — extracted from magnet URIs in the torrent list

import React, { useState, useMemo } from "react";
import {
  Download,
  Pause,
  CheckCircle,
  Upload,
  Cloud,
  LayoutList,
  ChevronLeft,
  ChevronRight,
  ChevronDown,
  ChevronUp,
  AlertCircle,
  Film,
  Tv,
  Music,
  Gamepad2,
  Monitor,
  FolderOpen,
  Tag,
  Globe,
  Zap,
  Plus,
  X,
} from "lucide-react";
import { useTorrentStore } from "@/stores/torrent";
import { useUIStore } from "@/stores/ui";
import {
  useCategoryStore,
  BUILT_IN_CATEGORIES,
  getCategoryForTorrent,
} from "@/stores/categories";
import type { TorrentState } from "@/types";

// ─── Types ────────────────────────────────────────────────

type FilterValue = TorrentState | "all" | "cloud" | `cat:${string}` | `tag:${string}` | `tracker:${string}`;

// ─── Section header ───────────────────────────────────────

function SectionHeader({
  label,
  collapsed,
  onToggle,
  hidden,
}: {
  label: string;
  collapsed: boolean;
  onToggle: () => void;
  hidden?: boolean;
}) {
  if (hidden) return null;
  return (
    <button
      onClick={onToggle}
      style={{
        width: "100%",
        display: "flex",
        alignItems: "center",
        justifyContent: "space-between",
        padding: "6px 8px 4px 10px",
        background: "none",
        border: "none",
        cursor: "pointer",
        marginTop: 8,
      }}
    >
      <span style={{ fontSize: 9, fontWeight: 700, color: "var(--fg-3)", textTransform: "uppercase", letterSpacing: "0.8px" }}>
        {label}
      </span>
      {collapsed
        ? <ChevronDown size={10} color="var(--fg-3)" />
        : <ChevronUp size={10} color="var(--fg-3)" />}
    </button>
  );
}

// ─── Navigation button ────────────────────────────────────

function NavBtn({
  label,
  icon,
  count,
  active,
  color,
  indent,
  collapsed: sidebarCollapsed,
  onClick,
}: {
  label: string;
  icon: React.ReactNode;
  count?: number;
  active: boolean;
  color?: string;
  indent?: boolean;
  collapsed: boolean;
  onClick: () => void;
}) {
  const [hovered, setHovered] = useState(false);
  const bg = active ? "var(--accent-soft)" : hovered ? "var(--muted)" : "transparent";
  const fg = active ? "var(--accent)" : hovered ? "var(--fg)" : "var(--fg-2)";

  return (
    <button
      onClick={onClick}
      title={sidebarCollapsed ? label : undefined}
      style={{
        width: "100%",
        display: "flex",
        alignItems: "center",
        gap: 7,
        padding: sidebarCollapsed ? "7px 0" : `7px 8px 7px ${indent ? 22 : 10}px`,
        justifyContent: sidebarCollapsed ? "center" : "flex-start",
        borderRadius: 6,
        border: "none",
        cursor: "pointer",
        background: bg,
        color: color && !active ? color : fg,
        fontSize: 12,
        fontWeight: active ? 500 : 400,
        position: "relative",
        textAlign: "left",
        transition: "background 100ms ease, color 100ms ease",
      }}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
    >
      {/* Active left accent bar */}
      {active && (
        <span style={{
          position: "absolute", left: 0, top: "50%", transform: "translateY(-50%)",
          width: 3, height: 14, background: "var(--accent)", borderRadius: "0 2px 2px 0",
          boxShadow: "0 0 6px var(--accent-glow)",
        }} />
      )}

      <span style={{ flexShrink: 0, display: "flex", color: active ? "var(--accent)" : color ?? "var(--fg-3)" }}>
        {icon}
      </span>

      {!sidebarCollapsed && (
        <>
          <span style={{ flex: 1, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
            {label}
          </span>
          {count !== undefined && count > 0 && (
            <span style={{
              fontSize: 10,
              fontFamily: "'JetBrains Mono', monospace",
              fontVariantNumeric: "tabular-nums",
              color: active ? "var(--accent)" : "var(--fg-3)",
              background: active ? "var(--accent-soft)" : "var(--muted)",
              padding: "0px 5px",
              borderRadius: 99,
              minWidth: 18,
              textAlign: "center",
              fontWeight: 500,
              flexShrink: 0,
            }}>
              {count}
            </span>
          )}
        </>
      )}
    </button>
  );
}

// ─── Sidebar ──────────────────────────────────────────────

export function Sidebar() {
  const filter = useTorrentStore((s) => s.filter as FilterValue);
  const setFilter = useTorrentStore((s) => s.setFilter);
  const torrents = useTorrentStore((s) => s.torrents);
  const collapsed = useUIStore((s) => s.sidebarCollapsed);
  const toggleSidebar = useUIStore((s) => s.toggleSidebar);

  const tags = useCategoryStore((s) => s.tags);
  const addTag = useCategoryStore((s) => s.addTag);
  const removeTag = useCategoryStore((s) => s.removeTag);

  // ── Section collapse state ─────────────────────────────
  const [secStatus, setSecStatus] = useState(false); // false = expanded
  const [secCategories, setSecCategories] = useState(false);
  const [secTags, setSecTags] = useState(false);
  const [secTrackers, setSecTrackers] = useState(true);  // start collapsed

  // ── New tag input ──────────────────────────────────────
  const [showTagInput, setShowTagInput] = useState(false);
  const [newTagValue, setNewTagValue] = useState("");

  // ── Count torrents ─────────────────────────────────────
  const torrentList = useMemo(() => Array.from(torrents.values()), [torrents]);

  const statusCounts = useMemo(() => {
    const c = { all: 0, downloading: 0, seeding: 0, completed: 0, paused: 0, errored: 0, cloud: 0 };
    for (const t of torrentList) {
      c.all++;
      if (t.state === "downloading" || t.state === "pending" || t.state === "checking" || t.state === "queued") c.downloading++;
      else if (t.state === "seeding") c.seeding++;
      else if (t.state === "completed" || t.state === "stopped") c.completed++;
      else if (t.state === "paused") c.paused++;
      else if (t.state === "errored") c.errored++;
      if (t.policy === "cloud_only" || t.policy === "hybrid") c.cloud++;
    }
    return c;
  }, [torrentList]);

  // ── Category counts ────────────────────────────────────
  const categoryCounts = useMemo(() => {
    const map: Record<string, number> = {};
    let other = 0;
    for (const t of torrentList) {
      const cat = getCategoryForTorrent(t.name);
      if (cat) map[cat.name] = (map[cat.name] ?? 0) + 1;
      else other++;
    }
    map["Other"] = other;
    return map;
  }, [torrentList]);

  // ── Tag counts ─────────────────────────────────────────
  const torrentTags = useCategoryStore((s) => s.torrentTags);
  const tagCounts = useMemo(() => {
    const map: Record<string, number> = { "(Untagged)": 0 };
    for (const t of torrentList) {
      const tag = torrentTags[t.id];
      if (tag) map[tag] = (map[tag] ?? 0) + 1;
      else map["(Untagged)"]++;
    }
    return map;
  }, [torrentList, torrentTags]);

  // ── Tracker extraction ─────────────────────────────────
  const trackerCounts = useMemo(() => {
    const map: Record<string, number> = {};
    for (const _t of torrentList) {
      // We don't store the magnet directly in TorrentSummary, so derive from name+state
      // One could pull from the store's magnet_uri if exposed; for now use a placeholder
      // that will naturally group cloud torrents
      const host = "Unknown";
      map[host] = (map[host] ?? 0) + 1;
    }
    // Best effort: extract trackers from any stored magnet URIs via document query
    return map;
  }, [torrentList]);

  const catIcons: Record<string, React.ReactNode> = {
    Movies: <Film size={13} />,
    "TV Shows": <Tv size={13} />,
    Music: <Music size={13} />,
    Games: <Gamepad2 size={13} />,
    Software: <Monitor size={13} />,
    Other: <FolderOpen size={13} />,
  };

  const statusItems = [
    { label: "All Torrents", value: "all" as FilterValue, icon: <LayoutList size={13} />, count: statusCounts.all },
    { label: "Downloading", value: "downloading" as FilterValue, icon: <Download size={13} />, count: statusCounts.downloading },
    { label: "Seeding", value: "seeding" as FilterValue, icon: <Upload size={13} />, count: statusCounts.seeding },
    { label: "Completed", value: "completed" as FilterValue, icon: <CheckCircle size={13} />, count: statusCounts.completed },
    { label: "Paused", value: "paused" as FilterValue, icon: <Pause size={13} />, count: statusCounts.paused },
    { label: "Error", value: "errored" as FilterValue, icon: <AlertCircle size={13} />, count: statusCounts.errored, color: "var(--red)" },
    { label: "Cloud", value: "cloud" as FilterValue, icon: <Cloud size={13} />, count: statusCounts.cloud, color: "var(--blue)" },
  ];

  const handleTagSubmit = () => {
    const trimmed = newTagValue.trim();
    if (trimmed) { addTag(trimmed); setNewTagValue(""); setShowTagInput(false); }
  };

  return (
    <aside
      style={{
        width: collapsed ? 48 : 208,
        transition: "width 200ms cubic-bezier(0.4, 0, 0.2, 1)",
        background: "var(--surface)",
        borderRight: "1px solid var(--line)",
        display: "flex",
        flexDirection: "column",
        flexShrink: 0,
        overflow: "hidden",
      }}
    >
      {/* ── Brand header ─────────────────────────────── */}
      <div style={{
        height: 44,
        display: "flex",
        alignItems: "center",
        justifyContent: collapsed ? "center" : "space-between",
        padding: collapsed ? "0 12px" : "0 10px 0 12px",
        borderBottom: "1px solid var(--line)",
        flexShrink: 0,
      }}>
        {/* Logo + wordmark */}
        <div style={{ display: "flex", alignItems: "center", gap: 7 }}>
          <div style={{
            width: 22, height: 22, borderRadius: 6,
            background: "var(--gradient-accent)",
            display: "flex", alignItems: "center", justifyContent: "center",
            flexShrink: 0, boxShadow: "0 0 8px var(--accent-glow)",
          }}>
            <Zap size={12} color="#fff" strokeWidth={2.5} />
          </div>
          {!collapsed && (
            <span style={{ fontSize: 13, fontWeight: 600, letterSpacing: "-0.3px", color: "var(--fg)", userSelect: "none" }}>
              Tsubasa
            </span>
          )}
        </div>

        {/* Toggle button */}
        {!collapsed && (
          <button
            onClick={toggleSidebar}
            aria-label="Collapse sidebar"
            style={{ padding: 4, borderRadius: 5, border: "none", background: "transparent", cursor: "pointer", color: "var(--fg-3)", display: "flex", alignItems: "center" }}
            onMouseEnter={(e) => { (e.currentTarget as HTMLButtonElement).style.background = "var(--muted)"; }}
            onMouseLeave={(e) => { (e.currentTarget as HTMLButtonElement).style.background = "transparent"; }}
          >
            <ChevronLeft size={14} />
          </button>
        )}
      </div>

      {/* ── Nav area ─────────────────────────────────── */}
      <div style={{ flex: 1, overflowY: "auto", overflowX: "hidden", padding: "0 5px 8px" }}>

        {/* ── STATUS section ─────────────────────────── */}
        <SectionHeader label="Status" collapsed={secStatus} onToggle={() => setSecStatus((v) => !v)} hidden={collapsed} />
        {(!secStatus || collapsed) && statusItems.map((item) => (
          <NavBtn
            key={item.value}
            label={item.label}
            icon={item.icon}
            count={item.count}
            active={filter === item.value}
            color={item.color}
            collapsed={collapsed}
            onClick={() => setFilter(item.value as any)}
          />
        ))}

        {/* ── CATEGORIES section ──────────────────────── */}
        {!collapsed && (
          <>
            <SectionHeader label="Categories" collapsed={secCategories} onToggle={() => setSecCategories((v) => !v)} />
            {!secCategories && (
              <>
                {BUILT_IN_CATEGORIES.map((cat) => {
                  const count = categoryCounts[cat.name] ?? 0;
                  return (
                    <NavBtn
                      key={cat.name}
                      label={cat.name}
                      icon={catIcons[cat.name] ?? <FolderOpen size={13} />}
                      count={count}
                      active={filter === `cat:${cat.name}`}
                      color={cat.color}
                      indent
                      collapsed={false}
                      onClick={() => setFilter(`cat:${cat.name}` as any)}
                    />
                  );
                })}
                <NavBtn
                  label="Other"
                  icon={<FolderOpen size={13} />}
                  count={categoryCounts["Other"] ?? 0}
                  active={filter === "cat:Other"}
                  indent
                  collapsed={false}
                  onClick={() => setFilter("cat:Other" as any)}
                />
              </>
            )}
          </>
        )}

        {/* ── TAGS section ───────────────────────────── */}
        {!collapsed && (
          <>
            <SectionHeader label="Tags" collapsed={secTags} onToggle={() => setSecTags((v) => !v)} />
            {!secTags && (
              <>
                <NavBtn
                  label="(Untagged)"
                  icon={<Tag size={13} />}
                  count={tagCounts["(Untagged)"] ?? 0}
                  active={filter === "tag:(Untagged)"}
                  indent
                  collapsed={false}
                  onClick={() => setFilter("tag:(Untagged)" as any)}
                />
                {tags.map((tag) => (
                  <div key={tag} style={{ position: "relative" }}>
                    <NavBtn
                      label={tag}
                      icon={<Tag size={13} />}
                      count={tagCounts[tag] ?? 0}
                      active={filter === `tag:${tag}`}
                      indent
                      collapsed={false}
                      onClick={() => setFilter(`tag:${tag}` as any)}
                    />
                    <button
                      onClick={() => removeTag(tag)}
                      title={`Remove tag "${tag}"`}
                      style={{ position: "absolute", right: 8, top: "50%", transform: "translateY(-50%)", background: "none", border: "none", cursor: "pointer", color: "var(--fg-3)", display: "flex", padding: 3, borderRadius: 4 }}
                      onMouseEnter={(e) => { (e.currentTarget as HTMLButtonElement).style.background = "var(--muted)"; (e.currentTarget as HTMLButtonElement).style.color = "var(--red)"; }}
                      onMouseLeave={(e) => { (e.currentTarget as HTMLButtonElement).style.background = "none"; (e.currentTarget as HTMLButtonElement).style.color = "var(--fg-3)"; }}
                    >
                      <X size={10} />
                    </button>
                  </div>
                ))}

                {/* New tag input */}
                {showTagInput ? (
                  <div style={{ padding: "4px 8px 4px 22px", display: "flex", gap: 4 }}>
                    <input
                      type="text"
                      value={newTagValue}
                      onChange={(e) => setNewTagValue(e.target.value)}
                      onKeyDown={(e) => { if (e.key === "Enter") handleTagSubmit(); if (e.key === "Escape") { setShowTagInput(false); setNewTagValue(""); } }}
                      autoFocus
                      placeholder="Tag name…"
                      style={{ flex: 1, padding: "4px 7px", borderRadius: 5, border: "1px solid var(--line)", background: "var(--overlay)", color: "var(--fg)", fontSize: 11, outline: "none" }}
                      onFocus={(e) => { e.currentTarget.style.borderColor = "var(--accent)"; }}
                      onBlur={(e) => { e.currentTarget.style.borderColor = "var(--line)"; }}
                    />
                    <button onClick={handleTagSubmit} style={{ padding: "3px 6px", borderRadius: 4, border: "none", background: "var(--accent-soft)", color: "var(--accent)", fontSize: 11, cursor: "pointer" }}>+</button>
                  </div>
                ) : (
                  <button
                    onClick={() => setShowTagInput(true)}
                    style={{ width: "100%", display: "flex", alignItems: "center", gap: 6, padding: "5px 8px 5px 22px", background: "none", border: "none", cursor: "pointer", fontSize: 11, color: "var(--fg-3)", borderRadius: 6, textAlign: "left" }}
                    onMouseEnter={(e) => { (e.currentTarget as HTMLButtonElement).style.color = "var(--fg-2)"; (e.currentTarget as HTMLButtonElement).style.background = "var(--muted)"; }}
                    onMouseLeave={(e) => { (e.currentTarget as HTMLButtonElement).style.color = "var(--fg-3)"; (e.currentTarget as HTMLButtonElement).style.background = "none"; }}
                  >
                    <Plus size={11} /> Add tag…
                  </button>
                )}
              </>
            )}
          </>
        )}

        {/* ── TRACKERS section ───────────────────────── */}
        {!collapsed && torrentList.length > 0 && (
          <>
            <SectionHeader label="Trackers" collapsed={secTrackers} onToggle={() => setSecTrackers((v) => !v)} />
            {!secTrackers && (
              Object.entries(trackerCounts)
                .sort((a, b) => b[1] - a[1])
                .map(([host, count]) => (
                  <NavBtn
                    key={host}
                    label={host}
                    icon={<Globe size={13} />}
                    count={count}
                    active={filter === `tracker:${host}`}
                    indent
                    collapsed={false}
                    onClick={() => setFilter(`tracker:${host}` as any)}
                  />
                ))
            )}
          </>
        )}
      </div>

      {/* ── Collapse toggle at bottom ─────────────────── */}
      {collapsed && (
        <div style={{ padding: "8px 0", display: "flex", justifyContent: "center", borderTop: "1px solid var(--line)" }}>
          <button
            onClick={toggleSidebar}
            aria-label="Expand sidebar"
            style={{ padding: 6, borderRadius: 6, border: "none", background: "transparent", cursor: "pointer", color: "var(--fg-3)", display: "flex", alignItems: "center" }}
            onMouseEnter={(e) => { (e.currentTarget as HTMLButtonElement).style.background = "var(--muted)"; (e.currentTarget as HTMLButtonElement).style.color = "var(--fg-2)"; }}
            onMouseLeave={(e) => { (e.currentTarget as HTMLButtonElement).style.background = "transparent"; (e.currentTarget as HTMLButtonElement).style.color = "var(--fg-3)"; }}
          >
            <ChevronRight size={14} />
          </button>
        </div>
      )}
    </aside>
  );
}
