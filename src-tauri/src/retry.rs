// Tsubasa (翼) — Retry Logic
// Generic async retry with exponential backoff, jitter, and rate-limit awareness.
// Classifies errors as transient (retryable) vs permanent (not retryable).

use std::future::Future;
use std::time::Duration;

use crate::error::{CloudError, DownloadError, TsubasaError};

/// Configuration for retry behavior.
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of attempts (including the first try).
    pub max_attempts: u32,
    /// Base delay between retries in milliseconds.
    pub base_delay_ms: u64,
    /// Maximum delay cap in milliseconds.
    pub max_delay_ms: u64,
    /// Whether to add random jitter to the delay.
    pub jitter: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay_ms: 1000,
            max_delay_ms: 30_000,
            jitter: true,
        }
    }
}

impl RetryConfig {
    /// Config tuned for cloud API calls (rate limits, transient failures).
    pub fn cloud_api() -> Self {
        Self {
            max_attempts: 4,
            base_delay_ms: 2000,
            max_delay_ms: 60_000,
            jitter: true,
        }
    }

    /// Config tuned for HTTP file downloads (connection errors).
    pub fn http_download() -> Self {
        Self {
            max_attempts: 3,
            base_delay_ms: 3000,
            max_delay_ms: 30_000,
            jitter: true,
        }
    }
}

/// Classify whether a `TsubasaError` is transient (worth retrying).
///
/// Returns `Some(override_delay)` if the error specifies a retry-after delay,
/// or `None` if the default backoff should be used.
/// Returns `Err(())` if the error is permanent and should not be retried.
fn classify_error(err: &TsubasaError) -> Result<Option<Duration>, ()> {
    match err {
        // Rate limited with explicit retry-after — transient, use provider's delay
        TsubasaError::Cloud {
            source: CloudError::RateLimited { retry_after_secs },
            ..
        } => Ok(Some(Duration::from_secs(*retry_after_secs))),

        // Provider temporarily unavailable — transient
        TsubasaError::Cloud {
            source: CloudError::Unavailable(_),
            ..
        } => Ok(None),

        // API request errors may be transient (connection failures, 5xx, timeouts)
        TsubasaError::Cloud {
            source: CloudError::ApiRequest(msg),
            ..
        } => {
            if is_transient_api_message(msg) {
                Ok(None)
            } else {
                Err(())
            }
        }

        // Auth failures — permanent, never retry
        TsubasaError::Cloud {
            source: CloudError::AuthFailed(_),
            ..
        }
        | TsubasaError::Cloud {
            source: CloudError::InvalidApiKey,
            ..
        } => Err(()),

        // Quota exceeded — permanent
        TsubasaError::Cloud {
            source: CloudError::QuotaExceeded,
            ..
        } => Err(()),

        // Not cached — permanent
        TsubasaError::Cloud {
            source: CloudError::NotCached,
            ..
        } => Err(()),

        // Cloud download failure (provider-side) — permanent
        TsubasaError::Cloud {
            source: CloudError::DownloadFailed(_),
            ..
        } => Err(()),

        // HTTP fetch errors in download driver — may be transient
        TsubasaError::Download(DownloadError::HttpFetch(msg)) => {
            if is_transient_api_message(msg) {
                Ok(None)
            } else {
                Err(())
            }
        }

        // Cancelled — definitely not retryable
        TsubasaError::Download(DownloadError::Cancelled) => Err(()),

        // Timeouts — transient
        TsubasaError::Timeout { .. } => Ok(None),

        // IO errors — may be transient (e.g., temporary disk full)
        TsubasaError::Io(_) => Ok(None),

        // Everything else — permanent
        _ => Err(()),
    }
}

/// Heuristic: check if an API error message suggests a transient failure.
fn is_transient_api_message(msg: &str) -> bool {
    let lower = msg.to_lowercase();
    lower.contains("connection")
        || lower.contains("timeout")
        || lower.contains("timed out")
        || lower.contains("reset by peer")
        || lower.contains("broken pipe")
        || lower.contains("temporarily unavailable")
        || lower.contains("503")
        || lower.contains("502")
        || lower.contains("500")
        || lower.contains("429")
        || lower.contains("service unavailable")
        || lower.contains("bad gateway")
        || lower.contains("internal server error")
        || lower.contains("database_error")
        || lower.contains("eof")
        || lower.contains("dns")
}

/// Execute an async operation with retry and exponential backoff.
///
/// - On transient errors, retries up to `config.max_attempts` times.
/// - On permanent errors, returns immediately.
/// - Respects rate-limit `retry_after` hints from the error.
/// - Uses exponential backoff: base_delay * 2^attempt, capped at max_delay.
/// - Optionally adds random jitter (0-50% of the computed delay).
pub async fn retry_with_backoff<F, Fut, T>(
    config: &RetryConfig,
    operation_name: &str,
    mut f: F,
) -> crate::error::Result<T>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = crate::error::Result<T>>,
{
    let mut last_error: Option<TsubasaError> = None;

    for attempt in 0..config.max_attempts {
        match f().await {
            Ok(value) => return Ok(value),
            Err(err) => {
                // Classify the error
                match classify_error(&err) {
                    Ok(override_delay) => {
                        // Transient — maybe retry
                        let remaining = config.max_attempts - attempt - 1;
                        if remaining == 0 {
                            tracing::warn!(
                                operation = %operation_name,
                                attempt = attempt + 1,
                                max_attempts = config.max_attempts,
                                "All retry attempts exhausted: {err}"
                            );
                            return Err(err);
                        }

                        // Compute delay
                        let delay = if let Some(forced) = override_delay {
                            // Rate-limit hint — use it directly
                            forced
                        } else {
                            // Exponential backoff: base * 2^attempt
                            let exp_delay_ms =
                                config.base_delay_ms.saturating_mul(1u64 << attempt);
                            let capped_ms = exp_delay_ms.min(config.max_delay_ms);

                            let jittered_ms = if config.jitter {
                                // Add 0-50% jitter
                                let jitter = (capped_ms as f64 * 0.5 * fastrand_f64()) as u64;
                                capped_ms.saturating_add(jitter)
                            } else {
                                capped_ms
                            };

                            Duration::from_millis(jittered_ms)
                        };

                        tracing::warn!(
                            operation = %operation_name,
                            attempt = attempt + 1,
                            max_attempts = config.max_attempts,
                            delay_ms = delay.as_millis() as u64,
                            "Transient error, retrying: {err}"
                        );

                        tokio::time::sleep(delay).await;
                        last_error = Some(err);
                    }
                    Err(()) => {
                        // Permanent error — bail immediately
                        tracing::debug!(
                            operation = %operation_name,
                            attempt = attempt + 1,
                            "Permanent error, not retrying: {err}"
                        );
                        return Err(err);
                    }
                }
            }
        }
    }

    // Should not reach here, but just in case
    Err(last_error.unwrap_or_else(|| {
        TsubasaError::Internal(format!("{operation_name}: retry loop exhausted without error"))
    }))
}

/// Simple pseudo-random f64 in [0, 1) without pulling in the `rand` crate.
/// Uses a quick xorshift on the current time nanos.
fn fastrand_f64() -> f64 {
    let mut seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos() as u64;

    // xorshift64
    seed ^= seed << 13;
    seed ^= seed >> 7;
    seed ^= seed << 17;

    (seed % 1_000_000) as f64 / 1_000_000.0
}
