// Tsubasa (翼) — YTS Search Plugin
// Uses the public YTS.mx API (no API key required).
// Movies only — high quality encodes.

use async_trait::async_trait;
use serde::Deserialize;

use super::plugin::{PluginSearchResult, SearchCategory, SearchPlugin};

pub struct YtsPlugin;

#[derive(Debug, Deserialize)]
struct YtsApiResponse {
    status: String,
    data: Option<YtsData>,
}

#[derive(Debug, Deserialize)]
struct YtsData {
    movie_count: Option<u32>,
    movies: Option<Vec<YtsMovie>>,
}

#[derive(Debug, Deserialize)]
struct YtsMovie {
    id: u64,
    title: String,
    year: u32,
    rating: f64,
    genres: Option<Vec<String>>,
    torrents: Option<Vec<YtsTorrent>>,
}

#[derive(Debug, Deserialize)]
struct YtsTorrent {
    url: String,
    hash: String,
    quality: String,
    #[serde(rename = "type")]
    codec_type: Option<String>,
    seeds: u32,
    peers: u32,
    size: String,
    size_bytes: u64,
    date_uploaded: String,
}

#[async_trait]
impl SearchPlugin for YtsPlugin {
    fn id(&self) -> &str { "yts" }
    fn name(&self) -> &str { "YTS" }

    fn supported_categories(&self) -> Vec<SearchCategory> {
        vec![SearchCategory::Movies]
    }

    async fn search(
        &self,
        query: &str,
        _category: Option<SearchCategory>,
        page: u32,
        client: &reqwest::Client,
    ) -> Result<Vec<PluginSearchResult>, String> {
        let url = format!(
            "https://yts.mx/api/v2/list_movies.json?query_term={}&page={}&limit=50&sort_by=seeds",
            urlencoding::encode(query),
            page + 1
        );

        let resp = client
            .get(&url)
            .header("User-Agent", "Tsubasa/0.1.0")
            .send()
            .await
            .map_err(|e| format!("YTS request failed: {e}"))?;

        if !resp.status().is_success() {
            return Err(format!("YTS API returned HTTP {}", resp.status()));
        }

        let body = resp.text().await
            .map_err(|e| format!("Failed to read YTS response: {e}"))?;

        let api_resp: YtsApiResponse = serde_json::from_str(&body)
            .map_err(|e| format!("Failed to parse YTS response: {e}"))?;

        if api_resp.status != "ok" {
            return Err("YTS API returned non-ok status".to_string());
        }

        let movies = api_resp.data
            .and_then(|d| d.movies)
            .unwrap_or_default();

        let mut results = Vec::new();

        for movie in movies {
            let torrents = movie.torrents.unwrap_or_default();
            for t in torrents {
                let info_hash = t.hash.to_lowercase();
                let title = format!("{} ({}) [{}]", movie.title, movie.year, t.quality);

                let magnet = format!(
                    "magnet:?xt=urn:btih:{}&dn={}",
                    info_hash,
                    urlencoding::encode(&title)
                );

                results.push(PluginSearchResult {
                    title,
                    magnet: Some(magnet),
                    torrent_url: Some(t.url),
                    info_hash: Some(info_hash),
                    size_bytes: t.size_bytes,
                    seeders: t.seeds,
                    leechers: t.peers,
                    upload_date: Some(t.date_uploaded),
                    source: "yts".to_string(),
                    category: Some("movies".to_string()),
                    source_url: format!("https://yts.mx/movies/{}", movie.id),
                });
            }
        }

        Ok(results)
    }
}
