// Tsubasa — Detail Panel Component
// Shows detailed info for the selected torrent with tabbed views:
// General, Files, Peers, Trackers, Pieces, Cloud.

import React, { useEffect, useState, useCallback } from "react";
import {
  X,
  FileText,
  Users,
  Radio,
  Grid3x3,
  Cloud,
  Info,
  File,
  Folder,
  AlertCircle,
  RefreshCw,
  Download,
  CheckCircle,
  Clock,
  XCircle,
  Loader2,
  Activity,
} from "lucide-react";
import { useTorrentStore } from "@/stores/torrent";
import { useUIStore } from "@/stores/ui";
import { SpeedGraph } from "@/components/SpeedGraph";
import {
  formatBytes,
  formatSpeed,
  formatRatio,
  formatProgress,
} from "@/lib/utils";
import {
  getTorrentFiles,
  getTorrentPeers,
  getTorrentTrackers,
  getCloudStatus,
  cloudCheckCached,
  cloudCheckStatus,
  cloudGetLinks,
  cloudAddTorrent,
} from "@/lib/tauri";
import type {
  TorrentFileInfo,
  TorrentPeerInfo,
  TorrentTrackerInfo,
  CloudProviderStatus,
  DirectLink,
  CacheCheckResult,
} from "@/types";

const TABS = [
  { id: "general", label: "General", icon: <Info size={13} /> },
  { id: "files", label: "Files", icon: <FileText size={13} /> },
  { id: "peers", label: "Peers", icon: <Users size={13} /> },
  { id: "trackers", label: "Trackers", icon: <Radio size={13} /> },
  { id: "speed", label: "Speed", icon: <Activity size={13} /> },
  { id: "pieces", label: "Pieces", icon: <Grid3x3 size={13} /> },
  { id: "cloud", label: "Cloud", icon: <Cloud size={13} /> },
];

// ─── Shared tab helpers ──────────────────────────────────

function TabEmpty({ children }: { children: React.ReactNode }) {
  return (
    <div style={{ flex: 1, display: "flex", alignItems: "center", justifyContent: "center", fontSize: 12, color: "var(--fg-3)" }}>
      {children}
    </div>
  );
}

function TabError({ message }: { message: string }) {
  return (
    <div style={{ flex: 1, display: "flex", alignItems: "center", justifyContent: "center", gap: 6, fontSize: 12, color: "var(--amber)" }}>
      <AlertCircle size={13} /> {message}
    </div>
  );
}

// ─── General Tab ────────────────────────────────────────

function GeneralTab() {
  const selectedId = useTorrentStore((s) => s.selectedTorrentId);
  const torrents = useTorrentStore((s) => s.torrents);
  const torrent = selectedId ? torrents.get(selectedId) : undefined;
  if (!torrent) return null;

  const groups = [
    {
      title: "Transfer",
      items: [
        { label: "Downloaded", value: formatBytes(torrent.downloaded_bytes) },
        { label: "Uploaded", value: formatBytes(torrent.uploaded_bytes) },
        { label: "Ratio", value: formatRatio(torrent.ratio) },
        { label: "Download Speed", value: formatSpeed(torrent.download_speed) },
        { label: "Upload Speed", value: formatSpeed(torrent.upload_speed) },
        { label: "Peers", value: `${torrent.seeds_connected} seeds · ${torrent.peers_connected} peers` },
      ],
    },
    {
      title: "Info",
      items: [
        { label: "Total Size", value: formatBytes(torrent.total_bytes) },
        { label: "Policy", value: torrent.policy.replace("_", " ") },
        { label: "Added", value: new Date(torrent.added_at).toLocaleString() },
        { label: "Save Path", value: torrent.save_path },
        { label: "Info Hash", value: torrent.info_hash },
        ...(torrent.error_message ? [{ label: "Error", value: torrent.error_message }] : []),
      ],
    },
  ];

  return (
    <div style={{ display: "flex", gap: 0, height: "100%" }}>
      {groups.map((group, gi) => (
        <div
          key={group.title}
          style={{
            flex: 1,
            borderRight: gi < groups.length - 1 ? "1px solid var(--line)" : "none",
            padding: "10px 14px",
            display: "flex",
            flexDirection: "column",
            gap: 1,
            overflow: "hidden",
          }}
        >
          <div style={{ fontSize: 10, fontWeight: 600, color: "var(--fg-3)", textTransform: "uppercase", letterSpacing: "0.6px", marginBottom: 8 }}>
            {group.title}
          </div>
          {group.items.map((item) => (
            <div
              key={item.label}
              style={{ display: "grid", gridTemplateColumns: "110px 1fr", gap: 8, padding: "3px 0", borderBottom: "1px solid var(--line-subtle)" }}
            >
              <span style={{ fontSize: 11, color: "var(--fg-3)", alignSelf: "center" }}>{item.label}</span>
              <span style={{ fontSize: 11, color: "var(--fg-2)", fontFamily: "'JetBrains Mono', monospace", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                {item.value}
              </span>
            </div>
          ))}
        </div>
      ))}
    </div>
  );
}

// ─── Files Tab ──────────────────────────────────────────

function FileProgressBar({ progress }: { progress: number }) {
  return (
    <div style={{ width: "100%", height: 3, borderRadius: 99, background: "var(--muted)", marginTop: 3, overflow: "visible" }}>
      <div style={{
        height: "100%",
        width: `${Math.min(progress * 100, 100)}%`,
        borderRadius: 99,
        background: progress >= 1 ? "var(--green)" : "var(--accent)",
        boxShadow: progress >= 1 ? "0 0 4px var(--green-glow)" : "0 0 4px var(--accent-glow)",
        transition: "width 300ms ease",
      }} />
    </div>
  );
}

function FilesTab() {
  const selectedId = useTorrentStore((s) => s.selectedTorrentId);
  const [files, setFiles] = useState<TorrentFileInfo[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  const loadFiles = useCallback(async () => {
    if (!selectedId) return;
    setLoading(true); setError(null);
    try { setFiles(await getTorrentFiles(selectedId)); }
    catch (e) { setError(String(e)); setFiles([]); }
    finally { setLoading(false); }
  }, [selectedId]);

  useEffect(() => {
    loadFiles();
    const interval = setInterval(loadFiles, 2000);
    return () => clearInterval(interval);
  }, [loadFiles]);

  if (loading && files.length === 0) return <TabEmpty>Loading files…</TabEmpty>;
  if (error) return <TabError message={error} />;
  if (files.length === 0) return <TabEmpty>No file information available (torrent may still be loading metadata)</TabEmpty>;

  return (
    <div style={{ display: "flex", flexDirection: "column", height: "100%" }}>
      {/* Header */}
      <div style={{
        display: "grid",
        gridTemplateColumns: "1fr 72px 72px 52px",
        gap: 8, padding: "5px 12px",
        borderBottom: "1px solid var(--line)",
        fontSize: 10, fontWeight: 600, color: "var(--fg-3)",
        textTransform: "uppercase", letterSpacing: "0.6px",
      }}>
        <span>Name</span>
        <span style={{ textAlign: "right" }}>Size</span>
        <span style={{ textAlign: "right" }}>Done</span>
        <span style={{ textAlign: "right" }}>%</span>
      </div>

      <div style={{ flex: 1, overflowY: "auto" }}>
        {files.map((file) => {
          const isDir = file.path.includes("/") || file.path.includes("\\");
          const Icon = isDir ? Folder : File;
          return (
            <div
              key={file.index}
              style={{ display: "grid", gridTemplateColumns: "1fr 72px 72px 52px", gap: 8, alignItems: "center", padding: "5px 12px", borderBottom: "1px solid var(--line-subtle)" }}
              onMouseEnter={(e) => { (e.currentTarget as HTMLDivElement).style.background = "var(--muted)"; }}
              onMouseLeave={(e) => { (e.currentTarget as HTMLDivElement).style.background = "transparent"; }}
            >
              <div style={{ display: "flex", alignItems: "flex-start", gap: 7, minWidth: 0 }}>
                <Icon size={12} style={{ flexShrink: 0, color: "var(--fg-3)", marginTop: 2 }} />
                <div style={{ minWidth: 0, flex: 1 }}>
                  <span style={{ fontSize: 11, color: "var(--fg-2)", display: "block", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                    {file.name}
                  </span>
                  {file.path !== file.name && (
                    <span style={{ fontSize: 10, color: "var(--fg-3)", display: "block", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                      {file.path}
                    </span>
                  )}
                  <FileProgressBar progress={file.progress} />
                </div>
              </div>
              <span style={{ fontSize: 11, color: "var(--fg-3)", textAlign: "right", fontFamily: "'JetBrains Mono', monospace" }}>{formatBytes(file.size)}</span>
              <span style={{ fontSize: 11, color: "var(--fg-3)", textAlign: "right", fontFamily: "'JetBrains Mono', monospace" }}>{formatBytes(file.downloaded)}</span>
              <span style={{ fontSize: 11, textAlign: "right", fontFamily: "'JetBrains Mono', monospace", color: file.progress >= 1 ? "var(--green)" : "var(--fg-2)" }}>
                {formatProgress(file.progress)}
              </span>
            </div>
          );
        })}
      </div>

      <div style={{ display: "flex", justifyContent: "space-between", padding: "4px 12px", borderTop: "1px solid var(--line)", fontSize: 10, color: "var(--fg-3)" }}>
        <span>{files.length} file{files.length !== 1 ? "s" : ""}</span>
        <span style={{ fontFamily: "'JetBrains Mono', monospace" }}>{formatBytes(files.reduce((acc, f) => acc + f.size, 0))} total</span>
      </div>
    </div>
  );
}

// ─── Peers Tab ──────────────────────────────────────────

function PeersTab() {
  const selectedId = useTorrentStore((s) => s.selectedTorrentId);
  const [peers, setPeers] = useState<TorrentPeerInfo[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  const loadPeers = useCallback(async () => {
    if (!selectedId) return;
    setLoading(true); setError(null);
    try { setPeers(await getTorrentPeers(selectedId)); }
    catch (e) { setError(String(e)); setPeers([]); }
    finally { setLoading(false); }
  }, [selectedId]);

  useEffect(() => {
    loadPeers();
    const interval = setInterval(loadPeers, 2000);
    return () => clearInterval(interval);
  }, [loadPeers]);

  if (loading && peers.length === 0) return <TabEmpty>Loading peers…</TabEmpty>;
  if (error) return <TabError message={error} />;
  if (peers.length === 0) return <TabEmpty>No peers connected</TabEmpty>;

  const stateColor = (s: string) =>
    s === "live" ? "var(--green)"
      : s === "connecting" ? "var(--amber)"
        : "var(--fg-3)";

  return (
    <div style={{ display: "flex", flexDirection: "column", height: "100%" }}>
      <div style={{ display: "grid", gridTemplateColumns: "1fr 72px 80px 80px 44px", gap: 8, padding: "5px 12px", borderBottom: "1px solid var(--line)", fontSize: 10, fontWeight: 600, color: "var(--fg-3)", textTransform: "uppercase", letterSpacing: "0.6px" }}>
        <span>Address</span><span>State</span>
        <span style={{ textAlign: "right" }}>Downloaded</span>
        <span style={{ textAlign: "right" }}>Uploaded</span>
        <span style={{ textAlign: "right" }}>Errs</span>
      </div>
      <div style={{ flex: 1, overflowY: "auto" }}>
        {peers.map((peer) => (
          <div
            key={peer.address}
            style={{ display: "grid", gridTemplateColumns: "1fr 72px 80px 80px 44px", gap: 8, alignItems: "center", padding: "5px 12px", borderBottom: "1px solid var(--line-subtle)" }}
            onMouseEnter={(e) => { (e.currentTarget as HTMLDivElement).style.background = "var(--muted)"; }}
            onMouseLeave={(e) => { (e.currentTarget as HTMLDivElement).style.background = "transparent"; }}
          >
            <span style={{ fontSize: 11, color: "var(--fg-2)", fontFamily: "'JetBrains Mono', monospace", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>{peer.address}</span>
            <div style={{ display: "flex", alignItems: "center", gap: 5 }}>
              <span style={{ width: 6, height: 6, borderRadius: "50%", background: stateColor(peer.state), flexShrink: 0, boxShadow: peer.state === "live" ? "0 0 4px var(--green-glow)" : "none" }} />
              <span style={{ fontSize: 10, color: "var(--fg-3)", textTransform: "capitalize" }}>{peer.state}</span>
            </div>
            <span style={{ fontSize: 11, color: "var(--fg-3)", textAlign: "right", fontFamily: "'JetBrains Mono', monospace" }}>{formatBytes(peer.downloaded_bytes)}</span>
            <span style={{ fontSize: 11, color: "var(--fg-3)", textAlign: "right", fontFamily: "'JetBrains Mono', monospace" }}>{formatBytes(peer.uploaded_bytes)}</span>
            <span style={{ fontSize: 11, textAlign: "right", fontFamily: "'JetBrains Mono', monospace", color: peer.errors > 0 ? "var(--red)" : "var(--fg-3)" }}>{peer.errors}</span>
          </div>
        ))}
      </div>
      <div style={{ display: "flex", justifyContent: "space-between", padding: "4px 12px", borderTop: "1px solid var(--line)", fontSize: 10, color: "var(--fg-3)" }}>
        <span>{peers.length} peer{peers.length !== 1 ? "s" : ""}</span>
        <span>{peers.filter((p) => p.state === "live").length} live</span>
      </div>
    </div>
  );
}

// ─── Trackers Tab ───────────────────────────────────────

function TrackersTab() {
  const selectedId = useTorrentStore((s) => s.selectedTorrentId);
  const [trackers, setTrackers] = useState<TorrentTrackerInfo[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  const loadTrackers = useCallback(async () => {
    if (!selectedId) return;
    setLoading(true); setError(null);
    try { setTrackers(await getTorrentTrackers(selectedId)); }
    catch (e) { setError(String(e)); setTrackers([]); }
    finally { setLoading(false); }
  }, [selectedId]);

  useEffect(() => {
    loadTrackers();
    const interval = setInterval(loadTrackers, 10000);
    return () => clearInterval(interval);
  }, [loadTrackers]);

  if (loading && trackers.length === 0) return <TabEmpty>Loading trackers…</TabEmpty>;
  if (error) return <TabError message={error} />;
  if (trackers.length === 0) return <TabEmpty>No trackers (DHT/PEX only)</TabEmpty>;

  return (
    <div style={{ display: "flex", flexDirection: "column", height: "100%" }}>
      <div style={{ display: "grid", gridTemplateColumns: "1fr 96px", gap: 8, padding: "5px 12px", borderBottom: "1px solid var(--line)", fontSize: 10, fontWeight: 600, color: "var(--fg-3)", textTransform: "uppercase", letterSpacing: "0.6px" }}>
        <span>URL</span><span style={{ textAlign: "right" }}>Status</span>
      </div>
      <div style={{ flex: 1, overflowY: "auto" }}>
        {trackers.map((tracker, i) => (
          <div
            key={`${tracker.url}-${i}`}
            style={{ display: "grid", gridTemplateColumns: "1fr 96px", gap: 8, alignItems: "center", padding: "5px 12px", borderBottom: "1px solid var(--line-subtle)" }}
            onMouseEnter={(e) => { (e.currentTarget as HTMLDivElement).style.background = "var(--muted)"; }}
            onMouseLeave={(e) => { (e.currentTarget as HTMLDivElement).style.background = "transparent"; }}
          >
            <span style={{ fontSize: 11, color: "var(--fg-2)", fontFamily: "'JetBrains Mono', monospace", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>{tracker.url}</span>
            <span style={{ fontSize: 10, color: "var(--fg-3)", textAlign: "right", textTransform: "capitalize" }}>{tracker.status}</span>
          </div>
        ))}
      </div>
      <div style={{ padding: "4px 12px", borderTop: "1px solid var(--line)", fontSize: 10, color: "var(--fg-3)" }}>
        {trackers.length} tracker{trackers.length !== 1 ? "s" : ""}
      </div>
    </div>
  );
}

// ─── Cloud Tab ───────────────────────────────────────────

function parseCloudStatus(status: import("@/types").CloudStatus): { label: string; progress?: number; reason?: string } {
  if (typeof status === "string") return { label: status };
  if ("downloading" in status) return { label: "downloading", progress: status.downloading.progress };
  if ("failed" in status) return { label: "failed", reason: status.failed.reason };
  return { label: "unknown" };
}

function CloudStatusIcon({ label }: { label: string }) {
  switch (label) {
    case "completed":
    case "cached": return <CheckCircle size={13} style={{ color: "var(--green)" }} />;
    case "downloading": return <Loader2 size={13} style={{ color: "var(--accent)", animation: "spin 1s linear infinite" }} />;
    case "queued": return <Clock size={13} style={{ color: "var(--amber)" }} />;
    case "failed": return <XCircle size={13} style={{ color: "var(--red)" }} />;
    default: return <Cloud size={13} style={{ color: "var(--fg-3)" }} />;
  }
}

function CloudTab() {
  const selectedId = useTorrentStore((s) => s.selectedTorrentId);
  const torrents = useTorrentStore((s) => s.torrents);
  const torrent = selectedId ? torrents.get(selectedId) : undefined;

  const [providers, setProviders] = useState<CloudProviderStatus[]>([]);
  const [cacheResults, setCacheResults] = useState<CacheCheckResult>([]);
  const [cloudSubmission, setCloudSubmission] = useState<{ cloudId: string; provider: string } | null>(null);
  const [cloudStatus, setCloudStatus] = useState<import("@/types").CloudStatus | null>(null);
  const [links, setLinks] = useState<DirectLink[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [actionLoading, setActionLoading] = useState<string | null>(null);

  const loadProviders = useCallback(async () => {
    try { setProviders(await getCloudStatus()); }
    catch (e) { console.error("Failed to load cloud status:", e); }
  }, []);

  const loadCacheStatus = useCallback(async () => {
    if (!torrent?.info_hash) return;
    try { setCacheResults(await cloudCheckCached(torrent.info_hash)); }
    catch (e) { console.error("Failed to check cache:", e); }
  }, [torrent?.info_hash]);

  useEffect(() => { loadProviders(); loadCacheStatus(); }, [loadProviders, loadCacheStatus]);

  useEffect(() => {
    if (!cloudSubmission) return;
    const poll = async () => {
      try {
        const status = await cloudCheckStatus(cloudSubmission.cloudId, cloudSubmission.provider);
        setCloudStatus(status);
        const parsed = parseCloudStatus(status);
        if (parsed.label === "completed" || parsed.label === "cached") {
          try { setLinks(await cloudGetLinks(cloudSubmission.cloudId, cloudSubmission.provider)); }
          catch (e) { console.error("Failed to get links:", e); }
        }
      } catch (e) { console.error("Failed to check cloud status:", e); }
    };
    poll();
    const interval = setInterval(poll, 3000);
    return () => clearInterval(interval);
  }, [cloudSubmission]);

  const handleSendToCloud = async (providerName: string) => {
    if (!torrent) return;
    setActionLoading(providerName); setError(null);
    try {
      const result = await cloudAddTorrent(torrent.info_hash, providerName);
      setCloudSubmission({ cloudId: result.cloud_id, provider: result.provider });
    } catch (e) { setError(String(e)); }
    finally { setActionLoading(null); }
  };

  if (!torrent) return null;
  const configuredProviders = providers.filter((p) => p.configured);
  const hasProviders = configuredProviders.length > 0;

  const rowStyle: React.CSSProperties = { display: "flex", alignItems: "center", gap: 8, padding: "6px 10px", borderRadius: 7, background: "var(--overlay)", border: "1px solid var(--line)" };

  return (
    <div style={{ padding: "10px 14px", display: "flex", flexDirection: "column", gap: 12, overflowY: "auto", height: "100%" }}>
      <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
        <span style={{ fontSize: 11, fontWeight: 500, color: "var(--fg-2)" }}>Cloud Debrid</span>
        <button
          onClick={async () => { setLoading(true); await Promise.all([loadProviders(), loadCacheStatus()]); setLoading(false); }}
          disabled={loading}
          style={{ padding: 4, borderRadius: 5, border: "none", background: "transparent", cursor: "pointer", color: "var(--fg-3)", display: "flex" }}
        >
          <RefreshCw size={12} style={{ animation: loading ? "spin 1s linear infinite" : "none" }} />
        </button>
      </div>

      {error && (
        <div style={{ display: "flex", alignItems: "center", gap: 6, padding: "7px 10px", borderRadius: 7, background: "var(--red-soft)", fontSize: 11, color: "var(--red)" }}>
          <AlertCircle size={12} /> <span style={{ overflow: "hidden", textOverflow: "ellipsis" }}>{error}</span>
        </div>
      )}

      {/* Providers */}
      <div>
        <div style={{ fontSize: 10, fontWeight: 600, color: "var(--fg-3)", textTransform: "uppercase", letterSpacing: "0.6px", marginBottom: 6 }}>Providers</div>
        {providers.length === 0 ? (
          <p style={{ fontSize: 11, color: "var(--fg-3)" }}>No cloud providers configured. Add API keys in Settings.</p>
        ) : (
          <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 6 }}>
            {providers.map((p) => (
              <div key={p.name} style={rowStyle}>
                <span style={{ width: 6, height: 6, borderRadius: "50%", background: p.connected ? "var(--green)" : p.configured ? "var(--amber)" : "var(--fg-muted)", flexShrink: 0, boxShadow: p.connected ? "0 0 4px var(--green-glow)" : "none" }} />
                <span style={{ fontSize: 11, color: "var(--fg-2)", flex: 1 }}>{p.name}</span>
                <span style={{ fontSize: 10, color: "var(--fg-3)" }}>{p.connected ? "Connected" : p.configured ? "Configured" : "Not set up"}</span>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Cache */}
      {hasProviders && cacheResults.length > 0 && (
        <div>
          <div style={{ fontSize: 10, fontWeight: 600, color: "var(--fg-3)", textTransform: "uppercase", letterSpacing: "0.6px", marginBottom: 6 }}>Instant Availability</div>
          <div style={{ display: "flex", gap: 6, flexWrap: "wrap" }}>
            {cacheResults.map(([name, cached]) => (
              <div
                key={name}
                style={{
                  display: "flex", alignItems: "center", gap: 5, padding: "4px 10px", borderRadius: 99, fontSize: 11,
                  background: cached ? "var(--green-soft)" : "var(--overlay)",
                  color: cached ? "var(--green)" : "var(--fg-3)",
                  border: "1px solid var(--line)",
                }}
              >
                {cached ? <CheckCircle size={11} /> : <XCircle size={11} />}
                <span>{name}</span>
                <span style={{ fontSize: 10, opacity: 0.75 }}>{cached ? "Cached" : "Not cached"}</span>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Send to Cloud */}
      {hasProviders && !cloudSubmission && (
        <div>
          <div style={{ fontSize: 10, fontWeight: 600, color: "var(--fg-3)", textTransform: "uppercase", letterSpacing: "0.6px", marginBottom: 6 }}>Send to Cloud</div>
          <div style={{ display: "flex", gap: 6 }}>
            {configuredProviders.map((p) => (
              <button
                key={p.name}
                onClick={() => handleSendToCloud(p.name)}
                disabled={actionLoading !== null}
                className="transition-colors-fast"
                style={{ display: "flex", alignItems: "center", gap: 6, padding: "6px 14px", borderRadius: 7, background: "var(--accent-soft)", border: "none", color: "var(--accent)", fontSize: 12, cursor: "pointer", opacity: actionLoading !== null ? 0.5 : 1 }}
              >
                {actionLoading === p.name ? <Loader2 size={12} style={{ animation: "spin 1s linear infinite" }} /> : <Cloud size={12} />}
                {p.name}
              </button>
            ))}
          </div>
        </div>
      )}

      {/* Cloud status */}
      {cloudSubmission && cloudStatus && (() => {
        const parsed = parseCloudStatus(cloudStatus);
        return (
          <div>
            <div style={{ fontSize: 10, fontWeight: 600, color: "var(--fg-3)", textTransform: "uppercase", letterSpacing: "0.6px", marginBottom: 6 }}>
              Cloud Status ({cloudSubmission.provider})
            </div>
            <div style={{ ...rowStyle }}>
              <CloudStatusIcon label={parsed.label} />
              <span style={{ fontSize: 11, color: "var(--fg-2)", textTransform: "capitalize" }}>{parsed.label}</span>
              {parsed.progress !== undefined && (
                <div style={{ flex: 1, display: "flex", alignItems: "center", gap: 8 }}>
                  <div style={{ flex: 1, height: 3, background: "var(--muted)", borderRadius: 99, overflow: "hidden" }}>
                    <div style={{ height: "100%", width: `${Math.min(parsed.progress * 100, 100)}%`, background: "var(--accent)", borderRadius: 99 }} />
                  </div>
                  <span style={{ fontSize: 10, color: "var(--fg-3)", fontFamily: "'JetBrains Mono', monospace" }}>{(parsed.progress * 100).toFixed(1)}%</span>
                </div>
              )}
              {parsed.reason && <span style={{ fontSize: 10, color: "var(--red)", overflow: "hidden", textOverflow: "ellipsis" }}>{parsed.reason}</span>}
            </div>
          </div>
        );
      })()}

      {/* Download links */}
      {links.length > 0 && (
        <div>
          <div style={{ fontSize: 10, fontWeight: 600, color: "var(--fg-3)", textTransform: "uppercase", letterSpacing: "0.6px", marginBottom: 6 }}>Download Links</div>
          <div style={{ display: "flex", flexDirection: "column", gap: 4 }}>
            {links.map((link, i) => (
              <div key={i} style={{ ...rowStyle }}>
                <Download size={12} style={{ color: "var(--fg-3)", flexShrink: 0 }} />
                <span style={{ fontSize: 11, color: "var(--fg-2)", flex: 1, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>{link.filename}</span>
                <span style={{ fontSize: 10, color: "var(--fg-3)", fontFamily: "'JetBrains Mono', monospace" }}>{formatBytes(link.size_bytes)}</span>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}

// ─── Speed Tab ───────────────────────────────────────────

function SpeedTab({ torrentId }: { torrentId: string }) {
  return (
    <div style={{ padding: 16, height: "100%", display: "flex", flexDirection: "column" }}>
      <SpeedGraph torrentId={torrentId} width={400} height={180} />
      <div style={{ marginTop: 12, fontSize: 11, color: "var(--fg-3)", textAlign: "center" }}>
        Real-time bandwidth usage for this torrent (last 60 seconds).
      </div>
    </div>
  );
}

// ─── Pieces Placeholder ──────────────────────────────────

function PiecesTab() {
  return <TabEmpty>Pieces visualization coming soon</TabEmpty>;
}

// ─── Main Detail Panel ──────────────────────────────────

export function DetailPanel() {
  const detailPanelOpen = useUIStore((s) => s.detailPanelOpen);
  const detailPanelTab = useUIStore((s) => s.detailPanelTab);
  const setDetailPanelOpen = useUIStore((s) => s.setDetailPanelOpen);
  const setDetailPanelTab = useUIStore((s) => s.setDetailPanelTab);
  const selectedId = useTorrentStore((s) => s.selectedTorrentId);
  const torrents = useTorrentStore((s) => s.torrents);
  const torrent = selectedId ? torrents.get(selectedId) : undefined;

  if (!detailPanelOpen || !torrent) return null;

  return (
    <div
      style={{
        height: 220,
        borderTop: "1px solid var(--line)",
        background: "var(--surface)",
        display: "flex",
        flexDirection: "column",
        flexShrink: 0,
      }}
    >
      {/* Tab bar */}
      <div
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
          borderBottom: "1px solid var(--line)",
          padding: "0 4px 0 8px",
          flexShrink: 0,
        }}
      >
        <div style={{ display: "flex", alignItems: "center" }}>
          {TABS.map((tab) => {
            const active = detailPanelTab === tab.id;
            return (
              <button
                key={tab.id}
                onClick={() => setDetailPanelTab(tab.id)}
                className="transition-colors-fast"
                style={{
                  display: "flex",
                  alignItems: "center",
                  gap: 5,
                  padding: "7px 10px",
                  border: "none",
                  background: "transparent",
                  cursor: "pointer",
                  fontSize: 11,
                  fontWeight: active ? 500 : 400,
                  color: active ? "var(--accent)" : "var(--fg-3)",
                  position: "relative",
                  borderBottom: active ? "2px solid var(--accent)" : "2px solid transparent",
                  marginBottom: -1,
                  transition: "color 120ms ease, border-color 120ms ease !important",
                }}
                onMouseEnter={(e) => {
                  if (!active) (e.currentTarget as HTMLButtonElement).style.color = "var(--fg-2)";
                }}
                onMouseLeave={(e) => {
                  if (!active) (e.currentTarget as HTMLButtonElement).style.color = "var(--fg-3)";
                }}
              >
                {tab.icon}
                {tab.label}
              </button>
            );
          })}
        </div>

        {/* Right: torrent name + close */}
        <div style={{ display: "flex", alignItems: "center", gap: 8, paddingRight: 4 }}>
          <span style={{ fontSize: 11, color: "var(--fg-3)", maxWidth: 200, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
            {torrent.name}
          </span>
          <button
            onClick={() => setDetailPanelOpen(false)}
            className="transition-colors-fast"
            style={{ padding: 5, borderRadius: 5, border: "none", background: "transparent", cursor: "pointer", color: "var(--fg-3)", display: "flex" }}
            onMouseEnter={(e) => { (e.currentTarget as HTMLButtonElement).style.background = "var(--muted)"; (e.currentTarget as HTMLButtonElement).style.color = "var(--fg-2)"; }}
            onMouseLeave={(e) => { (e.currentTarget as HTMLButtonElement).style.background = "transparent"; (e.currentTarget as HTMLButtonElement).style.color = "var(--fg-3)"; }}
          >
            <X size={13} />
          </button>
        </div>
      </div>

      {/* Tab content */}
      <div style={{ flex: 1, overflow: "hidden", display: "flex", flexDirection: "column" }}>
        {detailPanelTab === "general" && <GeneralTab />}
        {detailPanelTab === "files" && <FilesTab />}
        {detailPanelTab === "peers" && <PeersTab />}
        {detailPanelTab === "trackers" && <TrackersTab />}
        {detailPanelTab === "speed" && <SpeedTab torrentId={torrent.id} />}
        {detailPanelTab === "pieces" && <PiecesTab />}
        {detailPanelTab === "cloud" && <CloudTab />}
      </div>
    </div>
  );
}
