// Tsubasa (翼) — Session File I/O
// Binary session files for hot torrent state (piece bitmaps, peer cache).
// This avoids writing high-frequency data to SQLite.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Session data for a single torrent.
/// Serialized to disk periodically and on shutdown.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TorrentSession {
    pub info_hash: String,
    /// Bitfield of completed pieces
    pub piece_bitmap: Vec<u8>,
    /// Known peer addresses for fast resume
    pub peer_cache: Vec<String>,
    /// Total bytes downloaded (for progress tracking across sessions)
    pub downloaded_bytes: u64,
    /// Total bytes uploaded
    pub uploaded_bytes: u64,
    /// Timestamp of last save
    pub last_saved: String,
}

/// Manages session file persistence.
pub struct SessionManager {
    session_dir: PathBuf,
}

impl SessionManager {
    /// Create a new session manager using the given directory.
    pub fn new(session_dir: PathBuf) -> crate::error::Result<Self> {
        std::fs::create_dir_all(&session_dir)?;
        Ok(Self { session_dir })
    }

    /// Get the file path for a torrent's session data.
    fn session_path(&self, info_hash: &str) -> PathBuf {
        self.session_dir.join(format!("{info_hash}.session"))
    }

    /// Save session data for a torrent.
    pub fn save_session(&self, session: &TorrentSession) -> crate::error::Result<()> {
        let path = self.session_path(&session.info_hash);
        let data = serde_json::to_vec(session)?;
        std::fs::write(&path, data)?;
        Ok(())
    }

    /// Load session data for a torrent. Returns None if no session file exists.
    pub fn load_session(&self, info_hash: &str) -> crate::error::Result<Option<TorrentSession>> {
        let path = self.session_path(info_hash);
        if !path.exists() {
            return Ok(None);
        }
        let data = std::fs::read(&path)?;
        let session: TorrentSession = serde_json::from_slice(&data)?;
        Ok(Some(session))
    }

    /// Remove session data for a torrent.
    pub fn remove_session(&self, info_hash: &str) -> crate::error::Result<()> {
        let path = self.session_path(info_hash);
        if path.exists() {
            std::fs::remove_file(&path)?;
        }
        Ok(())
    }

    /// Get the session directory path.
    pub fn session_dir(&self) -> &Path {
        &self.session_dir
    }
}
