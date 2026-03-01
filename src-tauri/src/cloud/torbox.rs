// Tsubasa — Torbox Provider
// Full implementation of the DebridProvider trait for Torbox.
// API docs: https://api.torbox.app/docs

use async_trait::async_trait;
use parking_lot::RwLock;
use serde::Deserialize;

use super::provider::{
    AccountInfo, CloudStatus, CloudTorrentId, DebridProvider, DirectLink, TorrentSource,
};
use crate::error::{CloudError, TsubasaError};

// ─── Torbox API response shapes ─────────────────────────

/// All Torbox responses wrap data in this envelope.
#[derive(Debug, Deserialize)]
struct TorboxResponse<T> {
    success: bool,
    detail: Option<String>,
    data: Option<T>,
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CreateTorrentData {
    torrent_id: i64,
    #[allow(dead_code)]
    hash: Option<String>,
    #[allow(dead_code)]
    name: Option<String>,
}

/// Envelope for duplicate-torrent error responses where success=false but data contains
/// the existing torrent_id (Torbox returns this for already-queued magnets).
#[derive(Debug, Deserialize)]
struct CreateTorrentEnvelope {
    success: bool,
    error: Option<String>,
    detail: Option<String>,
    data: Option<CreateTorrentData>,
}

/// Torbox torrent info from the /torrents/mylist endpoint.
/// Serde ignores unknown fields by default, so we only list what we need.
#[derive(Debug, Deserialize)]
struct TorboxTorrentInfo {
    #[allow(dead_code)]
    id: i64,
    #[allow(dead_code)]
    hash: Option<String>,
    #[allow(dead_code)]
    name: Option<String>,
    download_state: Option<String>,
    progress: Option<f64>,
    #[allow(dead_code)]
    size: Option<u64>,
    #[serde(default)]
    files: Option<Vec<TorboxFile>>,
}

/// Individual file inside a Torbox torrent.
/// The API returns both `name` (full path) and `short_name` (filename only).
/// We keep them as separate fields — using `#[serde(alias)]` would cause
/// a "duplicate field" error since both keys are present in the response.
#[derive(Debug, Deserialize)]
struct TorboxFile {
    id: i64,
    name: Option<String>,
    short_name: Option<String>,
    size: Option<u64>,
}

impl TorboxFile {
    /// Returns the best display name: prefer short_name, fall back to name.
    fn display_name(&self) -> String {
        self.short_name
            .clone()
            .or_else(|| self.name.clone())
            .unwrap_or_else(|| format!("file_{}", self.id))
    }
}

#[derive(Debug, Deserialize)]
struct TorboxUserData {
    #[allow(dead_code)]
    id: Option<i64>,
    email: Option<String>,
    plan: Option<i32>,
    #[allow(dead_code)]
    total_downloaded: Option<u64>,
    premium_expires_at: Option<String>,
    #[allow(dead_code)]
    server: Option<i32>,
}

// ─── Provider implementation ─────────────────────────────

pub struct TorboxProvider {
    pub(crate) api_key: RwLock<Option<String>>,
    client: reqwest::Client,
    base_url: String,
}

impl TorboxProvider {
    pub fn new(api_key: Option<String>) -> Self {
        Self {
            api_key: RwLock::new(api_key),
            client: reqwest::Client::new(),
            base_url: "https://api.torbox.app/v1/api".to_string(),
        }
    }

    pub fn set_api_key(&self, key: String) {
        *self.api_key.write() = Some(key);
    }

    pub fn clear_api_key(&self) {
        *self.api_key.write() = None;
    }

    /// Clone the API key out of the lock (short-lived lock, safe across .await).
    fn get_api_key(&self) -> crate::error::Result<String> {
        self.api_key
            .read()
            .clone()
            .ok_or_else(|| TsubasaError::Cloud {
                provider: "torbox".to_string(),
                source: CloudError::InvalidApiKey,
            })
    }

    /// Build a GET request with auth header.
    fn authed_get(&self, url: &str, api_key: &str) -> reqwest::RequestBuilder {
        self.client
            .get(url)
            .header("Authorization", format!("Bearer {api_key}"))
    }

    /// Build a POST request with auth header.
    fn authed_post(&self, url: &str, api_key: &str) -> reqwest::RequestBuilder {
        self.client
            .post(url)
            .header("Authorization", format!("Bearer {api_key}"))
    }

    /// Parse a Torbox envelope response, mapping HTTP/API errors to CloudError.
    async fn parse_response<T: serde::de::DeserializeOwned>(
        resp: reqwest::Response,
    ) -> crate::error::Result<T> {
        let status = resp.status();

        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            let retry_after = resp
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(30);
            return Err(TsubasaError::Cloud {
                provider: "torbox".to_string(),
                source: CloudError::RateLimited {
                    retry_after_secs: retry_after,
                },
            });
        }

        if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
            return Err(TsubasaError::Cloud {
                provider: "torbox".to_string(),
                source: CloudError::AuthFailed(format!("HTTP {status}")),
            });
        }

        let body = resp.text().await.map_err(|e| TsubasaError::Cloud {
            provider: "torbox".to_string(),
            source: CloudError::ApiRequest(format!("Failed to read response body: {e}")),
        })?;

        let envelope: TorboxResponse<T> =
            serde_json::from_str(&body).map_err(|e| TsubasaError::Cloud {
                provider: "torbox".to_string(),
                source: CloudError::ApiRequest(format!(
                    "Failed to parse response: {e} — body: {body}"
                )),
            })?;

        if !envelope.success {
            let msg = envelope
                .error
                .or(envelope.detail)
                .unwrap_or_else(|| "Unknown API error".to_string());
            return Err(TsubasaError::Cloud {
                provider: "torbox".to_string(),
                source: CloudError::ApiRequest(msg),
            });
        }

        envelope.data.ok_or_else(|| TsubasaError::Cloud {
            provider: "torbox".to_string(),
            source: CloudError::ApiRequest("Response contained no data".to_string()),
        })
    }
}

#[async_trait]
impl DebridProvider for TorboxProvider {
    fn name(&self) -> &str {
        "Torbox"
    }

    fn is_configured(&self) -> bool {
        self.api_key.read().is_some()
    }

    async fn add_torrent(
        &self,
        source: &TorrentSource,
    ) -> crate::error::Result<CloudTorrentId> {
        let api_key = self.get_api_key()?;
        let url = format!("{}/torrents/createtorrent", self.base_url);

        let form = match source {
            TorrentSource::MagnetUri(magnet) => {
                reqwest::multipart::Form::new().text("magnet", magnet.clone())
            }
            TorrentSource::InfoHash(hash) => {
                let magnet = format!("magnet:?xt=urn:btih:{hash}");
                reqwest::multipart::Form::new().text("magnet", magnet)
            }
            TorrentSource::TorrentFile(bytes) => {
                let part = reqwest::multipart::Part::bytes(bytes.clone())
                    .file_name("torrent.torrent")
                    .mime_str("application/x-bittorrent")
                    .map_err(|e| TsubasaError::Cloud {
                        provider: "torbox".to_string(),
                        source: CloudError::ApiRequest(format!("Failed to create multipart: {e}")),
                    })?;
                reqwest::multipart::Form::new().part("file", part)
            }
        };

        let resp = self
            .authed_post(&url, &api_key)
            .multipart(form)
            .send()
            .await
            .map_err(|e| TsubasaError::Cloud {
                provider: "torbox".to_string(),
                source: CloudError::ApiRequest(format!("Request failed: {e}")),
            })?;

        // Handle HTTP-level errors first
        let http_status = resp.status();
        if http_status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(TsubasaError::Cloud {
                provider: "torbox".to_string(),
                source: CloudError::RateLimited { retry_after_secs: 30 },
            });
        }
        if http_status == reqwest::StatusCode::UNAUTHORIZED || http_status == reqwest::StatusCode::FORBIDDEN {
            return Err(TsubasaError::Cloud {
                provider: "torbox".to_string(),
                source: CloudError::AuthFailed(format!("HTTP {http_status}")),
            });
        }

        // Read the full body so we can handle both success + duplicate-torrent responses.
        let body = resp.text().await.map_err(|e| TsubasaError::Cloud {
            provider: "torbox".to_string(),
            source: CloudError::ApiRequest(format!("Failed to read response body: {e}")),
        })?;

        let envelope: CreateTorrentEnvelope =
            serde_json::from_str(&body).map_err(|e| TsubasaError::Cloud {
                provider: "torbox".to_string(),
                source: CloudError::ApiRequest(format!(
                    "Failed to parse createtorrent response: {e} — body: {body}"
                )),
            })?;

        if envelope.success {
            // Normal success path
            let data = envelope.data.ok_or_else(|| TsubasaError::Cloud {
                provider: "torbox".to_string(),
                source: CloudError::ApiRequest("createtorrent succeeded but data was empty".to_string()),
            })?;
            return Ok(CloudTorrentId(data.torrent_id.to_string()));
        }

        // success=false — check if it's a duplicate torrent (already queued/cached on Torbox).
        // Torbox returns: { success: false, error: "DUPLICATE_TORRENT", data: { torrent_id: N } }
        let error_code = envelope.error.as_deref().unwrap_or("");
        if error_code == "DUPLICATE_TORRENT" {
            if let Some(data) = envelope.data {
                tracing::info!(torrent_id = data.torrent_id, "Torbox: torrent already exists, reusing existing ID");
                return Ok(CloudTorrentId(data.torrent_id.to_string()));
            }
        }

        // Any other failure
        let msg = envelope
            .detail
            .unwrap_or_else(|| format!("Torbox API error: {error_code}"));
        Err(TsubasaError::Cloud {
            provider: "torbox".to_string(),
            source: CloudError::ApiRequest(msg),
        })
    }

    async fn check_status(
        &self,
        id: &CloudTorrentId,
    ) -> crate::error::Result<CloudStatus> {
        let api_key = self.get_api_key()?;
        let url = format!(
            "{}/torrents/mylist?bypass_cache=true&id={}",
            self.base_url, id.0
        );

        let resp = self
            .authed_get(&url, &api_key)
            .send()
            .await
            .map_err(|e| TsubasaError::Cloud {
                provider: "torbox".to_string(),
                source: CloudError::ApiRequest(format!("Request failed: {e}")),
            })?;

        let info: TorboxTorrentInfo = Self::parse_response(resp).await?;

        let status = match info.download_state.as_deref() {
            Some("cached") => CloudStatus::Cached,
            Some("completed" | "uploading") => CloudStatus::Completed,
            Some("downloading" | "metaDL" | "compressing") => CloudStatus::Downloading {
                progress: info.progress.unwrap_or(0.0),
            },
            Some("queued" | "stalled (no seeds)") => CloudStatus::Queued,
            Some("error") | Some("dead") => CloudStatus::Failed {
                reason: "Torrent failed on cloud provider".to_string(),
            },
            Some("paused") => CloudStatus::Queued,
            Some(other) => {
                tracing::warn!(state = other, "Unknown Torbox download_state");
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

        // First get the torrent info to list files
        let info_url = format!(
            "{}/torrents/mylist?bypass_cache=true&id={}",
            self.base_url, id.0
        );
        let info_resp = self
            .authed_get(&info_url, &api_key)
            .send()
            .await
            .map_err(|e| TsubasaError::Cloud {
                provider: "torbox".to_string(),
                source: CloudError::ApiRequest(format!("Request failed: {e}")),
            })?;

        let info: TorboxTorrentInfo = Self::parse_response(info_resp).await?;
        let files = info.files.unwrap_or_default();

        if files.is_empty() {
            return Err(TsubasaError::Cloud {
                provider: "torbox".to_string(),
                source: CloudError::DownloadFailed("No files in torrent".to_string()),
            });
        }

        // Request a download link for each file
        let mut links = Vec::with_capacity(files.len());
        for file in &files {
            let dl_url = format!(
                "{}/torrents/requestdl?token={}&torrent_id={}&file_id={}&zip_link=false",
                self.base_url, api_key, id.0, file.id
            );

            let dl_resp = self
                .authed_get(&dl_url, &api_key)
                .send()
                .await
                .map_err(|e| TsubasaError::Cloud {
                    provider: "torbox".to_string(),
                    source: CloudError::ApiRequest(format!("Request failed: {e}")),
                })?;

            // requestdl returns data as a string URL
            let download_url: String = Self::parse_response(dl_resp).await?;

            links.push(DirectLink {
                filename: file.display_name(),
                url: download_url,
                size_bytes: file.size.unwrap_or(0),
            });
        }

        Ok(links)
    }

    async fn check_cached(
        &self,
        info_hash: &str,
    ) -> crate::error::Result<bool> {
        let api_key = self.get_api_key()?;
        let url = format!(
            "{}/torrents/checkcached?hash={}&format=list&list_files=false",
            self.base_url, info_hash
        );

        let resp = self
            .authed_get(&url, &api_key)
            .send()
            .await
            .map_err(|e| TsubasaError::Cloud {
                provider: "torbox".to_string(),
                source: CloudError::ApiRequest(format!("Request failed: {e}")),
            })?;

        let status = resp.status();
        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(TsubasaError::Cloud {
                provider: "torbox".to_string(),
                source: CloudError::RateLimited {
                    retry_after_secs: 30,
                },
            });
        }

        let body = resp.text().await.map_err(|e| TsubasaError::Cloud {
            provider: "torbox".to_string(),
            source: CloudError::ApiRequest(format!("Failed to read response: {e}")),
        })?;

        // The response is { success: true, data: [...] }
        // If data is a non-empty array, the torrent is cached.
        let envelope: TorboxResponse<serde_json::Value> =
            serde_json::from_str(&body).map_err(|e| TsubasaError::Cloud {
                provider: "torbox".to_string(),
                source: CloudError::ApiRequest(format!("Parse error: {e}")),
            })?;

        if !envelope.success {
            return Ok(false);
        }

        match envelope.data {
            Some(serde_json::Value::Array(arr)) => Ok(!arr.is_empty()),
            Some(serde_json::Value::Null) => Ok(false),
            _ => Ok(false),
        }
    }

    async fn account_info(&self) -> crate::error::Result<AccountInfo> {
        let api_key = self.get_api_key()?;
        let url = format!("{}/user/me", self.base_url);

        let resp = self
            .authed_get(&url, &api_key)
            .send()
            .await
            .map_err(|e| TsubasaError::Cloud {
                provider: "torbox".to_string(),
                source: CloudError::ApiRequest(format!("Request failed: {e}")),
            })?;

        let user: TorboxUserData = Self::parse_response(resp).await?;

        let plan_name = match user.plan {
            Some(0) => "Free",
            Some(1) => "Essential",
            Some(2) => "Pro",
            Some(3) => "Standard",
            _ => "Unknown",
        };

        Ok(AccountInfo {
            provider: "Torbox".to_string(),
            username: user.email.unwrap_or_else(|| "Unknown".to_string()),
            plan: plan_name.to_string(),
            expiry: user.premium_expires_at,
            storage_used: 0,
            storage_total: 0,
            points_used: None,
            points_total: None,
        })
    }

    async fn delete_torrent(
        &self,
        id: &CloudTorrentId,
    ) -> crate::error::Result<()> {
        let api_key = self.get_api_key()?;
        let url = format!("{}/torrents/controltorrent", self.base_url);

        let resp = self
            .authed_post(&url, &api_key)
            .json(&serde_json::json!({
                "torrent_id": id.0.parse::<i64>().unwrap_or(0),
                "operation": "delete"
            }))
            .send()
            .await
            .map_err(|e| TsubasaError::Cloud {
                provider: "torbox".to_string(),
                source: CloudError::ApiRequest(format!("Request failed: {e}")),
            })?;

        let status = resp.status();
        if status.is_success() {
            Ok(())
        } else {
            let body = resp.text().await.unwrap_or_default();
            Err(TsubasaError::Cloud {
                provider: "torbox".to_string(),
                source: CloudError::ApiRequest(format!("Delete failed (HTTP {status}): {body}")),
            })
        }
    }
}
