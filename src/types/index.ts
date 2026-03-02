// Tsubasa — TypeScript Types
// Shared type definitions mirroring the Rust backend.

// ─── Torrent Types ─────────────────────────────────────

export type TorrentState =
  | "pending"
  | "checking"
  | "queued"
  | "downloading"
  | "paused"
  | "completed"
  | "seeding"
  | "stopped"
  | "errored";

export type DownloadPolicy = "local_only" | "cloud_only" | "hybrid";

export interface TorrentSummary {
  id: string;
  info_hash: string;
  name: string;
  state: TorrentState;
  policy: DownloadPolicy;
  progress: number;
  total_bytes: number;
  downloaded_bytes: number;
  uploaded_bytes: number;
  download_speed: number;
  upload_speed: number;
  peers_connected: number;
  seeds_connected: number;
  eta_seconds: number | null;
  ratio: number;
  added_at: string;
  save_path: string;
  error_message: string | null;
}

export interface TorrentRecord {
  id: string;
  info_hash: string;
  name: string;
  state: TorrentState;
  policy: DownloadPolicy;
  total_bytes: number;
  downloaded_bytes: number;
  uploaded_bytes: number;
  save_path: string;
  magnet_uri: string | null;
  added_at: string;
  completed_at: string | null;
  download_speed_limit: number;
  upload_speed_limit: number;
  max_peers: number;
  ratio_limit: number;
  error_message: string | null;
}

export interface AddTorrentRequest {
  source: string;
  save_path?: string;
  policy?: DownloadPolicy;
}

export interface AddTorrentResponse {
  id: string;
  name: string;
  info_hash: string;
}

// ─── Detail Panel Types ─────────────────────────────────

export interface TorrentFileInfo {
  index: number;
  path: string;
  name: string;
  size: number;
  downloaded: number;
  progress: number;
}

export interface TorrentPeerInfo {
  address: string;
  state: string;
  downloaded_bytes: number;
  uploaded_bytes: number;
  connection_attempts: number;
  errors: number;
}

export interface TorrentTrackerInfo {
  url: string;
  status: string;
}

// ─── Settings Types ────────────────────────────────────

export interface AppSettings {
  download_dir: string;
  default_policy: DownloadPolicy;
  global_download_limit: number;
  global_upload_limit: number;
  max_active_torrents: number;
  max_active_downloads: number;
  max_active_seeds: number;
  default_ratio_limit: number;
  enable_dht: boolean;
  enable_pex: boolean;
  listen_port: number;
  torbox_api_key: string | null;
  realdebrid_api_key: string | null;
  onboarding_completed: boolean;
  theme: string;
}

// ─── Cloud Types ───────────────────────────────────────

export interface CloudProviderStatus {
  name: string;
  configured: boolean;
  connected: boolean;
}

// CloudStatus is a Rust enum serialized with serde rename_all = "snake_case".
// It arrives as a tagged enum: {"queued": null}, {"downloading": {"progress": 0.5}}, etc.
export type CloudStatus =
  | "queued"
  | { downloading: { progress: number } }
  | "completed"
  | "cached"
  | { failed: { reason: string } }
  | "unknown";

export interface DirectLink {
  filename: string;
  url: string;
  size_bytes: number;
}

export interface AccountInfo {
  provider: string;
  username: string;
  plan: string;
  expiry: string | null;
  storage_used: number;
  storage_total: number;
  points_used: number | null;
  points_total: number | null;
}

export interface CloudAddResult {
  cloud_id: string;
  provider: string;
}

export interface CloudDownloadRequest {
  url: string;
  filename: string;
  save_dir: string;
  torrent_id: string;
  provider: string;
}

export interface CloudDownloadResult {
  filename: string;
  save_path: string;
  total_bytes: number;
}

export type CacheCheckResult = [string, boolean][];

// ─── Search Types ──────────────────────────────────────

export interface SearchResult {
  name: string;
  info_hash: string;
  size: number;
  seeders: number;
  leechers: number;
  source: string;
  category: string;
  magnet_uri: string;
  cached: boolean | null;
  uploaded_at: string | null;
}

export interface SearchHistoryEntry {
  id: number;
  query: string;
  timestamp: string;
}

// ─── Settings v2 Types (Grouped) ───────────────────────

export interface BehaviorSettings {
  confirm_on_exit: boolean;
  minimize_to_tray: boolean;
  start_minimized: boolean;
  single_instance: boolean;
  locale: string;
  onboarding_completed: boolean;
  theme: string;
}

export interface DownloadSettings {
  default_save_path: string;
  temp_path: string | null;
  create_subfolder: boolean;
  pre_allocate_disk: boolean;
  append_incomplete_ext: boolean;
  auto_delete_torrent_file: boolean;
  move_completed_path: string | null;
}

export interface ConnectionSettings {
  global_max_connections: number;
  per_torrent_max_connections: number;
  max_upload_slots_global: number;
  max_upload_slots_per_torrent: number;
  enable_upnp: boolean;
  enable_natpmp: boolean;
  listen_port: number;
  bind_interface: string | null;
  enable_utp: boolean;
}

export interface SpeedSettings {
  global_dl_limit: number;
  global_ul_limit: number;
  alt_dl_limit: number;
  alt_ul_limit: number;
  alt_speed_enabled: boolean;
  alt_speed_schedule: SpeedSchedule | null;
  rate_limit_utp: boolean;
  rate_limit_overhead: boolean;
}

export interface SpeedSchedule {
  enabled: boolean;
  start_hour: number;
  end_hour: number;
  days: number[];
}

export interface BitTorrentSettings {
  enable_dht: boolean;
  enable_pex: boolean;
  enable_lsd: boolean;
  anonymous_mode: boolean;
  encryption: "Forced" | "Preferred" | "Disabled";
  sequential_download_default: boolean;
}

export interface QueueSettings {
  max_active_downloads: number;
  max_active_uploads: number;
  max_active_total: number;
  slow_torrent_dl_threshold: number;
  slow_torrent_ul_threshold: number;
  slow_torrent_inactive_secs: number;
  exclude_slow_from_count: boolean;
}

export interface SeedingSettings {
  global_ratio_limit: number | null;
  global_time_limit_mins: number | null;
  inactive_timeout_mins: number | null;
  action_on_limit: "Pause" | "Remove" | "RemoveWithFiles";
}

export interface CloudSettingsV2 {
  torbox_api_key: string | null;
  realdebrid_api_key: string | null;
  default_policy: DownloadPolicy;
  cache_check_on_add: boolean;
}

export interface AllSettings {
  behavior: BehaviorSettings;
  downloads: DownloadSettings;
  connections: ConnectionSettings;
  proxy: ProxySettings;
  speed: SpeedSettings;
  bittorrent: BitTorrentSettings;
  queue: QueueSettings;
  seeding: SeedingSettings;
  tracker: TrackerSettings;
  ip_filter: IPFilterSettings;
  search: SearchSettingsV2;
  cloud: CloudSettingsV2;
}

export interface ProxySettings {
  proxy_type: "None" | "Socks4" | "Socks5" | "Http";
  host: string;
  port: number;
  auth: { username: string; password: string } | null;
  apply_to_peers: boolean;
  apply_to_trackers: boolean;
  apply_to_search: boolean;
  resolve_hostname_via_proxy: boolean;
}

export interface TrackerSettings {
  auto_append_enabled: boolean;
  auto_append_url: string | null;
  auto_append_list: string[];
}

export interface IPFilterSettings {
  enabled: boolean;
  blocklist_path: string | null;
  apply_to_trackers: boolean;
  auto_update_url: string | null;
}

export interface SearchSettingsV2 {
  enabled_plugins: string[];
  max_results_per_plugin: number;
  search_timeout_secs: number;
  safe_search: boolean;
}

// ─── Search Aggregator Types ───────────────────────────

export interface PluginSearchResult {
  title: string;
  magnet: string | null;
  torrent_url: string | null;
  info_hash: string | null;
  size_bytes: number;
  seeders: number;
  leechers: number;
  upload_date: string | null;
  source: string;
  category: string | null;
  source_url: string;
}

export interface SearchPluginInfo {
  id: string;
  name: string;
  categories: string[];
}

// ─── Speed Graph Types ─────────────────────────────────

export interface SpeedSample {
  timestamp_secs: number;
  download_speed: number;
  upload_speed: number;
}

// ─── Queue Types ───────────────────────────────────────

export interface QueuePositionInfo {
  torrent_id: string;
  position: string;
}

// ─── System Types ──────────────────────────────────────

export interface AppInfo {
  version: string;
  engine_ready: boolean;
  uptime_seconds: number;
}

export interface LogEntry {
  timestamp: string;
  level: string;
  target: string;
  message: string;
}

// ─── Event Types ───────────────────────────────────────

export type TsubasaEventPayload =
  | { type: "TorrentAdded"; payload: { id: string; name: string; info_hash: string } }
  | { type: "TorrentRemoved"; payload: { id: string } }
  | { type: "TorrentStateChanged"; payload: { id: string; from: string; to: string } }
  | {
    type: "ProgressUpdate";
    payload: {
      id: string;
      downloaded_bytes: number;
      total_bytes: number;
      download_speed: number;
      upload_speed: number;
      peers_connected: number;
      seeds_connected: number;
    };
  }
  | { type: "DownloadComplete"; payload: { id: string; name: string; path: string; size_bytes: number } }
  | { type: "CloudStatusChanged"; payload: { torrent_id: string; provider: string; status: string } }
  | { type: "CloudDownloadProgress"; payload: { torrent_id: string; provider: string; progress_pct: number } }
  | { type: "Error"; payload: { torrent_id: string | null; message: string; recoverable: boolean } }
  | { type: "EngineReady" }
  | { type: "EngineShuttingDown" }
  | {
    type: "GlobalStats";
    payload: {
      total_download_speed: number;
      total_upload_speed: number;
      active_torrents: number;
      total_peers: number;
    };
  };
