// Tsubasa — Search Commands
// IPC handlers for torrent search and search history.

use std::sync::Arc;

use tauri::State;

use crate::app_state::AppState;
use crate::error::TsubasaError;
use crate::search::{SearchHistoryEntry, SearchResult};

/// Search for torrents using the Torbox Search API.
#[tauri::command]
pub async fn search_torrents(
    state: State<'_, Arc<AppState>>,
    query: String,
    check_cache: Option<bool>,
) -> Result<Vec<SearchResult>, TsubasaError> {
    let query = query.trim().to_string();
    if query.is_empty() {
        return Err(TsubasaError::Search("Search query cannot be empty".to_string()));
    }

    tracing::info!(query = %query, "Searching torrents");

    let results = state
        .search_engine
        .search(&query, check_cache.unwrap_or(false))
        .await?;

    tracing::info!(query = %query, results = results.len(), "Search completed");
    Ok(results)
}

/// Save a search query to history.
#[tauri::command]
pub async fn save_search_history(
    state: State<'_, Arc<AppState>>,
    query: String,
) -> Result<(), TsubasaError> {
    let query = query.trim().to_string();
    if query.is_empty() {
        return Ok(());
    }
    state.search_engine.save_search_history(&query)
}

/// Get recent search history entries.
#[tauri::command]
pub async fn get_search_history(
    state: State<'_, Arc<AppState>>,
) -> Result<Vec<SearchHistoryEntry>, TsubasaError> {
    state.search_engine.get_search_history()
}

/// Clear all search history.
#[tauri::command]
pub async fn clear_search_history(
    state: State<'_, Arc<AppState>>,
) -> Result<(), TsubasaError> {
    state.search_engine.clear_search_history()
}
