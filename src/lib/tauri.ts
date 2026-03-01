// Tsubasa — Tauri IPC Bridge
// Type-safe wrappers around Tauri invoke/listen.

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  AccountInfo,
  AddTorrentRequest,
  AddTorrentResponse,
  AppInfo,
  AppSettings,
  CacheCheckResult,
  CloudAddResult,
  CloudDownloadRequest,
  CloudDownloadResult,
  CloudProviderStatus,
  CloudStatus,
  DirectLink,
  LogEntry,
  SearchHistoryEntry,
  SearchResult,
  TorrentFileInfo,
  TorrentPeerInfo,
  TorrentRecord,
  TorrentSummary,
  TorrentTrackerInfo,
  TsubasaEventPayload,
} from "@/types";

// ─── Torrent Commands ──────────────────────────────────

export async function addTorrent(
  request: AddTorrentRequest
): Promise<AddTorrentResponse> {
  return invoke("add_torrent", { request });
}

export async function getTorrents(): Promise<TorrentSummary[]> {
  return invoke("get_torrents");
}

export async function pauseTorrent(id: string): Promise<void> {
  return invoke("pause_torrent", { id });
}

export async function resumeTorrent(id: string): Promise<void> {
  return invoke("resume_torrent", { id });
}

export async function removeTorrent(
  id: string,
  deleteFiles: boolean = false
): Promise<void> {
  return invoke("remove_torrent", { id, deleteFiles });
}

export async function getTorrentDetails(
  id: string
): Promise<TorrentRecord> {
  return invoke("get_torrent_details", { id });
}

export async function getTorrentFiles(
  id: string
): Promise<TorrentFileInfo[]> {
  return invoke("get_torrent_files", { id });
}

export async function getTorrentPeers(
  id: string
): Promise<TorrentPeerInfo[]> {
  return invoke("get_torrent_peers", { id });
}

export async function getTorrentTrackers(
  id: string
): Promise<TorrentTrackerInfo[]> {
  return invoke("get_torrent_trackers", { id });
}

// ─── Settings Commands ─────────────────────────────────

export async function getSettings(): Promise<AppSettings> {
  return invoke("get_settings");
}

export async function updateSettings(
  settings: AppSettings
): Promise<void> {
  return invoke("update_settings", { settings });
}

export async function getSetting(key: string): Promise<string | null> {
  return invoke("get_setting", { key });
}

export async function setSetting(
  key: string,
  value: string
): Promise<void> {
  return invoke("set_setting", { key, value });
}

// ─── Cloud Commands ────────────────────────────────────

export async function getCloudStatus(): Promise<CloudProviderStatus[]> {
  return invoke("get_cloud_status");
}

export async function cloudAddTorrent(
  source: string,
  provider: string
): Promise<CloudAddResult> {
  return invoke("cloud_add_torrent", { source, provider });
}

export async function cloudCheckStatus(
  cloudId: string,
  provider: string
): Promise<CloudStatus> {
  return invoke("cloud_check_status", { cloudId, provider });
}

export async function cloudGetLinks(
  cloudId: string,
  provider: string
): Promise<DirectLink[]> {
  return invoke("cloud_get_links", { cloudId, provider });
}

export async function cloudCheckCached(
  infoHash: string
): Promise<CacheCheckResult> {
  return invoke("cloud_check_cached", { infoHash });
}

export async function cloudAccountInfo(
  provider: string
): Promise<AccountInfo> {
  return invoke("cloud_account_info", { provider });
}

export async function cloudDeleteTorrent(
  cloudId: string,
  provider: string
): Promise<void> {
  return invoke("cloud_delete_torrent", { cloudId, provider });
}

export async function cloudDownloadFile(
  request: CloudDownloadRequest
): Promise<CloudDownloadResult> {
  return invoke("cloud_download_file", { request });
}

// ─── Search Commands ────────────────────────────────────

export async function searchTorrents(
  query: string,
  checkCache?: boolean
): Promise<SearchResult[]> {
  return invoke("search_torrents", { query, checkCache: checkCache ?? null });
}

export async function saveSearchHistory(query: string): Promise<void> {
  return invoke("save_search_history", { query });
}

export async function getSearchHistory(): Promise<SearchHistoryEntry[]> {
  return invoke("get_search_history");
}

export async function clearSearchHistory(): Promise<void> {
  return invoke("clear_search_history");
}

// ─── System Commands ───────────────────────────────────

export async function getAppInfo(): Promise<AppInfo> {
  return invoke("get_app_info");
}

export async function getLogs(
  sinceIndex?: number
): Promise<LogEntry[]> {
  return invoke("get_logs", { sinceIndex: sinceIndex ?? null });
}

export async function clearLogs(): Promise<void> {
  return invoke("clear_logs");
}

// ─── Event Listener ────────────────────────────────────

export async function onTsubasaEvent(
  callback: (event: TsubasaEventPayload) => void
): Promise<UnlistenFn> {
  return listen<TsubasaEventPayload>("tsubasa-event", (event) => {
    callback(event.payload);
  });
}
