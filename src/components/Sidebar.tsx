// Tsubasa (翼) — Sidebar Component (v3 — Manifesto Redesign)
// Icon-first layout, collapsible sidebar, glow active indicator.
// Categories section, bottom-anchored Settings + Stats.

import { useState } from "react";
import {
  Download,
  CheckCircle,
  Upload,
  Cloud,
  LayoutList,
  ChevronLeft,
  ChevronRight,
  Settings,
  BarChart3,
  Search,
  Zap,
  AlertCircle,
  Pause,
  Film,
  Tv,
  Music,
  Gamepad2,
  Monitor,
  FolderOpen,
  Globe2,
} from "lucide-react";
import { useTorrentStore } from "@/stores/torrent";
import { useUIStore } from "@/stores/ui";
import { useSearchStore } from "@/stores/search";
import { useAggregatorStore } from "@/stores/aggregator";
import { BUILT_IN_CATEGORIES } from "@/stores/categories";
import { SettingsPanel } from "@/components/SettingsPanel";
import "./Sidebar.css";

// ─── Types ────────────────────────────────────────────────

interface NavItemProps {
  label: string;
  icon: React.ReactNode;
  active: boolean;
  collapsed: boolean;
  onClick: () => void;
}

function NavItem({ label, icon, active, collapsed, onClick }: NavItemProps) {
  return (
    <button
      className="sidebar__item"
      data-active={active}
      title={collapsed ? label : undefined}
      onClick={onClick}
    >
      <span className="sidebar__item-icon">{icon}</span>
      <span className="sidebar__item-label">{label}</span>
    </button>
  );
}

// ─── Category icons ───────────────────────────────────────

const catIcons: Record<string, React.ReactNode> = {
  Movies: <Film size={16} strokeWidth={1.5} />,
  "TV Shows": <Tv size={16} strokeWidth={1.5} />,
  Music: <Music size={16} strokeWidth={1.5} />,
  Games: <Gamepad2 size={16} strokeWidth={1.5} />,
  Software: <Monitor size={16} strokeWidth={1.5} />,
  Other: <FolderOpen size={16} strokeWidth={1.5} />,
};

// ─── Main Sidebar ─────────────────────────────────────────

export function Sidebar() {
  const filter = useTorrentStore((s) => s.filter);
  const setFilter = useTorrentStore((s) => s.setFilter);
  const collapsed = useUIStore((s) => s.sidebarCollapsed);
  const toggleSidebar = useUIStore((s) => s.toggleSidebar);
  const setDetailPanelOpen = useUIStore((s) => s.setDetailPanelOpen);
  const setSearchOpen = useSearchStore((s) => s.setOpen);
  const setAggregatorOpen = useAggregatorStore((s) => s.setOpen);

  const [showSettings, setShowSettings] = useState(false);


  const navItems = [
    { label: "All", value: "all", icon: <LayoutList size={18} strokeWidth={1.5} /> },
    { label: "Downloading", value: "downloading", icon: <Download size={18} strokeWidth={1.5} /> },
    { label: "Seeding", value: "seeding", icon: <Upload size={18} strokeWidth={1.5} /> },
    { label: "Completed", value: "completed", icon: <CheckCircle size={18} strokeWidth={1.5} /> },
    { label: "Paused", value: "paused", icon: <Pause size={18} strokeWidth={1.5} /> },
    { label: "Error", value: "errored", icon: <AlertCircle size={18} strokeWidth={1.5} /> },
    { label: "Cloud", value: "cloud", icon: <Cloud size={18} strokeWidth={1.5} /> },
  ];

  return (
    <>
      <aside className="sidebar" data-collapsed={collapsed}>
        {/* ── Brand header ── */}
        <div className="sidebar__header">
          <div className="sidebar__brand">
            <div className="sidebar__logo">
              <Zap size={12} color="#fff" strokeWidth={2.5} />
            </div>
            <span className="sidebar__wordmark">Tsubasa</span>
          </div>
          <button
            className="sidebar__toggle sidebar__toggle--collapse"
            onClick={toggleSidebar}
            aria-label="Collapse sidebar"
          >
            <ChevronLeft size={14} />
          </button>
        </div>

        {/* ── Navigation ── */}
        <nav className="sidebar__nav">
          {/* Status filters */}
          {navItems.map((item) => (
            <NavItem
              key={item.value}
              label={item.label}
              icon={item.icon}
              active={filter === item.value}
              collapsed={collapsed}
              onClick={() => setFilter(item.value as any)}
            />
          ))}

          {/* Categories section — only when expanded and data exists */}
          {!collapsed && (
            <>
              <div className="sidebar__section-label">Categories</div>
              {BUILT_IN_CATEGORIES.map((cat) => (
                <NavItem
                  key={cat.name}
                  label={cat.name}
                  icon={catIcons[cat.name] ?? <FolderOpen size={16} strokeWidth={1.5} />}
                  active={filter === `cat:${cat.name}`}
                  collapsed={collapsed}
                  onClick={() => setFilter(`cat:${cat.name}` as any)}
                />
              ))}
              <NavItem
                label="Other"
                icon={<FolderOpen size={16} strokeWidth={1.5} />}
                active={filter === "cat:Other"}
                collapsed={collapsed}
                onClick={() => setFilter("cat:Other" as any)}
              />
            </>
          )}

          {/* Search */}
          <div className="sidebar__section-label">Tools</div>
          <NavItem
            label="Search"
            icon={<Search size={18} strokeWidth={1.5} />}
            active={false}
            collapsed={collapsed}
            onClick={() => setSearchOpen(true)}
          />
          <NavItem
            label="Plugins"
            icon={<Globe2 size={18} strokeWidth={1.5} />}
            active={false}
            collapsed={collapsed}
            onClick={() => setAggregatorOpen(true)}
          />

          {/* Spacer pushes bottom items down */}
          <div className="sidebar__spacer" />
        </nav>

        {/* ── Bottom-anchored items ── */}
        <div className="sidebar__bottom">
          <NavItem
            label="Stats"
            icon={<BarChart3 size={18} strokeWidth={1.5} />}
            active={false}
            collapsed={collapsed}
            onClick={() => { setDetailPanelOpen(true); useUIStore.getState().setDetailPanelTab('trackers'); }}
          />
          <NavItem
            label="Settings"
            icon={<Settings size={18} strokeWidth={1.5} />}
            active={showSettings}
            collapsed={collapsed}
            onClick={() => setShowSettings(!showSettings)}
          />
        </div>

        {/* ── Expand toggle (collapsed state) ── */}
        <div className="sidebar__expand">
          <button
            className="sidebar__toggle"
            onClick={toggleSidebar}
            aria-label="Expand sidebar"
          >
            <ChevronRight size={14} />
          </button>
        </div>
      </aside>

      {/* Settings Panel — rendered outside sidebar to overlay properly */}
      {showSettings && <SettingsPanel onClose={() => setShowSettings(false)} />}
    </>
  );
}
