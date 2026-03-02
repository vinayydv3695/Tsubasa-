// Tsubasa — Tauri IPC Bridge
// Type-safe wrappers around Tauri invoke/listen.

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  AccountInfo,
  AddTorrentRequest,
  AddTorrentResponse,
  AllSettings,
  AppInfo,
  AppSettings,
  BehaviorSettings,
  BitTorrentSettings,
  CacheCheckResult,
  CloudAddResult,
  CloudDownloadRequest,
  CloudDownloadResult,
  CloudProviderStatus,
  CloudSettingsV2,
  CloudStatus,
  ConnectionSettings,
  DirectLink,
  DownloadSettings,
  LogEntry,
  PluginSearchResult,
  QueuePositionInfo,
  QueueSettings,
  SearchHistoryEntry,
  SearchPluginInfo,
  SearchResult,
  SeedingSettings,
  SpeedSample,
  SpeedSettings,
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

// ─── Settings v2 Commands ──────────────────────────────

export async function getAllSettings(): Promise<AllSettings> {
  return invoke("get_all_settings");
}

export async function getBehaviorSettings(): Promise<BehaviorSettings> {
  return invoke("get_behavior_settings");
}

export async function getDownloadSettings(): Promise<DownloadSettings> {
  return invoke("get_download_settings");
}

export async function getConnectionSettings(): Promise<ConnectionSettings> {
  return invoke("get_connection_settings");
}

export async function getSpeedSettings(): Promise<SpeedSettings> {
  return invoke("get_speed_settings");
}

export async function getBitTorrentSettings(): Promise<BitTorrentSettings> {
  return invoke("get_bittorrent_settings");
}

export async function getQueueSettings(): Promise<QueueSettings> {
  return invoke("get_queue_settings");
}

export async function getSeedingSettings(): Promise<SeedingSettings> {
  return invoke("get_seeding_settings");
}

export async function getCloudSettingsV2(): Promise<CloudSettingsV2> {
  return invoke("get_cloud_settings");
}

export async function updateBehaviorSettings(settings: BehaviorSettings): Promise<void> {
  return invoke("update_behavior_settings", { settings });
}

export async function updateDownloadSettings(settings: DownloadSettings): Promise<void> {
  return invoke("update_download_settings", { settings });
}

export async function updateConnectionSettings(settings: ConnectionSettings): Promise<void> {
  return invoke("update_connection_settings", { settings });
}

export async function updateSpeedSettings(settings: SpeedSettings): Promise<void> {
  return invoke("update_speed_settings", { settings });
}

export async function updateBitTorrentSettings(settings: BitTorrentSettings): Promise<void> {
  return invoke("update_bittorrent_settings", { settings });
}

export async function updateQueueSettings(settings: QueueSettings): Promise<void> {
  return invoke("update_queue_settings", { settings });
}

export async function updateSeedingSettings(settings: SeedingSettings): Promise<void> {
  return invoke("update_seeding_settings", { settings });
}

export async function updateCloudSettingsV2(settings: CloudSettingsV2): Promise<void> {
  return invoke("update_cloud_settings", { settings });
}

// ─── Search Aggregator Commands ────────────────────────

export async function aggregatorSearch(
  query: string,
  category?: string,
  plugins?: string[],
): Promise<PluginSearchResult[]> {
  return invoke("aggregator_search", {
    query,
    category: category ?? null,
    plugins: plugins ?? null,
  });
}

export async function getSearchPlugins(): Promise<SearchPluginInfo[]> {
  return invoke("get_search_plugins");
}

// ─── Speed Graph Commands ──────────────────────────────

export async function getSpeedGraph(windowSecs?: number): Promise<SpeedSample[]> {
  return invoke("get_speed_graph", { windowSecs: windowSecs ?? null });
}

export async function getTorrentSpeedGraph(
  torrentId: string,
  windowSecs?: number,
): Promise<SpeedSample[]> {
  return invoke("get_torrent_speed_graph", {
    torrentId,
    windowSecs: windowSecs ?? null,
  });
}

// ─── Queue Commands ────────────────────────────────────

export async function getQueuePositions(): Promise<QueuePositionInfo[]> {
  return invoke("get_queue_positions");
}

export async function forceStartTorrent(torrentId: string): Promise<void> {
  return invoke("force_start_torrent", { torrentId });
}

export async function setTorrentPriority(
  torrentId: string,
  priority: number,
): Promise<void> {
  return invoke("set_torrent_priority", { torrentId, priority });
}

// ─── Event Listener ────────────────────────────────────

export async function onTsubasaEvent(
  callback: (event: TsubasaEventPayload) => void
): Promise<UnlistenFn> {
  return listen<TsubasaEventPayload>("tsubasa-event", (event) => {
    callback(event.payload);
  });
}
