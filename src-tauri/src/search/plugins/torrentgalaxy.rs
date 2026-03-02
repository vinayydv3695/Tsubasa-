// Tsubasa (翼) — TorrentGalaxy Search Plugin
// General-purpose torrent indexer. HTML scraping.
// No API key required.

use async_trait::async_trait;

use super::plugin::{PluginSearchResult, SearchCategory, SearchPlugin};

pub struct TorrentGalaxyPlugin;

impl TorrentGalaxyPlugin {
    fn category_id(cat: &SearchCategory) -> &str {
        match cat {
            SearchCategory::Movies => "c1=1",
            SearchCategory::TV => "c3=1",
            SearchCategory::Music => "c22=1",
            SearchCategory::Games => "c10=1",
            SearchCategory::Software => "c18=1",
            SearchCategory::Anime => "c28=1",
            SearchCategory::Books => "c13=1",
            _ => "",
        }
    }

    fn parse_results(html: &str) -> Vec<PluginSearchResult> {
        let mut results = Vec::new();

        // TorrentGalaxy uses a <div class="tgxtablerow"> structure
        let rows: Vec<&str> = html.split("tgxtablerow").skip(1).collect();

        for row in rows {
            // Extract title from <a class="txlight" title="..." href="/torrent/...">
            let title = Self::find_attr(row, "txlight", "title")
                .unwrap_or_default();
            if title.is_empty() { continue; }

            // Extract torrent detail path
            let detail_href = Self::find_attr(row, "txlight", "href");

            // Extract magnet link
            let magnet = if let Some(pos) = row.find("magnet:?") {
                let end = row[pos..].find('"').or(row[pos..].find('\'')).unwrap_or(row[pos..].len());
                Some(row[pos..pos + end].to_string())
            } else {
                None
            };

            // Extract info_hash from magnet
            let info_hash = magnet.as_ref().and_then(|m| {
                m.find("btih:").map(|pos| {
                    let start = pos + 5;
                    let end = m[start..].find('&').unwrap_or(m[start..].len());
                    m[start..start + end].to_lowercase()
                })
            });

            // Extract seeders/leechers
            // Look for <span ... color: green>N</span> (seeders) and color: red (leechers)
            let seeders = Self::find_colored_number(row, "green").unwrap_or(0);
            let leechers = Self::find_colored_number(row, "red").unwrap_or(0);

            // Extract size
            let size_str = Self::find_size(row).unwrap_or_default();
            let size_bytes = Self::parse_size(&size_str);

            results.push(PluginSearchResult {
                title: title.clone(),
                magnet,
                torrent_url: None,
                info_hash,
                size_bytes,
                seeders,
                leechers,
                upload_date: None,
                source: "torrentgalaxy".to_string(),
                category: None,
                source_url: detail_href
                    .map(|h| format!("https://torrentgalaxy.to{}", h))
                    .unwrap_or_default(),
            });
        }

        results
    }

    fn find_attr<'a>(html: &'a str, class_contains: &str, attr: &str) -> Option<String> {
        let class_pos = html.find(class_contains)?;
        // Search within a window around the class
        let search_start = if class_pos > 200 { class_pos - 200 } else { 0 };
        let search_end = (class_pos + 500).min(html.len());
        let region = &html[search_start..search_end];

        let attr_needle = format!("{}=\"", attr);
        let attr_pos = region.find(&attr_needle)?;
        let start = attr_pos + attr_needle.len();
        let end = region[start..].find('"')?;
        Some(region[start..start + end].to_string())
    }

    fn find_colored_number(html: &str, color: &str) -> Option<u32> {
        let pattern = format!("color: {}\"", color);
        let pos = html.find(&pattern)?;
        let after = &html[pos + pattern.len()..];
        // Find the next > then extract number
        let tag_end = after.find('>')?;
        let text_start = tag_end + 1;
        let text_end = after[text_start..].find('<')?;
        let num_str = after[text_start..text_start + text_end].trim();
        num_str.replace(',', "").parse().ok()
    }

    fn find_size(html: &str) -> Option<String> {
        // Look for size in "N.N GB" or "N.N MB" patterns
        let patterns = ["GB", "MB", "KB", "TB", "GiB", "MiB"];
        for &unit in &patterns {
            if let Some(pos) = html.find(unit) {
                // Go backwards to find the number
                let before = &html[..pos];
                let trimmed = before.trim_end();
                let num_start = trimmed.rfind(|c: char| !c.is_ascii_digit() && c != '.')
                    .map(|p| p + 1)
                    .unwrap_or(0);
                let num = &trimmed[num_start..];
                if !num.is_empty() {
                    return Some(format!("{} {}", num, unit));
                }
            }
        }
        None
    }

    fn parse_size(s: &str) -> u64 {
        let parts: Vec<&str> = s.trim().split_whitespace().collect();
        if parts.len() < 2 { return 0; }
        let num: f64 = parts[0].parse().unwrap_or(0.0);
        match parts[1].to_uppercase().as_str() {
            "KB" | "KIB" => (num * 1024.0) as u64,
            "MB" | "MIB" => (num * 1024.0 * 1024.0) as u64,
            "GB" | "GIB" => (num * 1024.0 * 1024.0 * 1024.0) as u64,
            "TB" | "TIB" => (num * 1024.0 * 1024.0 * 1024.0 * 1024.0) as u64,
            _ => 0,
        }
    }
}

#[async_trait]
impl SearchPlugin for TorrentGalaxyPlugin {
    fn id(&self) -> &str { "torrentgalaxy" }
    fn name(&self) -> &str { "TorrentGalaxy" }

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
        page: u32,
        client: &reqwest::Client,
    ) -> Result<Vec<PluginSearchResult>, String> {
        let cat = category.unwrap_or(SearchCategory::All);
        let cat_param = Self::category_id(&cat);

        let url = if cat_param.is_empty() {
            format!(
                "https://torrentgalaxy.to/torrents.php?search={}&sort=seeders&order=desc&page={}",
                urlencoding::encode(query),
                page
            )
        } else {
            format!(
                "https://torrentgalaxy.to/torrents.php?search={}&{}&sort=seeders&order=desc&page={}",
                urlencoding::encode(query),
                cat_param,
                page
            )
        };

        let resp = client
            .get(&url)
            .header("User-Agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36")
            .send()
            .await
            .map_err(|e| format!("TorrentGalaxy request failed: {e}"))?;

        if !resp.status().is_success() {
            return Err(format!("TorrentGalaxy returned HTTP {}", resp.status()));
        }

        let html = resp.text().await
            .map_err(|e| format!("Failed to read TorrentGalaxy response: {e}"))?;

        Ok(Self::parse_results(&html))
    }
}
