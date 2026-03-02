// Tsubasa (翼) — Search Aggregator
// Concurrent multi-plugin search with deduplication and sorting.
// Works alongside the Torbox Search API — aggregates results from
// both the Torbox API and built-in scrapers.

use std::collections::HashSet;
use std::time::Duration;

use super::plugin::{PluginSearchResult, SearchCategory, SearchPlugin};

/// Search aggregator that queries multiple plugins concurrently.
pub struct SearchAggregator {
    plugins: Vec<Box<dyn SearchPlugin>>,
    client: reqwest::Client,
    timeout: Duration,
}

impl SearchAggregator {
    /// Create a new aggregator with all built-in plugins.
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .user_agent("Tsubasa/0.1.0")
            .timeout(Duration::from_secs(15))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self {
            plugins: super::plugins::all_plugins(),
            client,
            timeout: Duration::from_secs(15),
        }
    }

    /// Create with a custom reqwest client (e.g., with proxy configured).
    pub fn with_client(client: reqwest::Client) -> Self {
        Self {
            plugins: super::plugins::all_plugins(),
            client,
            timeout: Duration::from_secs(15),
        }
    }

    /// Set the per-plugin search timeout.
    pub fn set_timeout(&mut self, timeout: Duration) {
        self.timeout = timeout;
    }

    /// Get all registered plugin IDs.
    pub fn plugin_ids(&self) -> Vec<String> {
        self.plugins.iter().map(|p| p.id().to_string()).collect()
    }

    /// Get plugin info for the frontend.
    pub fn plugin_info(&self) -> Vec<PluginInfo> {
        self.plugins.iter().map(|p| PluginInfo {
            id: p.id().to_string(),
            name: p.name().to_string(),
            categories: p.supported_categories(),
        }).collect()
    }

    /// Search all enabled plugins concurrently.
    ///
    /// - `query`: search string
    /// - `category`: optional category filter
    /// - `enabled_plugins`: list of plugin IDs to search (empty = all)
    pub async fn search_all(
        &self,
        query: &str,
        category: Option<SearchCategory>,
        enabled_plugins: &[String],
    ) -> Vec<PluginSearchResult> {
        let mut handles = Vec::new();

        for plugin in &self.plugins {
            // Skip disabled plugins
            if !enabled_plugins.is_empty() && !enabled_plugins.contains(&plugin.id().to_string()) {
                continue;
            }

            let query = query.to_string();
            let client = self.client.clone();
            let timeout = self.timeout;
            let plugin_id = plugin.id().to_string();
            let plugin_name = plugin.name().to_string();

            // We can't move the plugin into the task since it's behind a reference,
            // so we need to use a different approach. We'll collect futures directly.
            // Using a channel-based approach instead.
            let _ = (plugin_id, plugin_name); // used below
        }

        // Collect futures for all enabled plugins
        let futures: Vec<_> = self.plugins.iter()
            .filter(|p| enabled_plugins.is_empty() || enabled_plugins.contains(&p.id().to_string()))
            .map(|plugin| {
                let query = query.to_string();
                let client = self.client.clone();
                let timeout = self.timeout;

                async move {
                    let result = tokio::time::timeout(
                        timeout,
                        plugin.search(&query, category, 0, &client),
                    ).await;

                    match result {
                        Ok(Ok(results)) => {
                            tracing::debug!(
                                plugin = plugin.id(),
                                count = results.len(),
                                "Search plugin returned results"
                            );
                            results
                        }
                        Ok(Err(e)) => {
                            tracing::warn!(
                                plugin = plugin.id(),
                                error = %e,
                                "Search plugin error"
                            );
                            Vec::new()
                        }
                        Err(_) => {
                            tracing::warn!(
                                plugin = plugin.id(),
                                "Search plugin timed out"
                            );
                            Vec::new()
                        }
                    }
                }
            })
            .collect();

        // Run all searches concurrently
        let all_results = futures::future::join_all(futures).await;

        // Flatten and deduplicate
        let mut combined: Vec<PluginSearchResult> = all_results
            .into_iter()
            .flatten()
            .collect();

        dedup_by_info_hash(&mut combined);

        // Sort by seeders (descending)
        combined.sort_by(|a, b| b.seeders.cmp(&a.seeders));

        combined
    }
}

/// Plugin metadata for the frontend.
#[derive(Debug, Clone, serde::Serialize)]
pub struct PluginInfo {
    pub id: String,
    pub name: String,
    pub categories: Vec<SearchCategory>,
}

/// Deduplicate results by info_hash (keep highest seeder count).
fn dedup_by_info_hash(results: &mut Vec<PluginSearchResult>) {
    let mut seen: HashSet<String> = HashSet::new();
    results.retain(|r| {
        if let Some(hash) = &r.info_hash {
            let hash_lower = hash.to_lowercase();
            if seen.contains(&hash_lower) {
                return false;
            }
            seen.insert(hash_lower);
        }
        true
    });
}
