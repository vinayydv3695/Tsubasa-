// Tsubasa — Real-Debrid Provider
// Full implementation of the DebridProvider trait for Real-Debrid.
// API docs: https://api.real-debrid.com

use async_trait::async_trait;
use parking_lot::RwLock;
use serde::Deserialize;
use std::collections::HashMap;

use super::provider::{
    AccountInfo, CloudStatus, CloudTorrentId, DebridProvider, DirectLink, TorrentSource,
};
use crate::error::{CloudError, TsubasaError};

// ─── Real-Debrid API response shapes ────────────────────

#[derive(Debug, Deserialize)]
struct RdAddMagnetResponse {
    id: String,
    #[allow(dead_code)]
    uri: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RdAddTorrentResponse {
    id: String,
    #[allow(dead_code)]
    uri: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RdTorrentInfo {
    id: String,
    filename: Option<String>,
    #[allow(dead_code)]
    hash: Option<String>,
    bytes: Option<u64>,
    status: Option<String>,
    progress: Option<f64>,
    links: Option<Vec<String>>,
    #[allow(dead_code)]
    files: Option<Vec<RdFile>>,
}

#[derive(Debug, Deserialize)]
struct RdFile {
    #[allow(dead_code)]
    id: i64,
    path: Option<String>,
    bytes: Option<u64>,
    #[allow(dead_code)]
    selected: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct RdUnrestrictResponse {
    #[allow(dead_code)]
    id: Option<String>,
    filename: Option<String>,
    download: Option<String>,
    filesize: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct RdUser {
    #[allow(dead_code)]
    id: Option<i64>,
    username: Option<String>,
    #[allow(dead_code)]
    email: Option<String>,
    #[serde(rename = "type")]
    account_type: Option<String>,
    expiration: Option<String>,
    points: Option<i64>,
}

/// Error response from Real-Debrid.
#[derive(Debug, Deserialize)]
struct RdError {
    error: Option<String>,
    #[allow(dead_code)]
    error_code: Option<i32>,
}

// ─── Provider implementation ─────────────────────────────

pub struct RealDebridProvider {
    api_key: RwLock<Option<String>>,
    client: reqwest::Client,
    base_url: String,
}

impl RealDebridProvider {
    pub fn new(api_key: Option<String>) -> Self {
        Self {
            api_key: RwLock::new(api_key),
            client: reqwest::Client::new(),
            base_url: "https://api.real-debrid.com/rest/1.0".to_string(),
        }
    }

    pub fn set_api_key(&self, key: String) {
        *self.api_key.write() = Some(key);
    }

    pub fn clear_api_key(&self) {
        *self.api_key.write() = None;
    }

    fn get_api_key(&self) -> crate::error::Result<String> {
        self.api_key
            .read()
            .clone()
            .ok_or_else(|| TsubasaError::Cloud {
                provider: "realdebrid".to_string(),
                source: CloudError::InvalidApiKey,
            })
    }

    fn authed_get(&self, url: &str, api_key: &str) -> reqwest::RequestBuilder {
        self.client
            .get(url)
            .header("Authorization", format!("Bearer {api_key}"))
    }

    fn authed_post(&self, url: &str, api_key: &str) -> reqwest::RequestBuilder {
        self.client
            .post(url)
            .header("Authorization", format!("Bearer {api_key}"))
    }

    fn authed_put(&self, url: &str, api_key: &str) -> reqwest::RequestBuilder {
        self.client
            .put(url)
            .header("Authorization", format!("Bearer {api_key}"))
    }

    fn authed_delete(&self, url: &str, api_key: &str) -> reqwest::RequestBuilder {
        self.client
            .delete(url)
            .header("Authorization", format!("Bearer {api_key}"))
    }

    /// Check for HTTP-level errors common to all RD endpoints.
    async fn check_error(resp: &reqwest::Response) -> Option<TsubasaError> {
        let status = resp.status();

        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Some(TsubasaError::Cloud {
                provider: "realdebrid".to_string(),
                source: CloudError::RateLimited {
                    retry_after_secs: 60,
                },
            });
        }

        if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
            return Some(TsubasaError::Cloud {
                provider: "realdebrid".to_string(),
                source: CloudError::AuthFailed(format!("HTTP {status}")),
            });
        }

        None
    }

    /// Parse error body from a failed RD response.
    async fn parse_error_body(resp: reqwest::Response) -> TsubasaError {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        let msg = serde_json::from_str::<RdError>(&body)
            .ok()
            .and_then(|e| e.error)
            .unwrap_or_else(|| format!("HTTP {status}: {body}"));
        TsubasaError::Cloud {
            provider: "realdebrid".to_string(),
            source: CloudError::ApiRequest(msg),
        }
    }
}

#[async_trait]
impl DebridProvider for RealDebridProvider {
    fn name(&self) -> &str {
        "Real-Debrid"
    }

    fn is_configured(&self) -> bool {
        self.api_key.read().is_some()
    }

    async fn add_torrent(
        &self,
        source: &TorrentSource,
    ) -> crate::error::Result<CloudTorrentId> {
        let api_key = self.get_api_key()?;

        let torrent_id = match source {
            TorrentSource::MagnetUri(magnet) => {
                let url = format!("{}/torrents/addMagnet", self.base_url);
                let resp = self
                    .authed_post(&url, &api_key)
                    .form(&[("magnet", magnet.as_str())])
                    .send()
                    .await
                    .map_err(|e| TsubasaError::Cloud {
                        provider: "realdebrid".to_string(),
                        source: CloudError::ApiRequest(format!("Request failed: {e}")),
                    })?;

                if let Some(err) = Self::check_error(&resp).await {
                    return Err(err);
                }
                if !resp.status().is_success() {
                    return Err(Self::parse_error_body(resp).await);
                }

                let data: RdAddMagnetResponse = resp.json().await.map_err(|e| {
                    TsubasaError::Cloud {
                        provider: "realdebrid".to_string(),
                        source: CloudError::ApiRequest(format!("Parse error: {e}")),
                    }
                })?;
                data.id
            }
            TorrentSource::InfoHash(hash) => {
                let magnet = format!("magnet:?xt=urn:btih:{hash}");
                let url = format!("{}/torrents/addMagnet", self.base_url);
                let resp = self
                    .authed_post(&url, &api_key)
                    .form(&[("magnet", magnet.as_str())])
                    .send()
                    .await
                    .map_err(|e| TsubasaError::Cloud {
                        provider: "realdebrid".to_string(),
                        source: CloudError::ApiRequest(format!("Request failed: {e}")),
                    })?;

                if let Some(err) = Self::check_error(&resp).await {
                    return Err(err);
                }
                if !resp.status().is_success() {
                    return Err(Self::parse_error_body(resp).await);
                }

                let data: RdAddMagnetResponse = resp.json().await.map_err(|e| {
                    TsubasaError::Cloud {
                        provider: "realdebrid".to_string(),
                        source: CloudError::ApiRequest(format!("Parse error: {e}")),
                    }
                })?;
                data.id
            }
            TorrentSource::TorrentFile(bytes) => {
                let url = format!("{}/torrents/addTorrent", self.base_url);
                let resp = self
                    .authed_put(&url, &api_key)
                    .header("Content-Type", "application/x-bittorrent")
                    .body(bytes.clone())
                    .send()
                    .await
                    .map_err(|e| TsubasaError::Cloud {
                        provider: "realdebrid".to_string(),
                        source: CloudError::ApiRequest(format!("Request failed: {e}")),
                    })?;

                if let Some(err) = Self::check_error(&resp).await {
                    return Err(err);
                }
                if !resp.status().is_success() {
                    return Err(Self::parse_error_body(resp).await);
                }

                let data: RdAddTorrentResponse = resp.json().await.map_err(|e| {
                    TsubasaError::Cloud {
                        provider: "realdebrid".to_string(),
                        source: CloudError::ApiRequest(format!("Parse error: {e}")),
                    }
                })?;
                data.id
            }
        };

        // Real-Debrid requires selecting files after adding a torrent.
        // We select all files by default.
        let select_url = format!("{}/torrents/selectFiles/{}", self.base_url, torrent_id);
        let select_resp = self
            .authed_post(&select_url, &api_key)
            .form(&[("files", "all")])
            .send()
            .await
            .map_err(|e| TsubasaError::Cloud {
                provider: "realdebrid".to_string(),
                source: CloudError::ApiRequest(format!("Select files failed: {e}")),
            })?;

        // selectFiles returns 204 on success, or error
        if !select_resp.status().is_success() {
            tracing::warn!(
                status = %select_resp.status(),
                "Real-Debrid selectFiles returned non-success (torrent may still work)"
            );
        }

        Ok(CloudTorrentId(torrent_id))
    }

    async fn check_status(
        &self,
        id: &CloudTorrentId,
    ) -> crate::error::Result<CloudStatus> {
        let api_key = self.get_api_key()?;
        let url = format!("{}/torrents/info/{}", self.base_url, id.0);

        let resp = self
            .authed_get(&url, &api_key)
            .send()
            .await
            .map_err(|e| TsubasaError::Cloud {
                provider: "realdebrid".to_string(),
                source: CloudError::ApiRequest(format!("Request failed: {e}")),
            })?;

        if let Some(err) = Self::check_error(&resp).await {
            return Err(err);
        }
        if !resp.status().is_success() {
            return Err(Self::parse_error_body(resp).await);
        }

        let info: RdTorrentInfo = resp.json().await.map_err(|e| TsubasaError::Cloud {
            provider: "realdebrid".to_string(),
            source: CloudError::ApiRequest(format!("Parse error: {e}")),
        })?;

        // Real-Debrid statuses: magnet_error, magnet_conversion, waiting_files_selection,
        // queued, downloading, downloaded, error, virus, compressing, uploading, dead
        let status = match info.status.as_deref() {
            Some("downloaded") => CloudStatus::Completed,
            Some("downloading" | "compressing" | "uploading") => CloudStatus::Downloading {
                progress: info.progress.unwrap_or(0.0) / 100.0, // RD uses 0-100
            },
            Some("queued" | "magnet_conversion" | "waiting_files_selection") => CloudStatus::Queued,
            Some("error" | "virus" | "dead" | "magnet_error") => CloudStatus::Failed {
                reason: format!(
                    "Real-Debrid status: {}",
                    info.status.as_deref().unwrap_or("unknown")
                ),
            },
            Some(other) => {
                tracing::warn!(status = other, "Unknown Real-Debrid torrent status");
                CloudStatus::Unknown
            }
            None => CloudStatus::Unknown,
        };

        Ok(status)
    }

    async fn get_download_links(
        &self,
        id: &CloudTorrentId,
    ) -> crate::error::Result<Vec<DirectLink>> {
        let api_key = self.get_api_key()?;

        // Get torrent info first to retrieve the hoster links
        let info_url = format!("{}/torrents/info/{}", self.base_url, id.0);
        let info_resp = self
            .authed_get(&info_url, &api_key)
            .send()
            .await
            .map_err(|e| TsubasaError::Cloud {
                provider: "realdebrid".to_string(),
                source: CloudError::ApiRequest(format!("Request failed: {e}")),
            })?;

        if let Some(err) = Self::check_error(&info_resp).await {
            return Err(err);
        }
        if !info_resp.status().is_success() {
            return Err(Self::parse_error_body(info_resp).await);
        }

        let info: RdTorrentInfo = info_resp.json().await.map_err(|e| TsubasaError::Cloud {
            provider: "realdebrid".to_string(),
            source: CloudError::ApiRequest(format!("Parse error: {e}")),
        })?;

        let hoster_links = info.links.unwrap_or_default();
        if hoster_links.is_empty() {
            return Err(TsubasaError::Cloud {
                provider: "realdebrid".to_string(),
                source: CloudError::DownloadFailed("No links available".to_string()),
            });
        }

        // Unrestrict each link to get direct CDN URLs
        let mut links = Vec::with_capacity(hoster_links.len());
        for link in &hoster_links {
            let unrestrict_url = format!("{}/unrestrict/link", self.base_url);
            let resp = self
                .authed_post(&unrestrict_url, &api_key)
                .form(&[("link", link.as_str())])
                .send()
                .await
                .map_err(|e| TsubasaError::Cloud {
                    provider: "realdebrid".to_string(),
                    source: CloudError::ApiRequest(format!("Unrestrict failed: {e}")),
                })?;

            if let Some(err) = Self::check_error(&resp).await {
                return Err(err);
            }
            if !resp.status().is_success() {
                tracing::warn!(link = %link, "Failed to unrestrict link, skipping");
                continue;
            }

            let unrestricted: RdUnrestrictResponse =
                resp.json().await.map_err(|e| TsubasaError::Cloud {
                    provider: "realdebrid".to_string(),
                    source: CloudError::ApiRequest(format!("Parse error: {e}")),
                })?;

            if let Some(download_url) = unrestricted.download {
                links.push(DirectLink {
                    filename: unrestricted
                        .filename
                        .unwrap_or_else(|| "unknown".to_string()),
                    url: download_url,
                    size_bytes: unrestricted.filesize.unwrap_or(0),
                });
            }
        }

        Ok(links)
    }

    async fn check_cached(
        &self,
        info_hash: &str,
    ) -> crate::error::Result<bool> {
        let api_key = self.get_api_key()?;
        let url = format!(
            "{}/torrents/instantAvailability/{}",
            self.base_url,
            info_hash.to_uppercase()
        );

        let resp = self
            .authed_get(&url, &api_key)
            .send()
            .await
            .map_err(|e| TsubasaError::Cloud {
                provider: "realdebrid".to_string(),
                source: CloudError::ApiRequest(format!("Request failed: {e}")),
            })?;

        if let Some(err) = Self::check_error(&resp).await {
            return Err(err);
        }
        if !resp.status().is_success() {
            return Ok(false);
        }

        // Response is { "HASH": { "rd": [...] } } — non-empty rd array means cached
        let body: HashMap<String, serde_json::Value> =
            resp.json().await.map_err(|e| TsubasaError::Cloud {
                provider: "realdebrid".to_string(),
                source: CloudError::ApiRequest(format!("Parse error: {e}")),
            })?;

        // Check if any hash key contains a non-empty "rd" array
        for (_hash, value) in &body {
            if let Some(obj) = value.as_object() {
                if let Some(rd) = obj.get("rd") {
                    if let Some(arr) = rd.as_array() {
                        if !arr.is_empty() {
                            return Ok(true);
                        }
                    }
                }
            }
        }

        Ok(false)
    }

    async fn account_info(&self) -> crate::error::Result<AccountInfo> {
        let api_key = self.get_api_key()?;
        let url = format!("{}/user", self.base_url);

        let resp = self
            .authed_get(&url, &api_key)
            .send()
            .await
            .map_err(|e| TsubasaError::Cloud {
                provider: "realdebrid".to_string(),
                source: CloudError::ApiRequest(format!("Request failed: {e}")),
            })?;

        if let Some(err) = Self::check_error(&resp).await {
            return Err(err);
        }
        if !resp.status().is_success() {
            return Err(Self::parse_error_body(resp).await);
        }

        let user: RdUser = resp.json().await.map_err(|e| TsubasaError::Cloud {
            provider: "realdebrid".to_string(),
            source: CloudError::ApiRequest(format!("Parse error: {e}")),
        })?;

        Ok(AccountInfo {
            provider: "Real-Debrid".to_string(),
            username: user.username.unwrap_or_else(|| "Unknown".to_string()),
            plan: user
                .account_type
                .unwrap_or_else(|| "unknown".to_string()),
            expiry: user.expiration,
            storage_used: 0,
            storage_total: 0,
            points_used: user.points.map(|p| p as u64),
            points_total: None,
        })
    }

    async fn delete_torrent(
        &self,
        id: &CloudTorrentId,
    ) -> crate::error::Result<()> {
        let api_key = self.get_api_key()?;
        let url = format!("{}/torrents/delete/{}", self.base_url, id.0);

        let resp = self
            .authed_delete(&url, &api_key)
            .send()
            .await
            .map_err(|e| TsubasaError::Cloud {
                provider: "realdebrid".to_string(),
                source: CloudError::ApiRequest(format!("Request failed: {e}")),
            })?;

        // delete returns 204 on success
        if resp.status().is_success() || resp.status() == reqwest::StatusCode::NO_CONTENT {
            Ok(())
        } else {
            Err(Self::parse_error_body(resp).await)
        }
    }
}
