// Tsubasa (翼) — Search Aggregator Commands
// IPC handlers for the plugin-based search system.
// Works alongside existing Torbox Search API commands.

use std::sync::Arc;

use tauri::State;

use crate::app_state::AppState;
use crate::error::TsubasaError;
use crate::search::aggregator::{PluginInfo, SearchAggregator};
use crate::search::plugin::{PluginSearchResult, SearchCategory};

/// Search across all enabled plugins (no API key required).
#[tauri::command]
pub async fn aggregator_search(
    state: State<'_, Arc<AppState>>,
    query: String,
    category: Option<String>,
    plugins: Option<Vec<String>>,
) -> Result<Vec<PluginSearchResult>, TsubasaError> {
    let query = query.trim().to_string();
    if query.is_empty() {
        return Err(TsubasaError::Search("Search query cannot be empty".to_string()));
    }

    let cat = category.as_deref().map(|c| match c {
        "movies" => SearchCategory::Movies,
        "tv" => SearchCategory::TV,
        "music" => SearchCategory::Music,
        "games" => SearchCategory::Games,
        "software" => SearchCategory::Software,
        "anime" => SearchCategory::Anime,
        "books" => SearchCategory::Books,
        _ => SearchCategory::All,
    });

    // Get enabled plugins from settings or use provided list
    let enabled = plugins.unwrap_or_else(|| {
        state.settings_manager.search().enabled_plugins
    });

    let aggregator = SearchAggregator::new();

    tracing::info!(query = %query, plugins = ?enabled, "Aggregator search started");

    let results = aggregator.search_all(&query, cat, &enabled).await;

    tracing::info!(query = %query, results = results.len(), "Aggregator search completed");
    Ok(results)
}

/// Get available search plugins and their categories.
#[tauri::command]
pub async fn get_search_plugins(
    _state: State<'_, Arc<AppState>>,
) -> Result<Vec<PluginInfo>, TsubasaError> {
    let aggregator = SearchAggregator::new();
    Ok(aggregator.plugin_info())
}
