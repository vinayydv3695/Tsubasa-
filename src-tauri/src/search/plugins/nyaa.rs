// Tsubasa (翼) — Nyaa.si Search Plugin
// Anime-focused torrent tracker. Uses RSS/HTML scraping.
// No API key required.

use async_trait::async_trait;

use super::plugin::{PluginSearchResult, SearchCategory, SearchPlugin};

pub struct NyaaPlugin;

impl NyaaPlugin {
    fn category_id(cat: &SearchCategory) -> &str {
        match cat {
            SearchCategory::Anime => "1_0",   // Anime - All
            SearchCategory::Music => "2_0",   // Audio - All
            SearchCategory::Books => "3_0",   // Literature - All
            SearchCategory::Software => "6_0", // Software - All
            _ => "0_0",                        // All categories
        }
    }

    /// Parse the Nyaa search results HTML table.
    fn parse_results(html: &str) -> Vec<PluginSearchResult> {
        let mut results = Vec::new();

        // Each result row has class "default", "success", or "danger"
        // We look for <tr class="..."> blocks that contain torrent data
        let rows: Vec<&str> = html.split("<tr class=\"").skip(1).collect();

        for row in rows {
            // Skip header rows
            if row.starts_with("table-responsive") { continue; }

            // Extract title from <a href="/view/..." title="...">
            let title = Self::extract_attr(row, "/view/", "title=\"")
                .unwrap_or_default();
            if title.is_empty() { continue; }

            // Extract magnet link
            let magnet = if let Some(magnet_pos) = row.find("magnet:?") {
                let end = row[magnet_pos..].find('"').unwrap_or(row[magnet_pos..].len());
                Some(row[magnet_pos..magnet_pos + end].to_string())
            } else {
                None
            };

            // Extract torrent download URL
            let torrent_url = Self::extract_href(row, "/download/");

            // Extract info_hash from magnet
            let info_hash = magnet.as_ref().and_then(|m| {
                m.find("btih:").map(|pos| {
                    let start = pos + 5;
                    let end = m[start..].find('&').unwrap_or(m[start..].len());
                    m[start..start + end].to_lowercase()
                })
            });

            // Parse seeders/leechers from <td> cells
            // Nyaa has: Category | Name | Links | Size | Date | Seeders | Leechers | Downloads
            let tds: Vec<&str> = row.split("<td").collect();
            let seeders = Self::extract_td_number(&tds, tds.len().saturating_sub(3));
            let leechers = Self::extract_td_number(&tds, tds.len().saturating_sub(2));

            // Parse size
            let size_str = Self::extract_td_text(&tds, tds.len().saturating_sub(5));
            let size_bytes = Self::parse_size(&size_str);

            // Parse date
            let date = Self::extract_td_text(&tds, tds.len().saturating_sub(4));

            // Build source URL
            let view_path = Self::extract_href(row, "/view/");

            results.push(PluginSearchResult {
                title,
                magnet,
                torrent_url: torrent_url.map(|p| format!("https://nyaa.si{}", p)),
                info_hash,
                size_bytes,
                seeders,
                leechers,
                upload_date: if date.is_empty() { None } else { Some(date) },
                source: "nyaa".to_string(),
                category: Some("anime".to_string()),
                source_url: view_path
                    .map(|p| format!("https://nyaa.si{}", p))
                    .unwrap_or_default(),
            });
        }

        results
    }

    fn extract_attr<'a>(html: &'a str, href_contains: &str, attr: &str) -> Option<String> {
        let href_pos = html.find(href_contains)?;
        // Look backwards for the attr
        let search_region = &html[..href_pos + href_contains.len() + 200.min(html.len() - href_pos - href_contains.len())];
        let attr_pos = search_region.find(attr)?;
        let start = attr_pos + attr.len();
        let end = search_region[start..].find('"')?;
        Some(search_region[start..start + end].to_string())
    }

    fn extract_href<'a>(html: &'a str, contains: &str) -> Option<String> {
        let pos = html.find(contains)?;
        // Go backwards to find href="
        let before = &html[..pos];
        let href_pos = before.rfind("href=\"")?;
        let start = href_pos + 6;
        let end = html[start..].find('"')?;
        Some(html[start..start + end].to_string())
    }

    fn extract_td_text(tds: &[&str], idx: usize) -> String {
        if idx >= tds.len() { return String::new(); }
        let td = tds[idx];
        if let Some(tag_end) = td.find('>') {
            let text = &td[tag_end + 1..];
            if let Some(close) = text.find("</td") {
                return text[..close].trim().to_string();
            }
        }
        String::new()
    }

    fn extract_td_number(tds: &[&str], idx: usize) -> u32 {
        let text = Self::extract_td_text(tds, idx);
        text.replace(',', "").parse().unwrap_or(0)
    }

    fn parse_size(s: &str) -> u64 {
        let parts: Vec<&str> = s.trim().split_whitespace().collect();
        if parts.len() < 2 { return 0; }
        let num: f64 = parts[0].parse().unwrap_or(0.0);
        match parts[1].to_uppercase().as_str() {
            "B" | "BYTES" => num as u64,
            "KIB" | "KB" => (num * 1024.0) as u64,
            "MIB" | "MB" => (num * 1024.0 * 1024.0) as u64,
            "GIB" | "GB" => (num * 1024.0 * 1024.0 * 1024.0) as u64,
            "TIB" | "TB" => (num * 1024.0 * 1024.0 * 1024.0 * 1024.0) as u64,
            _ => 0,
        }
    }
}

#[async_trait]
impl SearchPlugin for NyaaPlugin {
    fn id(&self) -> &str { "nyaa" }
    fn name(&self) -> &str { "Nyaa.si" }

    fn supported_categories(&self) -> Vec<SearchCategory> {
        vec![
            SearchCategory::All,
            SearchCategory::Anime,
            SearchCategory::Music,
            SearchCategory::Books,
            SearchCategory::Software,
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
        let cat_id = Self::category_id(&cat);

        let url = format!(
            "https://nyaa.si/?f=0&c={}&q={}&p={}&s=seeders&o=desc",
            cat_id,
            urlencoding::encode(query),
            page + 1
        );

        let resp = client
            .get(&url)
            .header("User-Agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36")
            .send()
            .await
            .map_err(|e| format!("Nyaa request failed: {e}"))?;

        if !resp.status().is_success() {
            return Err(format!("Nyaa returned HTTP {}", resp.status()));
        }

        let html = resp.text().await
            .map_err(|e| format!("Failed to read Nyaa response: {e}"))?;

        Ok(Self::parse_results(&html))
    }
}
