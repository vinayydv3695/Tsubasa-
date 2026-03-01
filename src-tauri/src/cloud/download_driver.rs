// Tsubasa — Cloud Download Driver
// HTTP file download from cloud provider CDN links with progress reporting.
// Supports cancellation, progress events, streaming writes, retry with backoff,
// and HTTP Range resume for partial downloads.

use std::path::Path;

use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;
use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;

use crate::error::{DownloadError, TsubasaError};
use crate::events::TsubasaEvent;
use crate::retry::{retry_with_backoff, RetryConfig};

/// Progress of an HTTP download from cloud.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudDownloadProgress {
    pub torrent_id: String,
    pub filename: String,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub speed_bytes_sec: f64,
}

/// Result of a completed cloud file download.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudDownloadResult {
    pub filename: String,
    pub save_path: String,
    pub total_bytes: u64,
}

/// Start an HTTP GET request, optionally resuming from `existing_bytes` via Range header.
/// This is the part we wrap with retry — just the connection + initial response,
/// not the streaming body.
async fn start_http_get(
    client: &reqwest::Client,
    url: &str,
    existing_bytes: u64,
) -> crate::error::Result<(reqwest::Response, u64)> {
    let mut req = client.get(url);

    if existing_bytes > 0 {
        req = req.header("Range", format!("bytes={existing_bytes}-"));
    }

    let resp = req.send().await.map_err(|e| {
        TsubasaError::Download(DownloadError::HttpFetch(format!("Request failed: {e}")))
    })?;

    let status = resp.status();

    // 416 Range Not Satisfiable means the file is already complete
    if status == reqwest::StatusCode::RANGE_NOT_SATISFIABLE {
        return Err(TsubasaError::Download(DownloadError::HttpFetch(
            "Range not satisfiable (file may be complete)".to_string(),
        )));
    }

    if !status.is_success() {
        return Err(TsubasaError::Download(DownloadError::HttpFetch(format!(
            "HTTP {status} for {url}"
        ))));
    }

    // Determine total file size
    let total_bytes = if status == reqwest::StatusCode::PARTIAL_CONTENT {
        // Server supports Range — total from Content-Range header or fallback
        resp.headers()
            .get("content-range")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.split('/').last())
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(existing_bytes + resp.content_length().unwrap_or(0))
    } else {
        // No range support — total is content-length (starting from 0)
        resp.content_length().unwrap_or(0)
    };

    Ok((resp, total_bytes))
}

/// Downloads a file from a direct cloud CDN link to disk.
///
/// - Retries the initial HTTP connection with exponential backoff.
/// - Resumes partial downloads via HTTP Range headers.
/// - Streams the response body to avoid holding the entire file in memory.
/// - Reports progress via the event bus every ~500ms.
/// - Supports cancellation via the `cancel` token.
pub async fn download_cloud_file(
    client: &reqwest::Client,
    url: &str,
    filename: &str,
    save_dir: &Path,
    torrent_id: &str,
    provider: &str,
    event_tx: &broadcast::Sender<TsubasaEvent>,
    cancel: &CancellationToken,
) -> crate::error::Result<CloudDownloadResult> {
    // Create save directory if it doesn't exist
    tokio::fs::create_dir_all(save_dir)
        .await
        .map_err(|e| {
            TsubasaError::Download(DownloadError::FileWrite(format!(
                "Failed to create directory {}: {e}",
                save_dir.display()
            )))
        })?;

    let save_path = save_dir.join(filename);

    // Check for existing partial file (for resume)
    let existing_bytes = if save_path.exists() {
        tokio::fs::metadata(&save_path)
            .await
            .map(|m| m.len())
            .unwrap_or(0)
    } else {
        0
    };

    // Start HTTP request with retry (retries connection, not the streaming body)
    let retry_config = RetryConfig::http_download();
    let client_ref = client;
    let url_str = url;
    let resume_from = existing_bytes;

    let (resp, total_bytes) = retry_with_backoff(
        &retry_config,
        &format!("cloud_download_connect({filename})"),
        || async { start_http_get(client_ref, url_str, resume_from).await },
    )
    .await?;

    let is_resume = resp.status() == reqwest::StatusCode::PARTIAL_CONTENT && existing_bytes > 0;

    // Open output file — append if resuming, create if starting fresh
    let mut file = if is_resume {
        tokio::fs::OpenOptions::new()
            .append(true)
            .open(&save_path)
            .await
            .map_err(|e| {
                TsubasaError::Download(DownloadError::FileWrite(format!(
                    "Failed to open for append {}: {e}",
                    save_path.display()
                )))
            })?
    } else {
        tokio::fs::File::create(&save_path)
            .await
            .map_err(|e| {
                TsubasaError::Download(DownloadError::FileWrite(format!(
                    "Failed to create {}: {e}",
                    save_path.display()
                )))
            })?
    };

    if is_resume {
        tracing::info!(
            filename = filename,
            existing_bytes = existing_bytes,
            total_bytes = total_bytes,
            "Resuming cloud download"
        );
    }

    let mut stream = resp.bytes_stream();
    let mut downloaded: u64 = if is_resume { existing_bytes } else { 0 };
    let mut last_report = std::time::Instant::now();
    let mut last_report_bytes: u64 = downloaded;
    let report_interval = std::time::Duration::from_millis(500);

    loop {
        tokio::select! {
            _ = cancel.cancelled() => {
                // Keep partial file for potential resume — don't delete
                return Err(TsubasaError::Download(DownloadError::Cancelled));
            }
            chunk = stream.next() => {
                match chunk {
                    Some(Ok(bytes)) => {
                        file.write_all(&bytes).await.map_err(|e| {
                            TsubasaError::Download(
                                DownloadError::FileWrite(format!("Write failed: {e}")),
                            )
                        })?;

                        downloaded += bytes.len() as u64;

                        // Report progress periodically
                        let now = std::time::Instant::now();
                        if now.duration_since(last_report) >= report_interval {
                            let elapsed = now.duration_since(last_report).as_secs_f64();
                            let bytes_since = downloaded - last_report_bytes;
                            let _speed = if elapsed > 0.0 {
                                bytes_since as f64 / elapsed
                            } else {
                                0.0
                            };

                            let progress_pct = if total_bytes > 0 {
                                (downloaded as f64 / total_bytes as f64) * 100.0
                            } else {
                                0.0
                            };

                            let _ = event_tx.send(TsubasaEvent::CloudDownloadProgress {
                                torrent_id: torrent_id.to_string(),
                                provider: provider.to_string(),
                                progress_pct,
                            });

                            last_report = now;
                            last_report_bytes = downloaded;
                        }
                    }
                    Some(Err(e)) => {
                        // Keep partial file for resume on stream errors
                        return Err(TsubasaError::Download(
                            DownloadError::HttpFetch(format!("Stream error: {e}")),
                        ));
                    }
                    None => {
                        // Stream finished
                        break;
                    }
                }
            }
        }
    }

    file.flush().await.map_err(|e| {
        TsubasaError::Download(DownloadError::FileWrite(format!("Flush failed: {e}")))
    })?;

    // Final progress report (100%)
    let _ = event_tx.send(TsubasaEvent::CloudDownloadProgress {
        torrent_id: torrent_id.to_string(),
        provider: provider.to_string(),
        progress_pct: 100.0,
    });

    tracing::info!(
        filename = filename,
        bytes = downloaded,
        resumed = is_resume,
        "Cloud file download complete"
    );

    Ok(CloudDownloadResult {
        filename: filename.to_string(),
        save_path: save_path.to_string_lossy().to_string(),
        total_bytes: downloaded,
    })
}

/// Downloads all files from cloud direct links into the given directory.
/// Returns the list of successfully downloaded files.
pub async fn download_all_cloud_files(
    client: &reqwest::Client,
    links: &[super::provider::DirectLink],
    save_dir: &Path,
    torrent_id: &str,
    provider: &str,
    event_tx: &broadcast::Sender<TsubasaEvent>,
    cancel: &CancellationToken,
) -> crate::error::Result<Vec<CloudDownloadResult>> {
    let mut results = Vec::with_capacity(links.len());

    for link in links {
        if cancel.is_cancelled() {
            return Err(TsubasaError::Download(DownloadError::Cancelled));
        }

        let result = download_cloud_file(
            client,
            &link.url,
            &link.filename,
            save_dir,
            torrent_id,
            provider,
            event_tx,
            cancel,
        )
        .await?;

        results.push(result);
    }

    Ok(results)
}
