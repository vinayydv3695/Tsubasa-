// Tsubasa (翼) — Sidebar Component (v3 — Manifesto Redesign)
// Pure navigation. No counters, no collapsible sections, no clutter.
// Icon-first layout, collapsible sidebar, glow active indicator.
// Bottom-anchored: Settings + Stats.

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
} from "lucide-react";
import { useTorrentStore } from "@/stores/torrent";
import { useUIStore } from "@/stores/ui";
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

// ─── Main Sidebar ─────────────────────────────────────────

export function Sidebar() {
  const filter = useTorrentStore((s) => s.filter);
  const setFilter = useTorrentStore((s) => s.setFilter);
  const collapsed = useUIStore((s) => s.sidebarCollapsed);
  const toggleSidebar = useUIStore((s) => s.toggleSidebar);

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

        {/* Search */}
        <div className="sidebar__section-label">Search</div>
        <NavItem
          label="Search"
          icon={<Search size={18} strokeWidth={1.5} />}
          active={filter === "search" as any}
          collapsed={collapsed}
          onClick={() => setFilter("search" as any)}
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
          onClick={() => {/* TODO: open stats panel */ }}
        />
        <NavItem
          label="Settings"
          icon={<Settings size={18} strokeWidth={1.5} />}
          active={false}
          collapsed={collapsed}
          onClick={() => {/* TODO: open settings modal */ }}
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
  );
}
