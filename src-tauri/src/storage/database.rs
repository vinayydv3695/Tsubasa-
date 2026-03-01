// Tsubasa (翼) — SQLite Database
// Persistent storage for torrents, settings, and history.

use parking_lot::Mutex;
use rusqlite::{params, Connection};
use std::path::Path;
use std::sync::Arc;

use crate::error::{DatabaseError, TsubasaError};
use crate::storage::models::{AppSettings, TorrentRecord};

/// Database handle wrapping a SQLite connection.
/// Uses parking_lot::Mutex for synchronous locking (SQLite is single-writer).
/// Arc-wrapped so the handle can be cheaply cloned for spawned tasks.
#[derive(Clone)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    /// Open (or create) the database at the given path and run migrations.
    pub fn open(path: &Path) -> crate::error::Result<Self> {
        let conn =
            Connection::open(path).map_err(|e| TsubasaError::Database(DatabaseError::Sqlite(e)))?;

        // Enable WAL mode for better concurrent read performance
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")
            .map_err(|e| TsubasaError::Database(DatabaseError::Sqlite(e)))?;

        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        db.run_migrations()?;
        Ok(db)
    }

    /// Run all schema migrations.
    fn run_migrations(&self) -> crate::error::Result<()> {
        let conn = self.conn.lock();

        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS torrents (
                id              TEXT PRIMARY KEY,
                info_hash       TEXT NOT NULL,
                name            TEXT NOT NULL,
                state           TEXT NOT NULL DEFAULT 'pending',
                policy          TEXT NOT NULL DEFAULT 'local_only',
                total_bytes     INTEGER NOT NULL DEFAULT 0,
                downloaded_bytes INTEGER NOT NULL DEFAULT 0,
                uploaded_bytes  INTEGER NOT NULL DEFAULT 0,
                save_path       TEXT NOT NULL,
                magnet_uri      TEXT,
                added_at        TEXT NOT NULL,
                completed_at    TEXT,
                download_speed_limit INTEGER NOT NULL DEFAULT 0,
                upload_speed_limit   INTEGER NOT NULL DEFAULT 0,
                max_peers       INTEGER NOT NULL DEFAULT 0,
                ratio_limit     REAL NOT NULL DEFAULT 2.0,
                error_message   TEXT
            );

            CREATE INDEX IF NOT EXISTS idx_torrents_info_hash ON torrents(info_hash);
            CREATE INDEX IF NOT EXISTS idx_torrents_state ON torrents(state);

            CREATE TABLE IF NOT EXISTS settings (
                key   TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS torrent_files (
                id          TEXT PRIMARY KEY,
                torrent_id  TEXT NOT NULL REFERENCES torrents(id) ON DELETE CASCADE,
                file_index  INTEGER NOT NULL,
                path        TEXT NOT NULL,
                size_bytes  INTEGER NOT NULL,
                priority    INTEGER NOT NULL DEFAULT 1,
                progress    REAL NOT NULL DEFAULT 0.0
            );

            CREATE INDEX IF NOT EXISTS idx_torrent_files_torrent ON torrent_files(torrent_id);

            CREATE TABLE IF NOT EXISTS search_history (
                id        INTEGER PRIMARY KEY AUTOINCREMENT,
                query     TEXT NOT NULL,
                timestamp TEXT NOT NULL
            );
            ",
        )
        .map_err(|e| TsubasaError::Database(DatabaseError::Migration(e.to_string())))?;

        Ok(())
    }

    // ─── Torrent CRUD ───────────────────────────────────────

    /// Insert a new torrent record.
    pub fn insert_torrent(&self, record: &TorrentRecord) -> crate::error::Result<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO torrents (id, info_hash, name, state, policy, total_bytes, downloaded_bytes,
             uploaded_bytes, save_path, magnet_uri, added_at, completed_at,
             download_speed_limit, upload_speed_limit, max_peers, ratio_limit, error_message)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
            params![
                record.id,
                record.info_hash,
                record.name,
                record.state.to_string(),
                record.policy.to_string(),
                record.total_bytes,
                record.downloaded_bytes,
                record.uploaded_bytes,
                record.save_path,
                record.magnet_uri,
                record.added_at,
                record.completed_at,
                record.download_speed_limit,
                record.upload_speed_limit,
                record.max_peers,
                record.ratio_limit,
                record.error_message,
            ],
        )
        .map_err(|e| TsubasaError::Database(DatabaseError::Sqlite(e)))?;
        Ok(())
    }

    /// Get all torrent records.
    pub fn get_all_torrents(&self) -> crate::error::Result<Vec<TorrentRecord>> {
        let conn = self.conn.lock();
        let mut stmt = conn
            .prepare(
                "SELECT id, info_hash, name, state, policy, total_bytes, downloaded_bytes,
                 uploaded_bytes, save_path, magnet_uri, added_at, completed_at,
                 download_speed_limit, upload_speed_limit, max_peers, ratio_limit, error_message
                 FROM torrents ORDER BY added_at DESC",
            )
            .map_err(|e| TsubasaError::Database(DatabaseError::Sqlite(e)))?;

        let records = stmt
            .query_map([], |row| {
                Ok(TorrentRecord {
                    id: row.get(0)?,
                    info_hash: row.get(1)?,
                    name: row.get(2)?,
                    state: parse_state(&row.get::<_, String>(3)?),
                    policy: parse_policy(&row.get::<_, String>(4)?),
                    total_bytes: row.get::<_, i64>(5)? as u64,
                    downloaded_bytes: row.get::<_, i64>(6)? as u64,
                    uploaded_bytes: row.get::<_, i64>(7)? as u64,
                    save_path: row.get(8)?,
                    magnet_uri: row.get(9)?,
                    added_at: row.get(10)?,
                    completed_at: row.get(11)?,
                    download_speed_limit: row.get::<_, i64>(12)? as u64,
                    upload_speed_limit: row.get::<_, i64>(13)? as u64,
                    max_peers: row.get::<_, i32>(14)? as u32,
                    ratio_limit: row.get(15)?,
                    error_message: row.get(16)?,
                })
            })
            .map_err(|e| TsubasaError::Database(DatabaseError::Sqlite(e)))?;

        let mut result = Vec::new();
        for record in records {
            result.push(record.map_err(|e| TsubasaError::Database(DatabaseError::Sqlite(e)))?);
        }
        Ok(result)
    }

    /// Get a single torrent by ID.
    pub fn get_torrent(&self, id: &str) -> crate::error::Result<TorrentRecord> {
        let conn = self.conn.lock();
        conn.query_row(
            "SELECT id, info_hash, name, state, policy, total_bytes, downloaded_bytes,
             uploaded_bytes, save_path, magnet_uri, added_at, completed_at,
             download_speed_limit, upload_speed_limit, max_peers, ratio_limit, error_message
             FROM torrents WHERE id = ?1",
            params![id],
            |row| {
                Ok(TorrentRecord {
                    id: row.get(0)?,
                    info_hash: row.get(1)?,
                    name: row.get(2)?,
                    state: parse_state(&row.get::<_, String>(3)?),
                    policy: parse_policy(&row.get::<_, String>(4)?),
                    total_bytes: row.get::<_, i64>(5)? as u64,
                    downloaded_bytes: row.get::<_, i64>(6)? as u64,
                    uploaded_bytes: row.get::<_, i64>(7)? as u64,
                    save_path: row.get(8)?,
                    magnet_uri: row.get(9)?,
                    added_at: row.get(10)?,
                    completed_at: row.get(11)?,
                    download_speed_limit: row.get::<_, i64>(12)? as u64,
                    upload_speed_limit: row.get::<_, i64>(13)? as u64,
                    max_peers: row.get::<_, i32>(14)? as u32,
                    ratio_limit: row.get(15)?,
                    error_message: row.get(16)?,
                })
            },
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => {
                TsubasaError::Database(DatabaseError::NotFound(format!("Torrent {id}")))
            }
            other => TsubasaError::Database(DatabaseError::Sqlite(other)),
        })
    }

    /// Update torrent state.
    pub fn update_torrent_state(
        &self,
        id: &str,
        state: &str,
        error_msg: Option<&str>,
    ) -> crate::error::Result<()> {
        let conn = self.conn.lock();
        conn.execute(
            "UPDATE torrents SET state = ?1, error_message = ?2 WHERE id = ?3",
            params![state, error_msg, id],
        )
        .map_err(|e| TsubasaError::Database(DatabaseError::Sqlite(e)))?;
        Ok(())
    }

    /// Update torrent progress.
    pub fn update_torrent_progress(
        &self,
        id: &str,
        downloaded: u64,
        uploaded: u64,
        total: u64,
    ) -> crate::error::Result<()> {
        let conn = self.conn.lock();
        conn.execute(
            "UPDATE torrents SET downloaded_bytes = ?1, uploaded_bytes = ?2, total_bytes = ?3 WHERE id = ?4",
            params![downloaded as i64, uploaded as i64, total as i64, id],
        )
        .map_err(|e| TsubasaError::Database(DatabaseError::Sqlite(e)))?;
        Ok(())
    }

    /// Remove a torrent record.
    pub fn remove_torrent(&self, id: &str) -> crate::error::Result<()> {
        let conn = self.conn.lock();
        conn.execute("DELETE FROM torrents WHERE id = ?1", params![id])
            .map_err(|e| TsubasaError::Database(DatabaseError::Sqlite(e)))?;
        Ok(())
    }

    // ─── Settings ───────────────────────────────────────────

    /// Get a setting value by key.
    pub fn get_setting(&self, key: &str) -> crate::error::Result<Option<String>> {
        let conn = self.conn.lock();
        let result = conn.query_row(
            "SELECT value FROM settings WHERE key = ?1",
            params![key],
            |row| row.get(0),
        );
        match result {
            Ok(value) => Ok(Some(value)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(TsubasaError::Database(DatabaseError::Sqlite(e))),
        }
    }

    /// Set a setting value (upsert).
    pub fn set_setting(&self, key: &str, value: &str) -> crate::error::Result<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )
        .map_err(|e| TsubasaError::Database(DatabaseError::Sqlite(e)))?;
        Ok(())
    }

    /// Load all settings into an AppSettings struct.
    pub fn load_settings(&self) -> crate::error::Result<AppSettings> {
        let mut settings = AppSettings::default();

        if let Some(v) = self.get_setting("download_dir")? {
            settings.download_dir = v;
        }
        if let Some(v) = self.get_setting("default_policy")? {
            settings.default_policy = parse_policy(&v);
        }
        if let Some(v) = self.get_setting("global_download_limit")? {
            settings.global_download_limit = v.parse().unwrap_or(0);
        }
        if let Some(v) = self.get_setting("global_upload_limit")? {
            settings.global_upload_limit = v.parse().unwrap_or(0);
        }
        if let Some(v) = self.get_setting("max_active_torrents")? {
            settings.max_active_torrents = v.parse().unwrap_or(10);
        }
        if let Some(v) = self.get_setting("max_active_downloads")? {
            settings.max_active_downloads = v.parse().unwrap_or(5);
        }
        if let Some(v) = self.get_setting("max_active_seeds")? {
            settings.max_active_seeds = v.parse().unwrap_or(5);
        }
        if let Some(v) = self.get_setting("default_ratio_limit")? {
            settings.default_ratio_limit = v.parse().unwrap_or(2.0);
        }
        if let Some(v) = self.get_setting("enable_dht")? {
            settings.enable_dht = v == "true";
        }
        if let Some(v) = self.get_setting("enable_pex")? {
            settings.enable_pex = v == "true";
        }
        if let Some(v) = self.get_setting("listen_port")? {
            settings.listen_port = v.parse().unwrap_or(6881);
        }
        if let Some(v) = self.get_setting("torbox_api_key")? {
            settings.torbox_api_key = Some(v);
        }
        if let Some(v) = self.get_setting("realdebrid_api_key")? {
            settings.realdebrid_api_key = Some(v);
        }
        if let Some(v) = self.get_setting("onboarding_completed")? {
            settings.onboarding_completed = v == "true";
        }
        if let Some(v) = self.get_setting("theme")? {
            settings.theme = v;
        }

        Ok(settings)
    }

    /// Save all settings from an AppSettings struct.
    pub fn save_settings(&self, settings: &AppSettings) -> crate::error::Result<()> {
        self.set_setting("download_dir", &settings.download_dir)?;
        self.set_setting("default_policy", &settings.default_policy.to_string())?;
        self.set_setting(
            "global_download_limit",
            &settings.global_download_limit.to_string(),
        )?;
        self.set_setting(
            "global_upload_limit",
            &settings.global_upload_limit.to_string(),
        )?;
        self.set_setting(
            "max_active_torrents",
            &settings.max_active_torrents.to_string(),
        )?;
        self.set_setting(
            "max_active_downloads",
            &settings.max_active_downloads.to_string(),
        )?;
        self.set_setting("max_active_seeds", &settings.max_active_seeds.to_string())?;
        self.set_setting(
            "default_ratio_limit",
            &settings.default_ratio_limit.to_string(),
        )?;
        self.set_setting("enable_dht", &settings.enable_dht.to_string())?;
        self.set_setting("enable_pex", &settings.enable_pex.to_string())?;
        self.set_setting("listen_port", &settings.listen_port.to_string())?;
        if let Some(ref key) = settings.torbox_api_key {
            self.set_setting("torbox_api_key", key)?;
        }
        if let Some(ref key) = settings.realdebrid_api_key {
            self.set_setting("realdebrid_api_key", key)?;
        }
        self.set_setting(
            "onboarding_completed",
            &settings.onboarding_completed.to_string(),
        )?;
        self.set_setting("theme", &settings.theme)?;

        Ok(())
    }

    // ─── Search History ──────────────────────────────────────

    /// Save a search query to the search_history table.
    pub fn save_search_query(&self, query: &str) -> crate::error::Result<()> {
        let conn = self.conn.lock();
        let timestamp = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO search_history (query, timestamp) VALUES (?1, ?2)",
            params![query, timestamp],
        )
        .map_err(|e| TsubasaError::Database(DatabaseError::Sqlite(e)))?;
        Ok(())
    }

    /// Get recent search history entries (most recent first, max 50).
    pub fn get_search_history(
        &self,
    ) -> crate::error::Result<Vec<crate::search::SearchHistoryEntry>> {
        let conn = self.conn.lock();
        let mut stmt = conn
            .prepare("SELECT id, query, timestamp FROM search_history ORDER BY id DESC LIMIT 50")
            .map_err(|e| TsubasaError::Database(DatabaseError::Sqlite(e)))?;

        let entries = stmt
            .query_map([], |row| {
                Ok(crate::search::SearchHistoryEntry {
                    id: row.get(0)?,
                    query: row.get(1)?,
                    timestamp: row.get(2)?,
                })
            })
            .map_err(|e| TsubasaError::Database(DatabaseError::Sqlite(e)))?;

        let mut result = Vec::new();
        for entry in entries {
            result.push(entry.map_err(|e| TsubasaError::Database(DatabaseError::Sqlite(e)))?);
        }
        Ok(result)
    }

    /// Clear all search history.
    pub fn clear_search_history(&self) -> crate::error::Result<()> {
        let conn = self.conn.lock();
        conn.execute("DELETE FROM search_history", [])
            .map_err(|e| TsubasaError::Database(DatabaseError::Sqlite(e)))?;
        Ok(())
    }
}

// ─── Helpers ────────────────────────────────────────────────

fn parse_state(s: &str) -> crate::download::state_machine::TorrentState {
    use crate::download::state_machine::TorrentState;
    match s {
        "pending" => TorrentState::Pending,
        "checking" => TorrentState::Checking,
        "queued" => TorrentState::Queued,
        "downloading" => TorrentState::Downloading,
        "paused" => TorrentState::Paused,
        "completed" => TorrentState::Completed,
        "seeding" => TorrentState::Seeding,
        "stopped" => TorrentState::Stopped,
        "errored" => TorrentState::Errored,
        _ => TorrentState::Errored, // Unknown states become errors
    }
}

fn parse_policy(s: &str) -> crate::download::state_machine::DownloadPolicy {
    use crate::download::state_machine::DownloadPolicy;
    match s {
        "local_only" => DownloadPolicy::LocalOnly,
        "cloud_only" => DownloadPolicy::CloudOnly,
        "hybrid" => DownloadPolicy::Hybrid,
        _ => DownloadPolicy::LocalOnly,
    }
}
