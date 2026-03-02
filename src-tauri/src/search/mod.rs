// Tsubasa (翼) — Search
// Torbox Search API integration via search-api.torbox.app.
// Also includes the plugin-based search aggregator for direct indexer scraping.

pub mod plugin;
pub mod plugins;
pub mod aggregator;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::cloud::provider::DebridProvider;
use crate::cloud::CloudManager;
use crate::error::TsubasaError;
use crate::storage::database::Database;

// ─── Search result types ────────────────────────────────

/// A single torrent search result from the Torbox Search API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Torrent name / title.
    pub name: String,
    /// Info hash (lowercase hex).
    pub info_hash: String,
    /// Total size in bytes.
    pub size: u64,
    /// Number of seeders.
    pub seeders: u32,
    /// Number of leechers.
    pub leechers: u32,
    /// Source tracker / indexer name.
    pub source: String,
    /// Category (e.g. "movies", "tv", "music", "software", "other").
    pub category: String,
    /// Magnet URI (constructed from info_hash + name).
    pub magnet_uri: String,
    /// Whether this torrent is cached on Torbox (if check_cache was enabled).
    pub cached: Option<bool>,
    /// When the torrent was first seen / uploaded (ISO timestamp if available).
    pub uploaded_at: Option<String>,
}

/// Search history entry from the database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHistoryEntry {
    pub id: i64,
    pub query: String,
    pub timestamp: String,
}

// ─── Torbox Search API response shapes ──────────────────

/// Raw torrent result from the Torbox search API.
/// Fields are optional because different indexers return different data.
#[derive(Debug, Deserialize)]
struct RawSearchResult {
    name: Option<String>,
    hash: Option<String>,
    size: Option<u64>,
    seeders: Option<u32>,
    leechers: Option<u32>,
    source: Option<String>,
    category: Option<String>,
    magnet: Option<String>,
    cached: Option<bool>,
    #[serde(alias = "updated_at", alias = "last_known_date")]
    uploaded_at: Option<String>,
}

/// Envelope wrapper — the search API uses a similar envelope pattern.
#[derive(Debug, Deserialize)]
struct SearchApiResponse {
    success: Option<bool>,
    data: Option<SearchData>,
    detail: Option<String>,
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SearchData {
    torrents: Option<Vec<RawSearchResult>>,
}

// ─── Search engine ──────────────────────────────────────

/// Search engine backed by the Torbox Search API.
pub struct SearchEngine {
    cloud_manager: Arc<CloudManager>,
    db: Database,
    client: reqwest::Client,
    base_url: String,
}

impl SearchEngine {
    pub fn new(cloud_manager: Arc<CloudManager>, db: Database) -> Self {
        Self {
            cloud_manager,
            db,
            client: reqwest::Client::builder()
                .user_agent("Tsubasa/0.1.0")
                .build()
                .unwrap_or_else(|_| reqwest::Client::new()),
            base_url: "https://search-api.torbox.app".to_string(),
        }
    }

    /// Search for torrents using the Torbox Search API.
    ///
    /// Requires a configured Torbox API key.
    /// `check_cache` — if true, includes cache status per result (slower).
    pub async fn search(
        &self,
        query: &str,
        check_cache: bool,
    ) -> crate::error::Result<Vec<SearchResult>> {
        let api_key = self.get_torbox_api_key()?;

        let encoded_query = urlencoding::encode(query);
        let url = format!("{}/torrents/search/{}", self.base_url, encoded_query);

        let resp = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {api_key}"))
            .query(&[
                ("metadata", "false"),
                ("check_cache", if check_cache { "true" } else { "false" }),
                ("check_owned", "false"),
            ])
            .send()
            .await
            .map_err(|e| TsubasaError::Search(format!("Search request failed: {e}")))?;

        let status = resp.status();

        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(TsubasaError::Search(
                "Search rate limited. Please wait before searching again.".to_string(),
            ));
        }

        if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
            return Err(TsubasaError::Search(
                "Torbox API key is invalid or expired.".to_string(),
            ));
        }

        if !status.is_success() {
            return Err(TsubasaError::Search(format!(
                "Search API returned HTTP {status}"
            )));
        }

        let body = resp
            .text()
            .await
            .map_err(|e| TsubasaError::Search(format!("Failed to read search response: {e}")))?;

        let envelope: SearchApiResponse =
            serde_json::from_str(&body).map_err(|e| TsubasaError::Search(format!(
                "Failed to parse search response: {e}"
            )))?;

        // Check for API-level errors
        if let Some(false) = envelope.success {
            let msg = envelope
                .error
                .or(envelope.detail)
                .unwrap_or_else(|| "Unknown search API error".to_string());
            return Err(TsubasaError::Search(msg));
        }

        let raw_results = envelope
            .data
            .and_then(|d| d.torrents)
            .unwrap_or_default();

        let results: Vec<SearchResult> = raw_results
            .into_iter()
            .filter_map(|raw| {
                let name = raw.name.unwrap_or_default();
                let info_hash = raw.hash.unwrap_or_default().to_lowercase();

                // Skip results without a name or info_hash
                if name.is_empty() || info_hash.is_empty() {
                    return None;
                }

                let magnet_uri = raw.magnet.unwrap_or_else(|| {
                    format!(
                        "magnet:?xt=urn:btih:{}&dn={}",
                        info_hash,
                        urlencoding::encode(&name)
                    )
                });

                Some(SearchResult {
                    name,
                    info_hash,
                    size: raw.size.unwrap_or(0),
                    seeders: raw.seeders.unwrap_or(0),
                    leechers: raw.leechers.unwrap_or(0),
                    source: raw.source.unwrap_or_else(|| "Unknown".to_string()),
                    category: raw.category.unwrap_or_else(|| "other".to_string()),
                    magnet_uri,
                    cached: raw.cached,
                    uploaded_at: raw.uploaded_at,
                })
            })
            .collect();

        Ok(results)
    }

    /// Save a search query to history.
    pub fn save_search_history(&self, query: &str) -> crate::error::Result<()> {
        self.db.save_search_query(query)
    }

    /// Get recent search history (most recent first, limited to 50).
    pub fn get_search_history(&self) -> crate::error::Result<Vec<SearchHistoryEntry>> {
        self.db.get_search_history()
    }

    /// Clear all search history.
    pub fn clear_search_history(&self) -> crate::error::Result<()> {
        self.db.clear_search_history()
    }

    /// Extract the Torbox API key from the cloud manager.
    fn get_torbox_api_key(&self) -> crate::error::Result<String> {
        if !self.cloud_manager.torbox.is_configured() {
            return Err(TsubasaError::Search(
                "Torbox API key is not configured. Set it in Settings to use search.".to_string(),
            ));
        }
        // Read the key from the provider's interior lock
        self.cloud_manager
            .torbox
            .api_key
            .read()
            .clone()
            .ok_or_else(|| TsubasaError::Search("Torbox API key not available".to_string()))
    }
}
