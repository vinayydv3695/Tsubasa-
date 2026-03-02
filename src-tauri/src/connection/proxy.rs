// Tsubasa (翼) — Proxy Manager
// SOCKS4/SOCKS5/HTTP proxy configuration.
// Builds pre-configured reqwest client with proxy settings.

use std::sync::Arc;

use parking_lot::RwLock;

use crate::settings::schema::{ProxySettings, ProxyType};

/// Manages proxy configuration and builds HTTP clients with proxy applied.
pub struct ProxyManager {
    config: Arc<RwLock<ProxySettings>>,
}

impl ProxyManager {
    pub fn new(config: Arc<RwLock<ProxySettings>>) -> Self {
        Self { config }
    }

    /// Get current proxy settings.
    pub fn settings(&self) -> ProxySettings {
        self.config.read().clone()
    }

    /// Check if a proxy is configured and active.
    pub fn is_active(&self) -> bool {
        let config = self.config.read();
        config.proxy_type != ProxyType::None && !config.host.is_empty()
    }

    /// Build the proxy URL string.
    pub fn proxy_url(&self) -> Option<String> {
        let config = self.config.read();
        if config.proxy_type == ProxyType::None || config.host.is_empty() {
            return None;
        }

        let scheme = match config.proxy_type {
            ProxyType::Socks4 => "socks4",
            ProxyType::Socks5 => "socks5",
            ProxyType::Http => "http",
            ProxyType::None => return None,
        };

        let auth = config.auth.as_ref().map(|a| {
            format!("{}:{}@", a.username, a.password)
        }).unwrap_or_default();

        Some(format!("{}://{}{}:{}", scheme, auth, config.host, config.port))
    }

    /// Build a reqwest::Client pre-configured with the current proxy settings.
    pub fn build_client(&self) -> reqwest::Client {
        let mut builder = reqwest::Client::builder()
            .user_agent("Tsubasa/0.1.0")
            .timeout(std::time::Duration::from_secs(30));

        if let Some(proxy_url) = self.proxy_url() {
            match reqwest::Proxy::all(&proxy_url) {
                Ok(proxy) => { builder = builder.proxy(proxy); }
                Err(e) => {
                    tracing::error!(url = %proxy_url, error = %e, "Failed to configure proxy");
                }
            }
        }

        builder.build().unwrap_or_else(|_| reqwest::Client::new())
    }

    /// Update proxy settings live.
    pub fn update(&self, new: ProxySettings) {
        *self.config.write() = new;
    }

    /// Should search traffic go through the proxy?
    pub fn apply_to_search(&self) -> bool {
        self.config.read().apply_to_search
    }

    /// Should tracker traffic go through the proxy?
    pub fn apply_to_trackers(&self) -> bool {
        self.config.read().apply_to_trackers
    }
}
