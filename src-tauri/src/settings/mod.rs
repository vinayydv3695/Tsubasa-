// Tsubasa (翼) — Settings Manager
// Grouped settings persistence with live reload.
// Stores each group as a JSON blob in the settings table.
// Migrates from the old flat AppSettings on first load.

pub mod schema;

use parking_lot::RwLock;
use std::sync::Arc;

use crate::error::TsubasaError;
use crate::storage::database::Database;

use schema::*;

/// Settings group identifiers (used as keys in the settings table).
const GROUP_BEHAVIOR: &str = "v2.behavior";
const GROUP_DOWNLOADS: &str = "v2.downloads";
const GROUP_CONNECTIONS: &str = "v2.connections";
const GROUP_PROXY: &str = "v2.proxy";
const GROUP_SPEED: &str = "v2.speed";
const GROUP_BITTORRENT: &str = "v2.bittorrent";
const GROUP_QUEUE: &str = "v2.queue";
const GROUP_SEEDING: &str = "v2.seeding";
const GROUP_TRACKER: &str = "v2.tracker";
const GROUP_IP_FILTER: &str = "v2.ip_filter";
const GROUP_SEARCH: &str = "v2.search";
const GROUP_CLOUD: &str = "v2.cloud";

/// Thread-safe settings manager with cached in-memory state.
/// Each group is independently loadable and saveable.
pub struct SettingsManager {
    db: Database,
    cache: RwLock<AllSettings>,
}

impl SettingsManager {
    /// Create a new SettingsManager, loading all settings from the database.
    /// If no v2 settings exist, migrates from the old flat schema.
    pub fn new(db: Database) -> crate::error::Result<Self> {
        let mgr = Self {
            db: db.clone(),
            cache: RwLock::new(AllSettings::default()),
        };

        // Check if v2 settings exist
        let has_v2 = db.get_setting(GROUP_BEHAVIOR)?.is_some();

        if has_v2 {
            mgr.load_all()?;
        } else {
            // Migrate from old flat settings
            mgr.migrate_from_v1()?;
        }

        Ok(mgr)
    }

    // ─── Group Getters (read from cache) ────────────────────

    pub fn behavior(&self) -> BehaviorSettings {
        self.cache.read().behavior.clone()
    }

    pub fn downloads(&self) -> DownloadSettings {
        self.cache.read().downloads.clone()
    }

    pub fn connections(&self) -> ConnectionSettings {
        self.cache.read().connections.clone()
    }

    pub fn proxy(&self) -> ProxySettings {
        self.cache.read().proxy.clone()
    }

    pub fn speed(&self) -> SpeedSettings {
        self.cache.read().speed.clone()
    }

    pub fn bittorrent(&self) -> BitTorrentSettings {
        self.cache.read().bittorrent.clone()
    }

    pub fn queue(&self) -> QueueSettings {
        self.cache.read().queue.clone()
    }

    pub fn seeding(&self) -> SeedingSettings {
        self.cache.read().seeding.clone()
    }

    pub fn tracker(&self) -> TrackerSettings {
        self.cache.read().tracker.clone()
    }

    pub fn ip_filter(&self) -> IPFilterSettings {
        self.cache.read().ip_filter.clone()
    }

    pub fn search(&self) -> SearchSettings {
        self.cache.read().search.clone()
    }

    pub fn cloud(&self) -> CloudSettings {
        self.cache.read().cloud.clone()
    }

    /// Get all settings (for full export / frontend settings panel).
    pub fn all(&self) -> AllSettings {
        self.cache.read().clone()
    }

    // ─── Group Setters (update cache + persist) ─────────────

    pub fn set_behavior(&self, s: BehaviorSettings) -> crate::error::Result<()> {
        self.save_group(GROUP_BEHAVIOR, &s)?;
        self.cache.write().behavior = s;
        Ok(())
    }

    pub fn set_downloads(&self, s: DownloadSettings) -> crate::error::Result<()> {
        self.save_group(GROUP_DOWNLOADS, &s)?;
        self.cache.write().downloads = s;
        Ok(())
    }

    pub fn set_connections(&self, s: ConnectionSettings) -> crate::error::Result<()> {
        self.save_group(GROUP_CONNECTIONS, &s)?;
        self.cache.write().connections = s;
        Ok(())
    }

    pub fn set_proxy(&self, s: ProxySettings) -> crate::error::Result<()> {
        self.save_group(GROUP_PROXY, &s)?;
        self.cache.write().proxy = s;
        Ok(())
    }

    pub fn set_speed(&self, s: SpeedSettings) -> crate::error::Result<()> {
        self.save_group(GROUP_SPEED, &s)?;
        self.cache.write().speed = s;
        Ok(())
    }

    pub fn set_bittorrent(&self, s: BitTorrentSettings) -> crate::error::Result<()> {
        self.save_group(GROUP_BITTORRENT, &s)?;
        self.cache.write().bittorrent = s;
        Ok(())
    }

    pub fn set_queue(&self, s: QueueSettings) -> crate::error::Result<()> {
        self.save_group(GROUP_QUEUE, &s)?;
        self.cache.write().queue = s;
        Ok(())
    }

    pub fn set_seeding(&self, s: SeedingSettings) -> crate::error::Result<()> {
        self.save_group(GROUP_SEEDING, &s)?;
        self.cache.write().seeding = s;
        Ok(())
    }

    pub fn set_tracker(&self, s: TrackerSettings) -> crate::error::Result<()> {
        self.save_group(GROUP_TRACKER, &s)?;
        self.cache.write().tracker = s;
        Ok(())
    }

    pub fn set_ip_filter(&self, s: IPFilterSettings) -> crate::error::Result<()> {
        self.save_group(GROUP_IP_FILTER, &s)?;
        self.cache.write().ip_filter = s;
        Ok(())
    }

    pub fn set_search(&self, s: SearchSettings) -> crate::error::Result<()> {
        self.save_group(GROUP_SEARCH, &s)?;
        self.cache.write().search = s;
        Ok(())
    }

    pub fn set_cloud(&self, s: CloudSettings) -> crate::error::Result<()> {
        self.save_group(GROUP_CLOUD, &s)?;
        self.cache.write().cloud = s;
        Ok(())
    }

    // ─── Internals ──────────────────────────────────────────

    /// Load all settings from DB into cache.
    fn load_all(&self) -> crate::error::Result<()> {
        let mut all = AllSettings::default();

        all.behavior = self.load_group::<BehaviorSettings>(GROUP_BEHAVIOR)?
            .unwrap_or_default();
        all.downloads = self.load_group::<DownloadSettings>(GROUP_DOWNLOADS)?
            .unwrap_or_default();
        all.connections = self.load_group::<ConnectionSettings>(GROUP_CONNECTIONS)?
            .unwrap_or_default();
        all.proxy = self.load_group::<ProxySettings>(GROUP_PROXY)?
            .unwrap_or_default();
        all.speed = self.load_group::<SpeedSettings>(GROUP_SPEED)?
            .unwrap_or_default();
        all.bittorrent = self.load_group::<BitTorrentSettings>(GROUP_BITTORRENT)?
            .unwrap_or_default();
        all.queue = self.load_group::<QueueSettings>(GROUP_QUEUE)?
            .unwrap_or_default();
        all.seeding = self.load_group::<SeedingSettings>(GROUP_SEEDING)?
            .unwrap_or_default();
        all.tracker = self.load_group::<TrackerSettings>(GROUP_TRACKER)?
            .unwrap_or_default();
        all.ip_filter = self.load_group::<IPFilterSettings>(GROUP_IP_FILTER)?
            .unwrap_or_default();
        all.search = self.load_group::<SearchSettings>(GROUP_SEARCH)?
            .unwrap_or_default();
        all.cloud = self.load_group::<CloudSettings>(GROUP_CLOUD)?
            .unwrap_or_default();

        *self.cache.write() = all;
        Ok(())
    }

    /// Load a single settings group from the DB.
    fn load_group<T: serde::de::DeserializeOwned>(&self, key: &str) -> crate::error::Result<Option<T>> {
        match self.db.get_setting(key)? {
            Some(json) => {
                let val: T = serde_json::from_str(&json)
                    .map_err(|e| TsubasaError::Settings(format!("Failed to parse {}: {}", key, e)))?;
                Ok(Some(val))
            }
            None => Ok(None),
        }
    }

    /// Save a single settings group to the DB.
    fn save_group<T: serde::Serialize>(&self, key: &str, value: &T) -> crate::error::Result<()> {
        let json = serde_json::to_string(value)
            .map_err(|e| TsubasaError::Settings(format!("Failed to serialize {}: {}", key, e)))?;
        self.db.set_setting(key, &json)?;
        Ok(())
    }

    /// Migrate from old v1 flat AppSettings to new grouped schema.
    fn migrate_from_v1(&self) -> crate::error::Result<()> {
        tracing::info!("Migrating settings from v1 to v2 grouped schema");

        // Try to load the old-style settings
        let old = self.db.load_settings();

        match old {
            Ok(old_settings) => {
                // Map old flat fields to new groups
                let behavior = BehaviorSettings {
                    onboarding_completed: old_settings.onboarding_completed,
                    theme: old_settings.theme.clone(),
                    ..BehaviorSettings::default()
                };

                let downloads = DownloadSettings {
                    default_save_path: old_settings.download_dir.clone(),
                    ..DownloadSettings::default()
                };

                let connections = ConnectionSettings {
                    listen_port: old_settings.listen_port,
                    ..ConnectionSettings::default()
                };

                let speed = SpeedSettings {
                    global_dl_limit: old_settings.global_download_limit,
                    global_ul_limit: old_settings.global_upload_limit,
                    ..SpeedSettings::default()
                };

                let bittorrent = BitTorrentSettings {
                    enable_dht: old_settings.enable_dht,
                    enable_pex: old_settings.enable_pex,
                    ..BitTorrentSettings::default()
                };

                let queue = QueueSettings {
                    max_active_downloads: old_settings.max_active_downloads,
                    max_active_uploads: old_settings.max_active_seeds,
                    max_active_total: old_settings.max_active_torrents,
                    ..QueueSettings::default()
                };

                let seeding = SeedingSettings {
                    global_ratio_limit: Some(old_settings.default_ratio_limit),
                    ..SeedingSettings::default()
                };

                let cloud = CloudSettings {
                    torbox_api_key: old_settings.torbox_api_key.clone(),
                    realdebrid_api_key: old_settings.realdebrid_api_key.clone(),
                    default_policy: old_settings.default_policy,
                    ..CloudSettings::default()
                };

                // Save all groups
                self.save_group(GROUP_BEHAVIOR, &behavior)?;
                self.save_group(GROUP_DOWNLOADS, &downloads)?;
                self.save_group(GROUP_CONNECTIONS, &connections)?;
                self.save_group(GROUP_PROXY, &ProxySettings::default())?;
                self.save_group(GROUP_SPEED, &speed)?;
                self.save_group(GROUP_BITTORRENT, &bittorrent)?;
                self.save_group(GROUP_QUEUE, &queue)?;
                self.save_group(GROUP_SEEDING, &seeding)?;
                self.save_group(GROUP_TRACKER, &TrackerSettings::default())?;
                self.save_group(GROUP_IP_FILTER, &IPFilterSettings::default())?;
                self.save_group(GROUP_SEARCH, &SearchSettings::default())?;
                self.save_group(GROUP_CLOUD, &cloud)?;

                // Load into cache
                let mut all = AllSettings::default();
                all.behavior = behavior;
                all.downloads = downloads;
                all.connections = connections;
                all.speed = speed;
                all.bittorrent = bittorrent;
                all.queue = queue;
                all.seeding = seeding;
                all.cloud = cloud;
                *self.cache.write() = all;

                tracing::info!("Settings migration complete");
            }
            Err(_) => {
                // No old settings either — use defaults and save them
                tracing::info!("No existing settings found, using defaults");
                let defaults = AllSettings::default();
                self.save_group(GROUP_BEHAVIOR, &defaults.behavior)?;
                self.save_group(GROUP_DOWNLOADS, &defaults.downloads)?;
                self.save_group(GROUP_CONNECTIONS, &defaults.connections)?;
                self.save_group(GROUP_PROXY, &defaults.proxy)?;
                self.save_group(GROUP_SPEED, &defaults.speed)?;
                self.save_group(GROUP_BITTORRENT, &defaults.bittorrent)?;
                self.save_group(GROUP_QUEUE, &defaults.queue)?;
                self.save_group(GROUP_SEEDING, &defaults.seeding)?;
                self.save_group(GROUP_TRACKER, &defaults.tracker)?;
                self.save_group(GROUP_IP_FILTER, &defaults.ip_filter)?;
                self.save_group(GROUP_SEARCH, &defaults.search)?;
                self.save_group(GROUP_CLOUD, &defaults.cloud)?;
                *self.cache.write() = defaults;
            }
        }

        Ok(())
    }

    // ─── Compatibility layer for old code ───────────────────

    /// Build a legacy AppSettings from the new grouped settings.
    /// Used for backward compatibility with existing IPC commands.
    pub fn to_legacy_settings(&self) -> crate::storage::models::AppSettings {
        let cache = self.cache.read();
        crate::storage::models::AppSettings {
            download_dir: cache.downloads.default_save_path.clone(),
            default_policy: cache.cloud.default_policy,
            global_download_limit: cache.speed.global_dl_limit,
            global_upload_limit: cache.speed.global_ul_limit,
            max_active_torrents: cache.queue.max_active_total,
            max_active_downloads: cache.queue.max_active_downloads,
            max_active_seeds: cache.queue.max_active_uploads,
            default_ratio_limit: cache.seeding.global_ratio_limit.unwrap_or(2.0),
            enable_dht: cache.bittorrent.enable_dht,
            enable_pex: cache.bittorrent.enable_pex,
            listen_port: cache.connections.listen_port,
            torbox_api_key: cache.cloud.torbox_api_key.clone(),
            realdebrid_api_key: cache.cloud.realdebrid_api_key.clone(),
            onboarding_completed: cache.behavior.onboarding_completed,
            theme: cache.behavior.theme.clone(),
        }
    }

    /// Update from a legacy AppSettings (backward compat for existing update_settings command).
    pub fn from_legacy_settings(&self, old: &crate::storage::models::AppSettings) -> crate::error::Result<()> {
        // Update each group that the legacy settings touch
        let mut behavior = self.behavior();
        behavior.onboarding_completed = old.onboarding_completed;
        behavior.theme = old.theme.clone();
        self.set_behavior(behavior)?;

        let mut downloads = self.downloads();
        downloads.default_save_path = old.download_dir.clone();
        self.set_downloads(downloads)?;

        let mut connections = self.connections();
        connections.listen_port = old.listen_port;
        self.set_connections(connections)?;

        let mut speed = self.speed();
        speed.global_dl_limit = old.global_download_limit;
        speed.global_ul_limit = old.global_upload_limit;
        self.set_speed(speed)?;

        let mut bittorrent = self.bittorrent();
        bittorrent.enable_dht = old.enable_dht;
        bittorrent.enable_pex = old.enable_pex;
        self.set_bittorrent(bittorrent)?;

        let mut queue = self.queue();
        queue.max_active_downloads = old.max_active_downloads;
        queue.max_active_uploads = old.max_active_seeds;
        queue.max_active_total = old.max_active_torrents;
        self.set_queue(queue)?;

        let mut seeding = self.seeding();
        seeding.global_ratio_limit = Some(old.default_ratio_limit);
        self.set_seeding(seeding)?;

        let mut cloud = self.cloud();
        cloud.torbox_api_key = old.torbox_api_key.clone();
        cloud.realdebrid_api_key = old.realdebrid_api_key.clone();
        cloud.default_policy = old.default_policy;
        self.set_cloud(cloud)?;

        Ok(())
    }
}
