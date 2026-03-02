// Tsubasa (翼) — 1337x Search Plugin
// HTML scraping from 1337x.to.
// Parses search results page for torrent info, then fetches magnet from detail page.

use async_trait::async_trait;

use super::plugin::{PluginSearchResult, SearchCategory, SearchPlugin};

pub struct LeetPlugin;

impl LeetPlugin {
    fn category_path(cat: &SearchCategory) -> &str {
        match cat {
            SearchCategory::Movies => "/category-search/{query}/Movies/{page}/",
            SearchCategory::TV => "/category-search/{query}/TV/{page}/",
            SearchCategory::Music => "/category-search/{query}/Music/{page}/",
            SearchCategory::Games => "/category-search/{query}/Games/{page}/",
            SearchCategory::Software => "/category-search/{query}/Apps/{page}/",
            SearchCategory::Anime => "/category-search/{query}/Anime/{page}/",
            _ => "/search/{query}/{page}/",
        }
    }

    /// Extract text between two markers in HTML.
    fn extract_between<'a>(html: &'a str, start: &str, end: &str) -> Option<&'a str> {
        let start_pos = html.find(start)?;
        let after_start = start_pos + start.len();
        let end_pos = html[after_start..].find(end)?;
        Some(&html[after_start..after_start + end_pos])
    }

    /// Parse search results from the HTML table.
    fn parse_search_results(html: &str) -> Vec<PartialResult> {
        let mut results = Vec::new();

        // Find all torrent links in the search results table
        // Pattern: <a href="/torrent/..." class="...">title</a>
        let mut pos = 0;
        while let Some(link_start) = html[pos..].find("<a href=\"/torrent/") {
            let abs_start = pos + link_start;
            let href_start = abs_start + "<a href=\"".len();

            // Extract the href
            if let Some(href_end_offset) = html[href_start..].find('"') {
                let href = &html[href_start..href_start + href_end_offset];

                // Extract the title (text between > and </a>)
                if let Some(tag_end) = html[href_start..].find('>') {
                    let text_start = href_start + tag_end + 1;
                    if let Some(text_end) = html[text_start..].find("</a>") {
                        let title = html[text_start..text_start + text_end].trim().to_string();

                        // Skip empty titles or navigation links
                        if !title.is_empty() && !title.contains('<') {
                            results.push(PartialResult {
                                title,
                                detail_path: href.to_string(),
                                seeders: 0,
                                leechers: 0,
                                size: String::new(),
                                upload_date: None,
                            });
                        }
                    }
                }
            }

            pos = abs_start + 1;
        }

        // Try to parse seeders/leechers from the table cells
        // The table has columns: Name, SE, LE, Time, Size, Uploader
        // For simplicity, we parse <td class="coll-2 seeds">N</td> and <td class="coll-3 leeches">N</td>
        let mut result_idx = 0;
        let mut search_pos = 0;

        while let Some(seed_start) = html[search_pos..].find("class=\"coll-2 seeds\">") {
            let abs_pos = search_pos + seed_start + "class=\"coll-2 seeds\">".len();
            if let Some(seed_end) = html[abs_pos..].find('<') {
                let seeds_str = html[abs_pos..abs_pos + seed_end].trim();
                if let Ok(seeds) = seeds_str.parse::<u32>() {
                    if result_idx < results.len() {
                        results[result_idx].seeders = seeds;
                    }
                }
            }

            // Parse leechers (right after seeds)
            if let Some(leech_start) = html[abs_pos..].find("class=\"coll-3 leeches\">") {
                let leech_abs = abs_pos + leech_start + "class=\"coll-3 leeches\">".len();
                if let Some(leech_end) = html[leech_abs..].find('<') {
                    let leeches_str = html[leech_abs..leech_abs + leech_end].trim();
                    if let Ok(leeches) = leeches_str.parse::<u32>() {
                        if result_idx < results.len() {
                            results[result_idx].leechers = leeches;
                        }
                    }
                }
            }

            // Parse size
            if let Some(size_start) = html[abs_pos..].find("class=\"coll-4 size") {
                let size_abs = abs_pos + size_start;
                if let Some(tag_end) = html[size_abs..].find('>') {
                    let text_start = size_abs + tag_end + 1;
                    if let Some(text_end) = html[text_start..].find('<') {
                        let size = html[text_start..text_start + text_end].trim().to_string();
                        if result_idx < results.len() {
                            results[result_idx].size = size;
                        }
                    }
                }
            }

            result_idx += 1;
            search_pos = abs_pos + 1;
        }

        results
    }

    /// Fetch magnet link from a torrent detail page.
    async fn fetch_magnet(client: &reqwest::Client, path: &str) -> Option<String> {
        let url = format!("https://1337x.to{}", path);
        let resp = client.get(&url)
            .header("User-Agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36")
            .send()
            .await
            .ok()?;

        let html = resp.text().await.ok()?;

        // Find magnet link
        if let Some(magnet_start) = html.find("magnet:?") {
            if let Some(magnet_end) = html[magnet_start..].find('"') {
                return Some(html[magnet_start..magnet_start + magnet_end].to_string());
            }
        }

        None
    }

    /// Parse size string like "1.5 GB" into bytes.
    fn parse_size(s: &str) -> u64 {
        let parts: Vec<&str> = s.trim().split_whitespace().collect();
        if parts.len() < 2 { return 0; }

        let num: f64 = parts[0].parse().unwrap_or(0.0);
        let unit = parts[1].to_uppercase();

        match unit.as_str() {
            "B" => num as u64,
            "KB" => (num * 1024.0) as u64,
            "MB" => (num * 1024.0 * 1024.0) as u64,
            "GB" => (num * 1024.0 * 1024.0 * 1024.0) as u64,
            "TB" => (num * 1024.0 * 1024.0 * 1024.0 * 1024.0) as u64,
            _ => 0,
        }
    }
}

struct PartialResult {
    title: String,
    detail_path: String,
    seeders: u32,
    leechers: u32,
    size: String,
    upload_date: Option<String>,
}

#[async_trait]
impl SearchPlugin for LeetPlugin {
    fn id(&self) -> &str { "leet" }
    fn name(&self) -> &str { "1337x" }

    fn supported_categories(&self) -> Vec<SearchCategory> {
        vec![
            SearchCategory::All,
            SearchCategory::Movies,
            SearchCategory::TV,
            SearchCategory::Music,
            SearchCategory::Games,
            SearchCategory::Software,
            SearchCategory::Anime,
        ]
    }

    async fn search(
        &self,
        query: &str,
        category: Option<SearchCategory>,
        page: u32,
        client: &reqwest::Client,
    ) -> Result<Vec<PluginSearchResult>, String> {
        let cat = category.unwrap_or(SearchCategory::All);
        let path_template = Self::category_path(&cat);

        let path = path_template
            .replace("{query}", &urlencoding::encode(query))
            .replace("{page}", &(page + 1).to_string());

        let url = format!("https://1337x.to{}", path);

        let resp = client
            .get(&url)
            .header("User-Agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36")
            .send()
            .await
            .map_err(|e| format!("1337x request failed: {e}"))?;

        if !resp.status().is_success() {
            return Err(format!("1337x returned HTTP {}", resp.status()));
        }

        let html = resp.text().await
            .map_err(|e| format!("Failed to read 1337x response: {e}"))?;

        let partial_results = Self::parse_search_results(&html);

        // Fetch magnets concurrently (limit to first 20 to avoid hammering)
        let mut results = Vec::new();
        let fetches: Vec<_> = partial_results.iter()
            .take(20)
            .map(|r| Self::fetch_magnet(client, &r.detail_path))
            .collect();

        let magnets = futures::future::join_all(fetches).await;

        for (i, partial) in partial_results.iter().take(20).enumerate() {
            let magnet = magnets.get(i).cloned().flatten();

            // Extract info_hash from magnet if available
            let info_hash = magnet.as_ref().and_then(|m| {
                m.find("btih:")
                    .map(|pos| {
                        let start = pos + 5;
                        let end = m[start..].find('&').unwrap_or(m[start..].len());
                        m[start..start + end].to_lowercase()
                    })
            });

            results.push(PluginSearchResult {
                title: partial.title.clone(),
                magnet,
                torrent_url: None,
                info_hash,
                size_bytes: Self::parse_size(&partial.size),
                seeders: partial.seeders,
                leechers: partial.leechers,
                upload_date: partial.upload_date.clone(),
                source: "leet".to_string(),
                category: Some(cat.as_str().to_string()),
                source_url: format!("https://1337x.to{}", partial.detail_path),
            });
        }

        Ok(results)
    }
}
