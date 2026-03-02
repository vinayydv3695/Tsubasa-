// Tsubasa (翼) — PirateBay Search Plugin
// Uses the public apibay.org JSON API (no API key required).

use async_trait::async_trait;
use serde::Deserialize;

use super::plugin::{PluginSearchResult, SearchCategory, SearchPlugin};

pub struct PirateBayPlugin;

#[derive(Debug, Deserialize)]
struct PbResult {
    id: String,
    name: String,
    info_hash: String,
    leechers: String,
    seeders: String,
    num_files: String,
    size: String,
    username: String,
    added: String,
    category: String,
}

impl PirateBayPlugin {
    fn category_code(cat: &SearchCategory) -> &str {
        match cat {
            SearchCategory::Movies => "201,207",
            SearchCategory::TV => "205,208",
            SearchCategory::Music => "101",
            SearchCategory::Games => "400",
            SearchCategory::Software => "300",
            SearchCategory::Anime => "205",
            SearchCategory::Books => "601",
            _ => "0",
        }
    }
}

#[async_trait]
impl SearchPlugin for PirateBayPlugin {
    fn id(&self) -> &str { "piratebay" }
    fn name(&self) -> &str { "The Pirate Bay" }

    fn supported_categories(&self) -> Vec<SearchCategory> {
        vec![
            SearchCategory::All,
            SearchCategory::Movies,
            SearchCategory::TV,
            SearchCategory::Music,
            SearchCategory::Games,
            SearchCategory::Software,
            SearchCategory::Anime,
            SearchCategory::Books,
        ]
    }

    async fn search(
        &self,
        query: &str,
        category: Option<SearchCategory>,
        _page: u32,
        client: &reqwest::Client,
    ) -> Result<Vec<PluginSearchResult>, String> {
        let cat = category.unwrap_or(SearchCategory::All);
        let cat_code = Self::category_code(&cat);

        let url = format!(
            "https://apibay.org/q.php?q={}&cat={}",
            urlencoding::encode(query),
            cat_code
        );

        let resp = client
            .get(&url)
            .header("User-Agent", "Tsubasa/0.1.0")
            .send()
            .await
            .map_err(|e| format!("PirateBay request failed: {e}"))?;

        if !resp.status().is_success() {
            return Err(format!("PirateBay API returned HTTP {}", resp.status()));
        }

        let body = resp.text().await
            .map_err(|e| format!("Failed to read PirateBay response: {e}"))?;

        let items: Vec<PbResult> = serde_json::from_str(&body)
            .map_err(|e| format!("Failed to parse PirateBay response: {e}"))?;

        let results: Vec<PluginSearchResult> = items
            .into_iter()
            .filter(|r| r.name != "No results returned")
            .map(|r| {
                let info_hash = r.info_hash.to_lowercase();
                let seeders = r.seeders.parse::<u32>().unwrap_or(0);
                let leechers = r.leechers.parse::<u32>().unwrap_or(0);
                let size_bytes = r.size.parse::<u64>().unwrap_or(0);

                let magnet = format!(
                    "magnet:?xt=urn:btih:{}&dn={}",
                    info_hash,
                    urlencoding::encode(&r.name)
                );

                PluginSearchResult {
                    title: r.name.clone(),
                    magnet: Some(magnet),
                    torrent_url: None,
                    info_hash: Some(info_hash.clone()),
                    size_bytes,
                    seeders,
                    leechers,
                    upload_date: Some(r.added),
                    source: "piratebay".to_string(),
                    category: Some(r.category),
                    source_url: format!("https://thepiratebay.org/description.php?id={}", r.id),
                }
            })
            .collect();

        Ok(results)
    }
}
