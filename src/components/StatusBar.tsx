// Tsubasa (翼) — Status Bar Component

import { useEffect, useState } from "react";
import { ArrowDown, ArrowUp, Users, Cloud } from "lucide-react";
import { useUIStore } from "@/stores/ui";
import { useTorrentStore } from "@/stores/torrent";
import { formatSpeed } from "@/lib/utils";
import { getCloudStatus } from "@/lib/tauri";

export function StatusBar() {
  const engineReady = useUIStore((s) => s.engineReady);
  const globalDownloadSpeed = useUIStore((s) => s.globalDownloadSpeed);
  const globalUploadSpeed = useUIStore((s) => s.globalUploadSpeed);
  const activeTorrents = useUIStore((s) => s.activeTorrents);
  const totalPeers = useUIStore((s) => s.totalPeers);
  const torrentCount = useTorrentStore((s) => s.torrents.size);

  const [cloudCount, setCloudCount] = useState(0);

  useEffect(() => {
    const load = async () => {
      try {
        const statuses = await getCloudStatus();
        setCloudCount(statuses.filter((s) => s.connected).length);
      } catch {
        // Cloud status not critical for status bar
      }
    };
    load();
    const interval = setInterval(load, 30_000);
    return () => clearInterval(interval);
  }, []);

  return (
    <div
      style={{
        height: 28,
        display: "flex",
        alignItems: "center",
        justifyContent: "space-between",
        padding: "0 12px",
        borderTop: "1px solid var(--line)",
        background: "var(--surface)",
        flexShrink: 0,
      }}
    >
      {/* Left: Status */}
      <div style={{ display: "flex", alignItems: "center", gap: 12 }}>
        {/* Engine status dot */}
        <div style={{ display: "flex", alignItems: "center", gap: 5 }}>
          <span
            className={engineReady ? "pulse-green" : undefined}
            style={{
              width: 6,
              height: 6,
              borderRadius: "50%",
              background: engineReady ? "var(--green)" : "var(--amber)",
              flexShrink: 0,
              boxShadow: engineReady ? "0 0 5px var(--green-glow)" : "none",
            }}
          />
          <span style={{ fontSize: 11, color: "var(--fg-3)" }}>
            {engineReady ? "Ready" : "Starting…"}
          </span>
        </div>

        <span style={{ fontSize: 11, color: "var(--fg-3)", fontFamily: "'JetBrains Mono', monospace", fontVariantNumeric: "tabular-nums" }}>
          {torrentCount} torrents
        </span>

        {activeTorrents > 0 && (
          <span style={{
            fontSize: 10,
            color: "var(--accent)",
            background: "var(--accent-soft)",
            padding: "0 6px",
            borderRadius: 99,
            fontFamily: "'JetBrains Mono', monospace",
            fontVariantNumeric: "tabular-nums",
          }}>
            {activeTorrents} active
          </span>
        )}
      </div>

      {/* Right: Stats */}
      <div style={{ display: "flex", alignItems: "center", gap: 12 }}>
        {cloudCount > 0 && (
          <div
            style={{ display: "flex", alignItems: "center", gap: 4 }}
            title={`${cloudCount} cloud provider${cloudCount !== 1 ? "s" : ""} connected`}
          >
            <Cloud size={10} color="var(--accent)" />
            <span style={{ fontSize: 11, color: "var(--fg-3)", fontFamily: "'JetBrains Mono', monospace" }}>
              {cloudCount}
            </span>
          </div>
        )}

        <div style={{ display: "flex", alignItems: "center", gap: 4 }}>
          <Users size={10} color="var(--fg-3)" />
          <span style={{ fontSize: 11, color: "var(--fg-3)", fontFamily: "'JetBrains Mono', monospace" }}>
            {totalPeers}
          </span>
        </div>

        <div style={{ display: "flex", alignItems: "center", gap: 4 }}>
          <ArrowDown size={10} color="var(--green)" />
          <span style={{ fontSize: 11, color: "var(--fg-2)", fontFamily: "'JetBrains Mono', monospace" }}>
            {formatSpeed(globalDownloadSpeed)}
          </span>
        </div>

        <div style={{ display: "flex", alignItems: "center", gap: 4 }}>
          <ArrowUp size={10} color="var(--accent)" />
          <span style={{ fontSize: 11, color: "var(--fg-2)", fontFamily: "'JetBrains Mono', monospace" }}>
            {formatSpeed(globalUploadSpeed)}
          </span>
        </div>
      </div>
    </div>
  );
}
