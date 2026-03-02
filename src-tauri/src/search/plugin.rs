// Tsubasa (翼) — Search Plugin Trait
// Defines the interface for torrent search plugins.
// Each plugin scrapes or queries a specific torrent indexer.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Search result category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SearchCategory {
    All,
    Movies,
    TV,
    Music,
    Games,
    Software,
    Anime,
    Books,
    Other,
}

impl SearchCategory {
    pub fn as_str(&self) -> &str {
        match self {
            Self::All => "all",
            Self::Movies => "movies",
            Self::TV => "tv",
            Self::Music => "music",
            Self::Games => "games",
            Self::Software => "software",
            Self::Anime => "anime",
            Self::Books => "books",
            Self::Other => "other",
        }
    }
}

/// A single torrent search result from a plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginSearchResult {
    /// Torrent title.
    pub title: String,
    /// Magnet URI (if available).
    pub magnet: Option<String>,
    /// .torrent download URL (if available).
    pub torrent_url: Option<String>,
    /// Info hash (lowercase hex, if available).
    pub info_hash: Option<String>,
    /// Total size in bytes (0 if unknown).
    pub size_bytes: u64,
    /// Number of seeders.
    pub seeders: u32,
    /// Number of leechers.
    pub leechers: u32,
    /// Upload date (human-readable string).
    pub upload_date: Option<String>,
    /// Plugin ID that produced this result.
    pub source: String,
    /// Category of the torrent.
    pub category: Option<String>,
    /// Link to the original page on the indexer.
    pub source_url: String,
}

/// Trait for torrent search plugins.
/// Each implementation scrapes/queries one torrent indexer.
#[async_trait]
pub trait SearchPlugin: Send + Sync {
    /// Unique plugin identifier (e.g., "piratebay", "leet", "nyaa").
    fn id(&self) -> &str;

    /// Human-readable display name (e.g., "The Pirate Bay", "1337x").
    fn name(&self) -> &str;

    /// Categories this plugin supports.
    fn supported_categories(&self) -> Vec<SearchCategory>;

    /// Execute a search query.
    ///
    /// - `query`: search string
    /// - `category`: optional category filter
    /// - `page`: page number (0-indexed)
    /// - `client`: pre-configured reqwest client (may have proxy set)
    async fn search(
        &self,
        query: &str,
        category: Option<SearchCategory>,
        page: u32,
        client: &reqwest::Client,
    ) -> Result<Vec<PluginSearchResult>, String>;
}
