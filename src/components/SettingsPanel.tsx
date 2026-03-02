// Tsubasa — Settings Panel Component
// Full settings UI with tabbed sections.
// Opens as a modal overlay triggered from the toolbar gear icon.

import React, { useState, useEffect, useCallback } from "react";
import {
  X,
  FolderOpen,
  Download,
  Upload,
  Cloud,
  Wifi,
  Info,
  Save,
  Loader2,
  HardDrive,
  Zap,
  Eye,
  EyeOff,
  CheckCircle,
  Palette,
  Sun,
  Moon,
  ListOrdered,
  Shield,
  Search,
} from "lucide-react";
import { open as dialogOpen } from "@tauri-apps/plugin-dialog";
import { getSettings, updateSettings, getAppInfo } from "@/lib/tauri";
import type { AppSettings, AppInfo, DownloadPolicy, QueueSettings, SeedingSettings, BitTorrentSettings } from "@/types";
import { motion } from "framer-motion";
import { useThemeStore, type Theme } from "@/stores/theme";
import { useSettingsStore } from "@/stores/settingsV2";

interface SettingsPanelProps { onClose: () => void; }
type SettingsTab = "general" | "bandwidth" | "cloud" | "network" | "appearance" | "about" | "queue" | "bittorrent" | "search_settings";

const TABS: { id: SettingsTab; label: string; icon: React.ReactNode }[] = [
  { id: "general", label: "General", icon: <FolderOpen size={13} /> },
  { id: "bandwidth", label: "Bandwidth", icon: <Download size={13} /> },
  { id: "bittorrent", label: "BitTorrent", icon: <Shield size={13} /> },
  { id: "queue", label: "Queue & Seeding", icon: <ListOrdered size={13} /> },
  { id: "search_settings", label: "Search", icon: <Search size={13} /> },
  { id: "cloud", label: "Cloud", icon: <Cloud size={13} /> },
  { id: "network", label: "Network", icon: <Wifi size={13} /> },
  { id: "appearance", label: "Appearance", icon: <Palette size={13} /> },
  { id: "about", label: "About", icon: <Info size={13} /> },
];

// ─── Shared sub-components ─────────────────────────────

function SectionTitle({ children }: { children: React.ReactNode }) {
  return (
    <div style={{ fontSize: 12, fontWeight: 600, color: "var(--fg)", marginBottom: 12, paddingBottom: 8, borderBottom: "1px solid var(--line)" }}>
      {children}
    </div>
  );
}

function FieldLabel({ children }: { children: React.ReactNode }) {
  return <label style={{ display: "block", fontSize: 11, color: "var(--fg-2)", marginBottom: 6, fontWeight: 500 }}>{children}</label>;
}

function FieldHint({ children }: { children: React.ReactNode }) {
  return <p style={{ fontSize: 10, color: "var(--fg-3)", marginTop: 5, lineHeight: 1.5 }}>{children}</p>;
}

function StyledInput({ ...props }: React.InputHTMLAttributes<HTMLInputElement>) {
  return (
    <input
      {...props}
      style={{
        width: "100%",
        padding: "7px 12px",
        borderRadius: 8,
        border: "1px solid var(--line)",
        background: "var(--overlay)",
        color: "var(--fg)",
        fontSize: 12,
        outline: "none",
        fontFamily: props.type === "number" ? "'JetBrains Mono', monospace" : "inherit",
        fontVariantNumeric: props.type === "number" ? "tabular-nums" : undefined,
        transition: "border-color 150ms ease, box-shadow 150ms ease",
        ...props.style,
      }}
      onFocus={(e) => {
        e.currentTarget.style.borderColor = "var(--accent)";
        e.currentTarget.style.boxShadow = "0 0 0 3px var(--accent-soft)";
        props.onFocus?.(e);
      }}
      onBlur={(e) => {
        e.currentTarget.style.borderColor = "var(--line)";
        e.currentTarget.style.boxShadow = "none";
        props.onBlur?.(e);
      }}
    />
  );
}

function ToggleSwitch({ checked, onChange }: { checked: boolean; onChange: (v: boolean) => void }) {
  return (
    <button
      onClick={() => onChange(!checked)}
      style={{
        position: "relative",
        width: 36,
        height: 20,
        borderRadius: 99,
        border: "none",
        background: checked ? "var(--accent)" : "var(--muted)",
        cursor: "pointer",
        flexShrink: 0,
        boxShadow: checked ? "0 0 8px var(--accent-glow)" : "none",
        transition: "background-color 150ms ease, box-shadow 150ms ease",
      }}
    >
      <div style={{
        position: "absolute",
        top: 2,
        left: checked ? 18 : 2,
        width: 16,
        height: 16,
        borderRadius: "50%",
        background: "#fff",
        boxShadow: "0 1px 3px rgba(0,0,0,0.3)",
        transition: "left 150ms cubic-bezier(0.4, 0, 0.2, 1)",
      }} />
    </button>
  );
}

// ─── Appearance Tab ─────────────────────────────────────

function AppearanceTab() {
  const theme = useThemeStore((s) => s.theme);
  const setTheme = useThemeStore((s) => s.setTheme);

  const themes: { value: Theme; label: string; description: string; icon: React.ReactNode; preview: string[] }[] = [
    { value: "black", label: "Deep Black", description: "Dark, focused, minimal.", icon: <Moon size={15} />, preview: ["#09090b", "#111114", "#6366f1"] },
    { value: "rose", label: "Rosé Pine", description: "Warm, muted with soft purple.", icon: <Palette size={15} />, preview: ["#191724", "#1f1d2e", "#c4a7e7"] },
    { value: "light", label: "Clean White", description: "Light, clean, professional.", icon: <Sun size={15} />, preview: ["#f6f6f9", "#ffffff", "#4f46e5"] },
  ];

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 20 }}>
      <div>
        <SectionTitle>Theme</SectionTitle>
        <p style={{ fontSize: 11, color: "var(--fg-3)", marginBottom: 16 }}>
          Choose a visual theme. Changes apply instantly across the entire app.
        </p>
        <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
          {themes.map((t) => {
            const active = theme === t.value;
            return (
              <button
                key={t.value}
                onClick={() => setTheme(t.value)}
                style={{
                  display: "flex",
                  alignItems: "center",
                  gap: 12,
                  padding: "12px 14px",
                  borderRadius: 10,
                  border: active ? "1px solid var(--accent)" : "1px solid var(--line)",
                  background: active ? "var(--accent-soft)" : "var(--overlay)",
                  cursor: "pointer",
                  textAlign: "left",
                  transition: "border-color 150ms ease, background-color 150ms ease, box-shadow 150ms ease",
                  boxShadow: active ? "0 0 0 1px var(--accent-soft), var(--shadow-glow)" : "none",
                }}
              >
                {/* Color swatches */}
                <div style={{ display: "flex", gap: 4, flexShrink: 0 }}>
                  {t.preview.map((color, i) => (
                    <div
                      key={i}
                      style={{
                        width: 18,
                        height: 18,
                        borderRadius: "50%",
                        background: color,
                        border: "1px solid rgba(255,255,255,0.12)",
                        boxShadow: i === 2 ? `0 0 6px ${color}55` : "none",
                      }}
                    />
                  ))}
                </div>
                <div style={{ flex: 1 }}>
                  <div style={{ fontSize: 12, fontWeight: 500, color: active ? "var(--accent)" : "var(--fg)" }}>{t.label}</div>
                  <div style={{ fontSize: 11, color: "var(--fg-3)", marginTop: 2 }}>{t.description}</div>
                </div>
                {active && <CheckCircle size={15} color="var(--accent)" style={{ flexShrink: 0 }} />}
              </button>
            );
          })}
        </div>
      </div>
    </div>
  );
}

// ─── General Tab ───────────────────────────────────────

function GeneralTab({ settings, update, onBrowse }: {
  settings: AppSettings;
  update: (patch: Partial<AppSettings>) => void;
  onBrowse: () => void;
}) {
  const policies: { value: DownloadPolicy; label: string; icon: React.ReactNode }[] = [
    { value: "local_only", label: "Local Only", icon: <HardDrive size={12} /> },
    { value: "cloud_only", label: "Cloud Only", icon: <Cloud size={12} /> },
    { value: "hybrid", label: "Hybrid", icon: <Zap size={12} /> },
  ];

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 24 }}>
      <div>
        <SectionTitle>Download Location</SectionTitle>
        <div style={{ display: "flex", gap: 8 }}>
          <StyledInput
            type="text"
            value={settings.download_dir}
            onChange={(e) => update({ download_dir: e.target.value })}
            placeholder="/home/user/Downloads"
            style={{ flex: 1 }}
          />
          <button
            onClick={onBrowse}
            className="btn-ghost"
            style={{ padding: "7px 14px", whiteSpace: "nowrap" }}
          >
            Browse
          </button>
        </div>
        <FieldHint>Where completed downloads will be saved.</FieldHint>
      </div>

      <div>
        <SectionTitle>Default Download Policy</SectionTitle>
        <div style={{ display: "flex", gap: 8 }}>
          {policies.map((p) => {
            const active = settings.default_policy === p.value;
            return (
              <button
                key={p.value}
                onClick={() => update({ default_policy: p.value })}
                style={{
                  flex: 1, display: "flex", alignItems: "center", justifyContent: "center", gap: 6,
                  padding: "8px 12px", borderRadius: 8,
                  border: active ? "1px solid var(--accent)" : "1px solid var(--line)",
                  background: active ? "var(--accent-soft)" : "var(--overlay)",
                  color: active ? "var(--accent)" : "var(--fg-2)",
                  fontSize: 12, cursor: "pointer",
                  transition: "all 150ms ease",
                }}
              >
                {p.icon} {p.label}
              </button>
            );
          })}
        </div>
        <FieldHint>Applied to new torrents by default. Override per-torrent.</FieldHint>
      </div>

      <div>
        <SectionTitle>Queue Limits</SectionTitle>
        <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr 1fr", gap: 12 }}>
          {[
            { label: "Max Active", key: "max_active_torrents" as const },
            { label: "Max Downloads", key: "max_active_downloads" as const },
            { label: "Max Seeds", key: "max_active_seeds" as const },
          ].map(({ label, key }) => (
            <div key={key}>
              <FieldLabel>{label}</FieldLabel>
              <StyledInput
                type="number"
                min={0}
                value={settings[key]}
                onChange={(e) => update({ [key]: parseInt(e.target.value) || 0 })}
              />
            </div>
          ))}
        </div>
        <FieldHint>0 = unlimited. Controls how many torrents run simultaneously.</FieldHint>
      </div>

      <div>
        <SectionTitle>Seeding</SectionTitle>
        <div style={{ maxWidth: 140 }}>
          <FieldLabel>Default Ratio Limit</FieldLabel>
          <StyledInput
            type="number"
            min={0}
            step={0.1}
            value={settings.default_ratio_limit}
            onChange={(e) => update({ default_ratio_limit: parseFloat(e.target.value) || 0 })}
          />
        </div>
        <FieldHint>Stop seeding after reaching this upload ratio. 0 = unlimited.</FieldHint>
      </div>
    </div>
  );
}

// ─── Bandwidth Tab ─────────────────────────────────────

function formatBandwidth(bytesPerSec: number): string {
  if (bytesPerSec >= 1048576) return `${(bytesPerSec / 1048576).toFixed(1)} MB/s`;
  if (bytesPerSec >= 1024) return `${(bytesPerSec / 1024).toFixed(0)} KB/s`;
  return `${bytesPerSec} B/s`;
}

function BandwidthTab({ settings, update }: { settings: AppSettings; update: (patch: Partial<AppSettings>) => void }) {
  const presets = [
    { label: "Unlimited", dl: 0, ul: 0 },
    { label: "1 MB/s", dl: 1048576, ul: 524288 },
    { label: "5 MB/s", dl: 5242880, ul: 2621440 },
    { label: "10 MB/s", dl: 10485760, ul: 5242880 },
    { label: "50 MB/s", dl: 52428800, ul: 26214400 },
  ];

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 24 }}>
      <div>
        <SectionTitle>Speed Limits</SectionTitle>
        <p style={{ fontSize: 11, color: "var(--fg-3)", marginBottom: 16 }}>Set to 0 for unlimited. Values in bytes per second.</p>
        <div style={{ display: "flex", flexDirection: "column", gap: 14 }}>
          {[
            { label: "Download Limit", key: "global_download_limit" as const, icon: <Download size={12} color="var(--green)" />, color: "var(--green)" },
            { label: "Upload Limit", key: "global_upload_limit" as const, icon: <Upload size={12} color="var(--accent)" />, color: "var(--accent)" },
          ].map(({ label, key, icon, color }) => (
            <div key={key}>
              <FieldLabel><span style={{ display: "flex", alignItems: "center", gap: 5 }}>{icon} {label} (bytes/s)</span></FieldLabel>
              <div style={{ display: "flex", alignItems: "center", gap: 10 }}>
                <StyledInput
                  type="number"
                  min={0}
                  value={settings[key]}
                  onChange={(e) => update({ [key]: parseInt(e.target.value) || 0 })}
                  style={{ maxWidth: 180 }}
                />
                <span style={{ fontSize: 11, color, fontFamily: "'JetBrains Mono', monospace", fontWeight: 500 }}>
                  {settings[key] === 0 ? "Unlimited" : formatBandwidth(settings[key])}
                </span>
              </div>
            </div>
          ))}
        </div>
      </div>

      <div>
        <SectionTitle>Quick Presets</SectionTitle>
        <div style={{ display: "flex", flexWrap: "wrap", gap: 6 }}>
          {presets.map((preset) => (
            <button
              key={preset.label}
              onClick={() => update({ global_download_limit: preset.dl, global_upload_limit: preset.ul })}
              className="btn-ghost"
              style={{ fontSize: 11, padding: "5px 12px" }}
            >
              {preset.label}
            </button>
          ))}
        </div>
        <FieldHint>Presets set download to listed value, upload to half.</FieldHint>
      </div>
    </div>
  );
}

// ─── Cloud Tab ─────────────────────────────────────────

function CloudTab({ settings, update }: { settings: AppSettings; update: (patch: Partial<AppSettings>) => void }) {
  const [showTorbox, setShowTorbox] = useState(false);
  const [showRd, setShowRd] = useState(false);

  const providerCard = (
    opts: {
      name: string;
      icon: React.ReactNode;
      badge: string;
      badgeColor: string;
      badgeBg: string;
      apiKey: string | null | undefined;
      show: boolean;
      setShow: (v: boolean) => void;
      hint: string;
      onChange: (v: string | null) => void;
    }
  ) => (
    <div style={{ padding: 14, borderRadius: 10, border: "1px solid var(--line)", background: "var(--overlay)" }}>
      <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: 10 }}>
        <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
          {opts.icon}
          <span style={{ fontSize: 12, fontWeight: 500, color: "var(--fg)" }}>{opts.name}</span>
          <span style={{ fontSize: 10, color: opts.badgeColor, background: opts.badgeBg, padding: "1px 6px", borderRadius: 99 }}>{opts.badge}</span>
        </div>
        {opts.apiKey ? (
          <span style={{ display: "flex", alignItems: "center", gap: 4, fontSize: 10, color: "var(--green)" }}>
            <CheckCircle size={10} /> Configured
          </span>
        ) : (
          <span style={{ fontSize: 10, color: "var(--fg-3)" }}>Not configured</span>
        )}
      </div>
      <div style={{ position: "relative" }}>
        <StyledInput
          type={opts.show ? "text" : "password"}
          value={opts.apiKey ?? ""}
          onChange={(e) => opts.onChange(e.target.value || null)}
          placeholder={`Enter ${opts.name} API key…`}
          style={{ paddingRight: 36 }}
        />
        <button
          onClick={() => opts.setShow(!opts.show)}
          style={{ position: "absolute", right: 10, top: "50%", transform: "translateY(-50%)", background: "none", border: "none", cursor: "pointer", color: "var(--fg-3)", display: "flex" }}
        >
          {opts.show ? <EyeOff size={12} /> : <Eye size={12} />}
        </button>
      </div>
      <FieldHint>{opts.hint}</FieldHint>
    </div>
  );

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 16 }}>
      <div>
        <SectionTitle>Cloud Debrid Providers</SectionTitle>
        <p style={{ fontSize: 11, color: "var(--fg-3)", marginBottom: 14 }}>
          Connect debrid services for instant cached downloads and cloud-based torrenting.
        </p>
      </div>
      {providerCard({
        name: "Torbox", icon: <Cloud size={13} color="var(--blue)" />,
        badge: "Primary", badgeColor: "var(--accent)", badgeBg: "var(--accent-soft)",
        apiKey: settings.torbox_api_key, show: showTorbox, setShow: setShowTorbox,
        hint: "Get your API key from torbox.app/settings.",
        onChange: (v) => update({ torbox_api_key: v }),
      })}
      {providerCard({
        name: "Real-Debrid", icon: <Cloud size={13} color="var(--amber)" />,
        badge: "Secondary", badgeColor: "var(--fg-3)", badgeBg: "var(--muted)",
        apiKey: settings.realdebrid_api_key, show: showRd, setShow: setShowRd,
        hint: "Get your API key from real-debrid.com/apitoken.",
        onChange: (v) => update({ realdebrid_api_key: v }),
      })}
    </div>
  );
}

// ─── Network Tab ───────────────────────────────────────

function NetworkTab({ settings, update }: { settings: AppSettings; update: (patch: Partial<AppSettings>) => void }) {
  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 24 }}>
      <div>
        <SectionTitle>Protocol Settings</SectionTitle>
        <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
          {[
            { key: "enable_dht" as const, label: "DHT (Distributed Hash Table)", desc: "Enables decentralized peer discovery. Recommended for most users." },
            { key: "enable_pex" as const, label: "PEX (Peer Exchange)", desc: "Share peer lists between connected clients for faster swarm discovery." },
          ].map(({ key, label, desc }) => (
            <div key={key} style={{ display: "flex", alignItems: "center", justifyContent: "space-between", padding: "12px 14px", borderRadius: 10, border: "1px solid var(--line)", background: "var(--overlay)" }}>
              <div>
                <div style={{ fontSize: 12, fontWeight: 500, color: "var(--fg)" }}>{label}</div>
                <div style={{ fontSize: 11, color: "var(--fg-3)", marginTop: 3 }}>{desc}</div>
              </div>
              <ToggleSwitch checked={settings[key]} onChange={(v) => update({ [key]: v })} />
            </div>
          ))}
        </div>
      </div>

      <div>
        <SectionTitle>Connection</SectionTitle>
        <div style={{ maxWidth: 160 }}>
          <FieldLabel>Listen Port</FieldLabel>
          <StyledInput
            type="number"
            min={1024}
            max={65535}
            value={settings.listen_port}
            onChange={(e) => update({ listen_port: parseInt(e.target.value) || 6881 })}
          />
        </div>
        <FieldHint>Port for incoming BitTorrent connections. Default: 6881.</FieldHint>
      </div>
    </div>
  );
}

// ─── Queue & Seeding Tab ───────────────────────────────

function QueueSeedingTab() {
  const { settings, updateQueue, updateSeeding } = useSettingsStore();
  if (!settings) return <p style={{ fontSize: 12, color: "var(--fg-3)" }}>Loading…</p>;
  const q = settings.queue;
  const s = settings.seeding;

  const updateQ = (patch: Partial<QueueSettings>) => updateQueue({ ...q, ...patch });
  const updateS = (patch: Partial<SeedingSettings>) => updateSeeding({ ...s, ...patch });

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 24 }}>
      <div>
        <SectionTitle>Active Limits</SectionTitle>
        <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr 1fr", gap: 12 }}>
          <div>
            <FieldLabel>Max Downloads</FieldLabel>
            <StyledInput type="number" min={1} max={100} value={q.max_active_downloads} onChange={(e) => updateQ({ max_active_downloads: parseInt(e.target.value) || 5 })} />
          </div>
          <div>
            <FieldLabel>Max Uploads</FieldLabel>
            <StyledInput type="number" min={1} max={100} value={q.max_active_uploads} onChange={(e) => updateQ({ max_active_uploads: parseInt(e.target.value) || 5 })} />
          </div>
          <div>
            <FieldLabel>Max Total</FieldLabel>
            <StyledInput type="number" min={1} max={200} value={q.max_active_total} onChange={(e) => updateQ({ max_active_total: parseInt(e.target.value) || 10 })} />
          </div>
        </div>
      </div>

      <div>
        <SectionTitle>Slow Torrent Detection</SectionTitle>
        <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", padding: "12px 14px", borderRadius: 10, border: "1px solid var(--line)", background: "var(--overlay)", marginBottom: 12 }}>
          <div>
            <div style={{ fontSize: 12, fontWeight: 500, color: "var(--fg)" }}>Exclude slow torrents from active count</div>
            <div style={{ fontSize: 11, color: "var(--fg-3)", marginTop: 3 }}>Slow torrents won't consume download slots</div>
          </div>
          <ToggleSwitch checked={q.exclude_slow_from_count} onChange={(v) => updateQ({ exclude_slow_from_count: v })} />
        </div>
        <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 12 }}>
          <div>
            <FieldLabel>DL Threshold (B/s)</FieldLabel>
            <StyledInput type="number" min={0} value={q.slow_torrent_dl_threshold} onChange={(e) => updateQ({ slow_torrent_dl_threshold: parseInt(e.target.value) || 0 })} />
          </div>
          <div>
            <FieldLabel>UL Threshold (B/s)</FieldLabel>
            <StyledInput type="number" min={0} value={q.slow_torrent_ul_threshold} onChange={(e) => updateQ({ slow_torrent_ul_threshold: parseInt(e.target.value) || 0 })} />
          </div>
        </div>
        <FieldHint>Torrents below both thresholds for the inactive period are considered slow.</FieldHint>
      </div>

      <div>
        <SectionTitle>Seeding Limits</SectionTitle>
        <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 12, marginBottom: 12 }}>
          <div>
            <FieldLabel>Global Ratio Limit</FieldLabel>
            <StyledInput type="number" min={0} step={0.1} value={s.global_ratio_limit ?? ""} placeholder="Unlimited" onChange={(e) => updateS({ global_ratio_limit: e.target.value ? parseFloat(e.target.value) : null })} />
          </div>
          <div>
            <FieldLabel>Time Limit (minutes)</FieldLabel>
            <StyledInput type="number" min={0} value={s.global_time_limit_mins ?? ""} placeholder="Unlimited" onChange={(e) => updateS({ global_time_limit_mins: e.target.value ? parseInt(e.target.value) : null })} />
          </div>
        </div>

        <div style={{ marginBottom: 12 }}>
          <FieldLabel>Action When Limit Reached</FieldLabel>
          <div style={{ display: "flex", gap: 8 }}>
            {(["Pause", "Remove", "RemoveWithFiles"] as const).map((action) => (
              <button
                key={action}
                onClick={() => updateS({ action_on_limit: action })}
                style={{
                  flex: 1, padding: "8px 12px", borderRadius: 8, border: "1px solid",
                  borderColor: s.action_on_limit === action ? "var(--accent)" : "var(--line)",
                  background: s.action_on_limit === action ? "var(--accent-soft)" : "var(--overlay)",
                  color: s.action_on_limit === action ? "var(--accent)" : "var(--fg-2)",
                  fontSize: 11, fontWeight: 500, cursor: "pointer", transition: "all 150ms ease",
                }}
              >
                {action === "RemoveWithFiles" ? "Remove + Files" : action}
              </button>
            ))}
          </div>
        </div>
        <FieldHint>Leave blank for unlimited. Per-torrent overrides take priority.</FieldHint>
      </div>
    </div>
  );
}

// ─── BitTorrent Tab ────────────────────────────────────

function BitTorrentTab() {
  const { settings, updateBitTorrent } = useSettingsStore();
  if (!settings) return <p style={{ fontSize: 12, color: "var(--fg-3)" }}>Loading…</p>;
  const bt = settings.bittorrent;

  const update = (patch: Partial<BitTorrentSettings>) => updateBitTorrent({ ...bt, ...patch });

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 24 }}>
      <div>
        <SectionTitle>Protocol Features</SectionTitle>
        <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
          {[
            { key: "enable_dht" as const, label: "DHT (Distributed Hash Table)", desc: "Decentralized peer discovery without trackers" },
            { key: "enable_pex" as const, label: "PEX (Peer Exchange)", desc: "Share peer lists between connected clients" },
            { key: "enable_lsd" as const, label: "LSD (Local Service Discovery)", desc: "Find peers on the same local network" },
            { key: "anonymous_mode" as const, label: "Anonymous Mode", desc: "Hide client name and version from peers" },
            { key: "sequential_download_default" as const, label: "Sequential Download Default", desc: "Download pieces in order (useful for media preview)" },
          ].map(({ key, label, desc }) => (
            <div key={key} style={{ display: "flex", alignItems: "center", justifyContent: "space-between", padding: "12px 14px", borderRadius: 10, border: "1px solid var(--line)", background: "var(--overlay)" }}>
              <div>
                <div style={{ fontSize: 12, fontWeight: 500, color: "var(--fg)" }}>{label}</div>
                <div style={{ fontSize: 11, color: "var(--fg-3)", marginTop: 3 }}>{desc}</div>
              </div>
              <ToggleSwitch checked={bt[key]} onChange={(v) => update({ [key]: v })} />
            </div>
          ))}
        </div>
      </div>

      <div>
        <SectionTitle>Encryption</SectionTitle>
        <div style={{ display: "flex", gap: 8 }}>
          {(["Forced", "Preferred", "Disabled"] as const).map((mode) => (
            <button
              key={mode}
              onClick={() => update({ encryption: mode })}
              style={{
                flex: 1, padding: "10px 14px", borderRadius: 8, border: "1px solid",
                borderColor: bt.encryption === mode ? "var(--accent)" : "var(--line)",
                background: bt.encryption === mode ? "var(--accent-soft)" : "var(--overlay)",
                color: bt.encryption === mode ? "var(--accent)" : "var(--fg-2)",
                fontSize: 12, fontWeight: 500, cursor: "pointer", transition: "all 150ms ease",
              }}
            >
              {mode}
            </button>
          ))}
        </div>
        <FieldHint>Forced: only encrypted connections. Preferred: try encryption first. Disabled: no encryption.</FieldHint>
      </div>
    </div>
  );
}

// ─── Search Settings Tab ───────────────────────────────

function SearchSettingsTab() {
  const { settings } = useSettingsStore();
  if (!settings) return <p style={{ fontSize: 12, color: "var(--fg-3)" }}>Loading…</p>;
  const search = settings.search;

  const availablePlugins = [
    { id: "piratebay", name: "PirateBay", desc: "General purpose, large database" },
    { id: "yts", name: "YTS", desc: "Movies only, high quality" },
    { id: "leet", name: "1337x", desc: "General purpose with categories" },
    { id: "nyaa", name: "Nyaa.si", desc: "Anime, manga, music" },
    { id: "torrentgalaxy", name: "TorrentGalaxy", desc: "General purpose" },
  ];

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 24 }}>
      <div>
        <SectionTitle>Search Plugins</SectionTitle>
        <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
          {availablePlugins.map(({ id, name, desc }) => (
            <div key={id} style={{ display: "flex", alignItems: "center", justifyContent: "space-between", padding: "12px 14px", borderRadius: 10, border: "1px solid var(--line)", background: "var(--overlay)" }}>
              <div>
                <div style={{ fontSize: 12, fontWeight: 500, color: "var(--fg)" }}>{name}</div>
                <div style={{ fontSize: 11, color: "var(--fg-3)", marginTop: 3 }}>{desc}</div>
              </div>
              <div style={{
                padding: "3px 8px", borderRadius: 6, fontSize: 10, fontWeight: 600,
                background: search.enabled_plugins.includes(id) ? "var(--green-soft)" : "var(--muted)",
                color: search.enabled_plugins.includes(id) ? "var(--green)" : "var(--fg-3)",
              }}>
                {search.enabled_plugins.includes(id) ? "Enabled" : "Disabled"}
              </div>
            </div>
          ))}
        </div>
        <FieldHint>Plugins search torrent sites directly — no API keys required.</FieldHint>
      </div>

      <div>
        <SectionTitle>Search Options</SectionTitle>
        <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 12 }}>
          <div>
            <FieldLabel>Max Results Per Plugin</FieldLabel>
            <StyledInput type="number" min={5} max={100} value={search.max_results_per_plugin} readOnly />
          </div>
          <div>
            <FieldLabel>Timeout (seconds)</FieldLabel>
            <StyledInput type="number" min={5} max={120} value={search.search_timeout_secs} readOnly />
          </div>
        </div>
      </div>
    </div>
  );
}

// ─── About Tab ─────────────────────────────────────────

function formatUptime(seconds: number): string {
  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  const s = Math.floor(seconds % 60);
  if (h > 0) return `${h}h ${m}m`;
  if (m > 0) return `${m}m ${s}s`;
  return `${s}s`;
}

function AboutTab({ appInfo }: { appInfo: AppInfo | null }) {
  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 24 }}>
      <div style={{ display: "flex", flexDirection: "column", alignItems: "center", textAlign: "center", padding: "8px 0 16px" }}>
        <div style={{ width: 52, height: 52, borderRadius: 14, background: "var(--gradient-accent)", display: "flex", alignItems: "center", justifyContent: "center", marginBottom: 14, boxShadow: "0 4px 16px var(--accent-glow)" }}>
          <Zap size={22} color="#fff" />
        </div>
        <h3 style={{ fontSize: 16, fontWeight: 600, color: "var(--fg)", letterSpacing: "-0.3px" }}>Tsubasa</h3>
        <p style={{ fontSize: 12, color: "var(--fg-3)", marginTop: 4 }}>A modern, premium BitTorrent client with cloud integration.</p>
      </div>

      {appInfo ? (
        <div style={{ padding: "14px 16px", borderRadius: 10, border: "1px solid var(--line)", background: "var(--overlay)", display: "flex", flexDirection: "column", gap: 8 }}>
          {[
            { label: "Version", value: appInfo.version, color: undefined },
            { label: "Engine", value: appInfo.engine_ready ? "Ready" : "Starting…", color: appInfo.engine_ready ? "var(--green)" : "var(--amber)" },
            { label: "Uptime", value: formatUptime(appInfo.uptime_seconds), color: undefined },
            { label: "Backend", value: "librqbit 8.1.1", color: undefined },
            { label: "Framework", value: "Tauri v2", color: undefined },
          ].map(({ label, value, color }) => (
            <div key={label} style={{ display: "flex", justifyContent: "space-between", paddingBottom: 6, borderBottom: "1px solid var(--line-subtle)" }}>
              <span style={{ fontSize: 12, color: "var(--fg-2)" }}>{label}</span>
              <span style={{ fontSize: 12, color: color ?? "var(--fg)", fontFamily: "'JetBrains Mono', monospace", fontWeight: 500 }}>{value}</span>
            </div>
          ))}
        </div>
      ) : (
        <p style={{ textAlign: "center", fontSize: 12, color: "var(--fg-3)" }}>Could not load application info.</p>
      )}
    </div>
  );
}

// ─── Main Settings Panel ───────────────────────────────

export function SettingsPanel({ onClose }: SettingsPanelProps) {
  const [tab, setTab] = useState<SettingsTab>("general");
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [appInfo, setAppInfo] = useState<AppInfo | null>(null);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [saved, setSaved] = useState(false);
  const [dirty, setDirty] = useState(false);

  // Load v2 settings on mount
  const settingsV2 = useSettingsStore();
  useEffect(() => {
    Promise.all([getSettings(), getAppInfo()])
      .then(([s, info]) => { setSettings(s); setAppInfo(info); })
      .catch((err) => console.error("Failed to load settings:", err))
      .finally(() => setLoading(false));
    settingsV2.load();
  }, []);

  useEffect(() => {
    const handleKey = (e: KeyboardEvent) => { if (e.key === "Escape") onClose(); };
    window.addEventListener("keydown", handleKey);
    return () => window.removeEventListener("keydown", handleKey);
  }, [onClose]);

  const update = useCallback((patch: Partial<AppSettings>) => {
    if (!settings) return;
    setSettings({ ...settings, ...patch });
    setDirty(true);
    setSaved(false);
  }, [settings]);

  const handleSave = async () => {
    if (!settings || saving) return;
    setSaving(true);
    try {
      await updateSettings(settings);
      setDirty(false);
      setSaved(true);
      setTimeout(() => setSaved(false), 2000);
    } catch (err) { console.error("Failed to save settings:", err); }
    finally { setSaving(false); }
  };

  const handleBrowseDir = useCallback(async () => {
    try {
      const selected = await dialogOpen({ directory: true, defaultPath: settings?.download_dir || undefined });
      if (selected && typeof selected === "string") update({ download_dir: selected });
    } catch (err) { console.error("Failed to open directory picker:", err); }
  }, [settings, update]);

  return (
    <motion.div
      style={{ position: "fixed", inset: 0, zIndex: 50, display: "flex", alignItems: "center", justifyContent: "center", background: "rgba(0,0,0,0.65)", backdropFilter: "blur(6px)" }}
      onClick={(e) => { if (e.target === e.currentTarget) onClose(); }}
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      transition={{ duration: 0.15 }}
    >
      <motion.div
        style={{
          width: 660,
          maxHeight: "82vh",
          background: "var(--surface)",
          border: "1px solid var(--line-strong)",
          borderRadius: 14,
          boxShadow: "var(--shadow-lg), 0 0 40px rgba(0,0,0,0.4)",
          overflow: "hidden",
          display: "flex",
          flexDirection: "column",
        }}
        initial={{ opacity: 0, y: -16, scale: 0.97 }}
        animate={{ opacity: 1, y: 0, scale: 1 }}
        exit={{ opacity: 0, y: -16, scale: 0.97 }}
        transition={{ duration: 0.2, ease: [0.25, 0.46, 0.45, 0.94] }}
      >
        {/* Header */}
        <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", padding: "14px 20px", borderBottom: "1px solid var(--line)" }}>
          <h2 style={{ fontSize: 14, fontWeight: 600, color: "var(--fg)", letterSpacing: "-0.2px" }}>Settings</h2>
          <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
            {dirty && (
              <button
                onClick={handleSave}
                disabled={saving}
                className="btn-primary"
                style={{ height: 30, padding: "0 14px", fontSize: 12 }}
              >
                {saving ? <Loader2 size={12} style={{ animation: "spin 1s linear infinite" }} /> : saved ? <CheckCircle size={12} /> : <Save size={12} />}
                {saving ? "Saving…" : saved ? "Saved" : "Save"}
              </button>
            )}
            {saved && !dirty && (
              <span style={{ display: "flex", alignItems: "center", gap: 4, fontSize: 11, color: "var(--green)" }}>
                <CheckCircle size={12} /> Saved
              </span>
            )}
            <button
              onClick={onClose}
              className="transition-colors-fast"
              style={{ width: 30, height: 30, borderRadius: 7, border: "none", background: "transparent", cursor: "pointer", color: "var(--fg-3)", display: "flex", alignItems: "center", justifyContent: "center" }}
              onMouseEnter={(e) => { (e.currentTarget as HTMLButtonElement).style.background = "var(--muted)"; (e.currentTarget as HTMLButtonElement).style.color = "var(--fg-2)"; }}
              onMouseLeave={(e) => { (e.currentTarget as HTMLButtonElement).style.background = "transparent"; (e.currentTarget as HTMLButtonElement).style.color = "var(--fg-3)"; }}
            >
              <X size={15} />
            </button>
          </div>
        </div>

        {/* Body: Tabs + Content */}
        <div style={{ display: "flex", flex: 1, minHeight: 0 }}>
          {/* Tab sidebar */}
          <div style={{ width: 152, flexShrink: 0, borderRight: "1px solid var(--line)", background: "var(--base)", padding: "8px 0" }}>
            {TABS.map((t) => {
              const active = tab === t.id;
              return (
                <button
                  key={t.id}
                  onClick={() => setTab(t.id)}
                  className="transition-colors-fast"
                  style={{
                    width: "100%",
                    display: "flex",
                    alignItems: "center",
                    gap: 8,
                    padding: "8px 16px",
                    border: "none",
                    background: active ? "var(--accent-soft)" : "transparent",
                    color: active ? "var(--accent)" : "var(--fg-2)",
                    fontSize: 12,
                    fontWeight: active ? 500 : 400,
                    cursor: "pointer",
                    textAlign: "left",
                    position: "relative",
                  }}
                  onMouseEnter={(e) => { if (!active) { (e.currentTarget as HTMLButtonElement).style.background = "var(--muted)"; (e.currentTarget as HTMLButtonElement).style.color = "var(--fg)"; } }}
                  onMouseLeave={(e) => { if (!active) { (e.currentTarget as HTMLButtonElement).style.background = "transparent"; (e.currentTarget as HTMLButtonElement).style.color = "var(--fg-2)"; } }}
                >
                  {/* Left accent bar */}
                  {active && (
                    <span style={{ position: "absolute", left: 0, top: "50%", transform: "translateY(-50%)", width: 3, height: 14, background: "var(--accent)", borderRadius: "0 2px 2px 0", boxShadow: "0 0 6px var(--accent-glow)" }} />
                  )}
                  {t.icon}
                  {t.label}
                </button>
              );
            })}
          </div>

          {/* Content */}
          <div style={{ flex: 1, overflowY: "auto", padding: "20px 24px" }}>
            {loading ? (
              <div style={{ display: "flex", alignItems: "center", justifyContent: "center", height: 160 }}>
                <Loader2 size={20} color="var(--fg-3)" style={{ animation: "spin 1s linear infinite" }} />
              </div>
            ) : !settings ? (
              <p style={{ fontSize: 12, color: "var(--red)" }}>Failed to load settings.</p>
            ) : (
              <>
                {tab === "general" && <GeneralTab settings={settings} update={update} onBrowse={handleBrowseDir} />}
                {tab === "bandwidth" && <BandwidthTab settings={settings} update={update} />}
                {tab === "cloud" && <CloudTab settings={settings} update={update} />}
                {tab === "bittorrent" && <BitTorrentTab />}
                {tab === "queue" && <QueueSeedingTab />}
                {tab === "search_settings" && <SearchSettingsTab />}
                {tab === "network" && <NetworkTab settings={settings} update={update} />}
                {tab === "appearance" && <AppearanceTab />}
                {tab === "about" && <AboutTab appInfo={appInfo} />}
              </>
            )}
          </div>
        </div>
      </motion.div>
    </motion.div>
  );
}
