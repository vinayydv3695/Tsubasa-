// Tsubasa (翼) — Event Bridge
// Connects Tauri backend events to Zustand stores.
// Single initialization, batched updates, OS notifications.

import { onTsubasaEvent } from "@/lib/tauri";
import { useTorrentStore } from "@/stores/torrent";
import { useUIStore } from "@/stores/ui";
import { useToastStore } from "@/stores/toast";
import type { TsubasaEventPayload } from "@/types";
import {
  isPermissionGranted,
  requestPermission,
  sendNotification,
} from "@tauri-apps/plugin-notification";

let initialized = false;

/** Send an OS notification if the app window is not focused. */
async function maybeNotify(title: string, body: string) {
  try {
    // Only notify when app is in background
    if (document.hasFocus()) return;

    let permitted = await isPermissionGranted();
    if (!permitted) {
      const permission = await requestPermission();
      permitted = permission === "granted";
    }
    if (permitted) {
      sendNotification({ title, body });
    }
  } catch {
    // Notification API may not be available in dev mode — silently ignore
  }
}

/**
 * Initialize the event bridge. Call once on app mount.
 * Subscribes to all Tauri events and dispatches to stores.
 */
export async function initEventBridge(): Promise<() => void> {
  if (initialized) {
    console.warn("Event bridge already initialized");
    return () => {};
  }
  initialized = true;

  // Batch progress updates to avoid re-render flooding
  let pendingProgressUpdates: Map<
    string,
    {
      id: string;
      downloaded_bytes: number;
      total_bytes: number;
      download_speed: number;
      upload_speed: number;
      peers_connected: number;
      seeds_connected: number;
    }
  > = new Map();

  let rafId: number | null = null;

  function flushProgressUpdates() {
    const store = useTorrentStore.getState();
    for (const update of pendingProgressUpdates.values()) {
      store.updateProgress(update);
    }
    pendingProgressUpdates.clear();
    rafId = null;
  }

  const unlisten = await onTsubasaEvent((event: TsubasaEventPayload) => {
    switch (event.type) {
      case "TorrentAdded":
        useTorrentStore.getState().handleTorrentAdded(event.payload);
        break;

      case "TorrentRemoved":
        useTorrentStore.getState().handleTorrentRemoved(event.payload);
        break;

      case "TorrentStateChanged":
        useTorrentStore.getState().updateState(event.payload);
        break;

      case "ProgressUpdate":
        // Batch into animation frame
        pendingProgressUpdates.set(event.payload.id, event.payload);
        if (rafId === null) {
          rafId = requestAnimationFrame(flushProgressUpdates);
        }
        break;

      case "DownloadComplete":
        // Reload to get final state
        useTorrentStore.getState().loadTorrents();
        useToastStore.getState().addToast({
          type: "success",
          title: "Download Complete",
          message: event.payload.name,
        });
        maybeNotify("Download Complete", event.payload.name);
        break;

      case "EngineReady":
        useUIStore.getState().setEngineReady(true);
        // Load initial torrents once engine is ready
        useTorrentStore.getState().loadTorrents();
        break;

      case "EngineShuttingDown":
        useUIStore.getState().setEngineReady(false);
        break;

      case "GlobalStats":
        useUIStore.getState().updateGlobalStats(event.payload);
        break;

      case "Error":
        console.error(
          `Tsubasa error (torrent: ${event.payload.torrent_id}): ${event.payload.message}`
        );
        useToastStore.getState().addToast({
          type: "error",
          title: event.payload.recoverable ? "Error" : "Fatal Error",
          message: event.payload.message,
          duration: event.payload.recoverable ? 5000 : 0,
        });
        if (!event.payload.recoverable) {
          maybeNotify("Tsubasa Error", event.payload.message);
        }
        break;

      default:
        break;
    }
  });

  return () => {
    unlisten();
    initialized = false;
    if (rafId !== null) {
      cancelAnimationFrame(rafId);
    }
  };
}
