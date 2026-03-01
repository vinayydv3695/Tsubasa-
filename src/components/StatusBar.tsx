// Tsubasa (翼) — Status Bar Component (v3 — Manifesto Redesign)
// Engine status dot, speeds, peer count. Clean and minimal.

import { useEffect, useState } from "react";
import { ArrowDown, ArrowUp, Users, Cloud } from "lucide-react";
import { useUIStore } from "@/stores/ui";
import { useTorrentStore } from "@/stores/torrent";
import { formatSpeed } from "@/lib/utils";
import { getCloudStatus } from "@/lib/tauri";
import "./StatusBar.css";

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
    <div className="statusbar">
      {/* Left: Engine + torrent count */}
      <div className="statusbar__left">
        <div className="statusbar__engine">
          <span
            className={`statusbar__dot ${engineReady ? "statusbar__dot--ready" : "statusbar__dot--starting"}`}
          />
          <span className="statusbar__label">
            {engineReady ? "Ready" : "Starting…"}
          </span>
        </div>

        <span className="statusbar__mono">
          {torrentCount} torrents
        </span>

        {activeTorrents > 0 && (
          <span className="badge badge-accent">
            {activeTorrents} active
          </span>
        )}
      </div>

      {/* Right: Cloud + peers + speeds */}
      <div className="statusbar__right">
        {cloudCount > 0 && (
          <div
            className="statusbar__stat"
            title={`${cloudCount} cloud provider${cloudCount !== 1 ? "s" : ""} connected`}
          >
            <Cloud size={10} color="var(--accent)" />
            <span className="statusbar__mono">{cloudCount}</span>
          </div>
        )}

        <div className="statusbar__stat">
          <Users size={10} color="var(--fg-3)" />
          <span className="statusbar__mono">{totalPeers}</span>
        </div>

        <div className="statusbar__stat">
          <ArrowDown size={10} color="var(--green)" />
          <span className="statusbar__stat-value">{formatSpeed(globalDownloadSpeed)}</span>
        </div>

        <div className="statusbar__stat">
          <ArrowUp size={10} color="var(--accent)" />
          <span className="statusbar__stat-value">{formatSpeed(globalUploadSpeed)}</span>
        </div>
      </div>
    </div>
  );
}
