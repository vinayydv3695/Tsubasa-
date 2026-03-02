// Tsubasa (翼) — Settings Schema
// Grouped configuration structs. No monolith config — each domain owns its settings.
// Per-torrent overrides merge with global defaults at read time.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::download::state_machine::DownloadPolicy;

// ─── Enums ──────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProxyType {
    None,
    Socks4,
    Socks5,
    Http,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyAuth {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EncryptionMode {
    Forced,
    Preferred,
    Disabled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SeedingAction {
    Pause,
    Remove,
    RemoveWithFiles,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeedSchedule {
    pub enabled: bool,
    pub start_hour: u8,   // 0-23
    pub end_hour: u8,     // 0-23
    pub days: Vec<u8>,    // 0=Sun, 1=Mon, ... 6=Sat
}

// ─── Settings Groups ────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorSettings {
    pub confirm_on_exit: bool,
    pub minimize_to_tray: bool,
    pub start_minimized: bool,
    pub single_instance: bool,
    pub locale: String,
    pub onboarding_completed: bool,
    pub theme: String,
}

impl Default for BehaviorSettings {
    fn default() -> Self {
        Self {
            confirm_on_exit: true,
            minimize_to_tray: true,
            start_minimized: false,
            single_instance: true,
            locale: "en".to_string(),
            onboarding_completed: false,
            theme: "black".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadSettings {
    pub default_save_path: String,
    pub temp_path: Option<String>,
    pub create_subfolder: bool,
    pub pre_allocate_disk: bool,
    pub append_incomplete_ext: bool,
    pub auto_delete_torrent_file: bool,
    pub move_completed_path: Option<String>,
}

impl Default for DownloadSettings {
    fn default() -> Self {
        let save_path = dirs::download_dir()
            .unwrap_or_else(|| dirs::home_dir().unwrap_or_default().join("Downloads"))
            .to_string_lossy()
            .to_string();
        Self {
            default_save_path: save_path,
            temp_path: None,
            create_subfolder: true,
            pre_allocate_disk: false,
            append_incomplete_ext: false,
            auto_delete_torrent_file: false,
            move_completed_path: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionSettings {
    pub global_max_connections: u32,
    pub per_torrent_max_connections: u32,
    pub max_upload_slots_global: u32,
    pub max_upload_slots_per_torrent: u32,
    pub enable_upnp: bool,
    pub enable_natpmp: bool,
    pub listen_port: u16,
    pub bind_interface: Option<String>,
    pub enable_utp: bool,
}

impl Default for ConnectionSettings {
    fn default() -> Self {
        Self {
            global_max_connections: 500,
            per_torrent_max_connections: 100,
            max_upload_slots_global: 10,
            max_upload_slots_per_torrent: 4,
            enable_upnp: true,
            enable_natpmp: true,
            listen_port: 6881,
            bind_interface: None,
            enable_utp: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxySettings {
    pub proxy_type: ProxyType,
    pub host: String,
    pub port: u16,
    pub auth: Option<ProxyAuth>,
    pub apply_to_peers: bool,
    pub apply_to_trackers: bool,
    pub apply_to_search: bool,
    pub resolve_hostname_via_proxy: bool,
}

impl Default for ProxySettings {
    fn default() -> Self {
        Self {
            proxy_type: ProxyType::None,
            host: String::new(),
            port: 1080,
            auth: None,
            apply_to_peers: false,
            apply_to_trackers: false,
            apply_to_search: false,
            resolve_hostname_via_proxy: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeedSettings {
    pub global_dl_limit: u64,
    pub global_ul_limit: u64,
    pub alt_dl_limit: u64,
    pub alt_ul_limit: u64,
    pub alt_speed_enabled: bool,
    pub alt_speed_schedule: Option<SpeedSchedule>,
    pub rate_limit_utp: bool,
    pub rate_limit_overhead: bool,
}

impl Default for SpeedSettings {
    fn default() -> Self {
        Self {
            global_dl_limit: 0,
            global_ul_limit: 0,
            alt_dl_limit: 0,
            alt_ul_limit: 0,
            alt_speed_enabled: false,
            alt_speed_schedule: None,
            rate_limit_utp: true,
            rate_limit_overhead: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitTorrentSettings {
    pub enable_dht: bool,
    pub enable_pex: bool,
    pub enable_lsd: bool,
    pub anonymous_mode: bool,
    pub encryption: EncryptionMode,
    pub sequential_download_default: bool,
}

impl Default for BitTorrentSettings {
    fn default() -> Self {
        Self {
            enable_dht: true,
            enable_pex: true,
            enable_lsd: true,
            anonymous_mode: false,
            encryption: EncryptionMode::Preferred,
            sequential_download_default: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueSettings {
    pub max_active_downloads: u32,
    pub max_active_uploads: u32,
    pub max_active_total: u32,
    pub slow_torrent_dl_threshold: u64,
    pub slow_torrent_ul_threshold: u64,
    pub slow_torrent_inactive_secs: u64,
    pub exclude_slow_from_count: bool,
}

impl Default for QueueSettings {
    fn default() -> Self {
        Self {
            max_active_downloads: 5,
            max_active_uploads: 5,
            max_active_total: 8,
            slow_torrent_dl_threshold: 1024,   // 1 KB/s
            slow_torrent_ul_threshold: 1024,
            slow_torrent_inactive_secs: 300,    // 5 minutes
            exclude_slow_from_count: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeedingSettings {
    pub global_ratio_limit: Option<f64>,
    pub global_time_limit_mins: Option<u64>,
    pub inactive_timeout_mins: Option<u64>,
    pub action_on_limit: SeedingAction,
}

impl Default for SeedingSettings {
    fn default() -> Self {
        Self {
            global_ratio_limit: Some(2.0),
            global_time_limit_mins: None,
            inactive_timeout_mins: None,
            action_on_limit: SeedingAction::Pause,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackerSettings {
    pub auto_append_enabled: bool,
    pub auto_append_url: Option<String>,
    pub auto_append_list: Vec<String>,
}

impl Default for TrackerSettings {
    fn default() -> Self {
        Self {
            auto_append_enabled: false,
            auto_append_url: Some("https://raw.githubusercontent.com/ngosang/trackerslist/master/trackers_best.txt".to_string()),
            auto_append_list: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IPFilterSettings {
    pub enabled: bool,
    pub blocklist_path: Option<String>,
    pub apply_to_trackers: bool,
    pub auto_update_url: Option<String>,
}

impl Default for IPFilterSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            blocklist_path: None,
            apply_to_trackers: false,
            auto_update_url: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchSettings {
    pub enabled_plugins: Vec<String>,
    pub max_results_per_plugin: u32,
    pub search_timeout_secs: u64,
    pub safe_search: bool,
}

impl Default for SearchSettings {
    fn default() -> Self {
        Self {
            enabled_plugins: vec![
                "piratebay".to_string(),
                "leet".to_string(),
                "yts".to_string(),
                "nyaa".to_string(),
                "torrentgalaxy".to_string(),
            ],
            max_results_per_plugin: 50,
            search_timeout_secs: 15,
            safe_search: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudSettings {
    pub torbox_api_key: Option<String>,
    pub realdebrid_api_key: Option<String>,
    pub default_policy: DownloadPolicy,
    pub cache_check_on_add: bool,
}

impl Default for CloudSettings {
    fn default() -> Self {
        Self {
            torbox_api_key: None,
            realdebrid_api_key: None,
            default_policy: DownloadPolicy::LocalOnly,
            cache_check_on_add: true,
        }
    }
}

// ─── Per-Torrent Overrides ──────────────────────────────────

/// Optional per-torrent overrides that take priority over global settings.
/// `None` fields inherit from the corresponding global setting.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TorrentOverrides {
    pub dl_limit: Option<u64>,
    pub ul_limit: Option<u64>,
    pub max_connections: Option<u32>,
    pub ratio_limit: Option<f64>,
    pub time_limit_mins: Option<u64>,
    pub sequential: Option<bool>,
    pub save_path: Option<String>,
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub download_policy: Option<DownloadPolicy>,
}

// ─── Aggregate (for serialization convenience) ──────────────

/// All settings in one struct. Used only for full export/import.
/// Normal access goes through SettingsManager group methods.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllSettings {
    pub behavior: BehaviorSettings,
    pub downloads: DownloadSettings,
    pub connections: ConnectionSettings,
    pub proxy: ProxySettings,
    pub speed: SpeedSettings,
    pub bittorrent: BitTorrentSettings,
    pub queue: QueueSettings,
    pub seeding: SeedingSettings,
    pub tracker: TrackerSettings,
    pub ip_filter: IPFilterSettings,
    pub search: SearchSettings,
    pub cloud: CloudSettings,
}

impl Default for AllSettings {
    fn default() -> Self {
        Self {
            behavior: BehaviorSettings::default(),
            downloads: DownloadSettings::default(),
            connections: ConnectionSettings::default(),
            proxy: ProxySettings::default(),
            speed: SpeedSettings::default(),
            bittorrent: BitTorrentSettings::default(),
            queue: QueueSettings::default(),
            seeding: SeedingSettings::default(),
            tracker: TrackerSettings::default(),
            ip_filter: IPFilterSettings::default(),
            search: SearchSettings::default(),
            cloud: CloudSettings::default(),
        }
    }
}
