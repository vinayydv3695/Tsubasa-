// Tsubasa — Cloud Provider Trait
// Abstract interface for debrid providers (Torbox, Real-Debrid, etc.)

pub mod provider;
pub mod torbox;
pub mod realdebrid;
pub mod download_driver;

use std::sync::Arc;

pub use provider::DebridProvider;
pub use torbox::TorboxProvider;
pub use realdebrid::RealDebridProvider;

/// Central manager for all cloud debrid providers.
/// Both providers are always present (but may not be configured with API keys).
/// Uses Arc so the manager can be shared without locking.
pub struct CloudManager {
    pub torbox: Arc<TorboxProvider>,
    pub realdebrid: Arc<RealDebridProvider>,
    /// Shared HTTP client for cloud file downloads.
    pub http_client: reqwest::Client,
}

impl CloudManager {
    /// Create a new CloudManager, initializing providers from saved API keys.
    pub fn new(
        torbox_api_key: Option<String>,
        realdebrid_api_key: Option<String>,
    ) -> Self {
        Self {
            torbox: Arc::new(TorboxProvider::new(torbox_api_key)),
            realdebrid: Arc::new(RealDebridProvider::new(realdebrid_api_key)),
            http_client: reqwest::Client::builder()
                .user_agent("Tsubasa/0.1.0")
                .build()
                .unwrap_or_else(|_| reqwest::Client::new()),
        }
    }

    /// Get a provider by name (case-insensitive).
    pub fn get_provider(&self, name: &str) -> Option<Arc<dyn DebridProvider>> {
        match name.to_lowercase().as_str() {
            "torbox" => Some(self.torbox.clone() as Arc<dyn DebridProvider>),
            "realdebrid" | "real-debrid" | "real_debrid" => {
                Some(self.realdebrid.clone() as Arc<dyn DebridProvider>)
            }
            _ => None,
        }
    }

    /// Update the Torbox API key at runtime.
    pub fn set_torbox_api_key(&self, key: String) {
        self.torbox.set_api_key(key);
    }

    /// Update the Real-Debrid API key at runtime.
    pub fn set_realdebrid_api_key(&self, key: String) {
        self.realdebrid.set_api_key(key);
    }

    /// Clear the Torbox API key.
    pub fn clear_torbox_api_key(&self) {
        self.torbox.clear_api_key();
    }

    /// Clear the Real-Debrid API key.
    pub fn clear_realdebrid_api_key(&self) {
        self.realdebrid.clear_api_key();
    }

    /// Check if any provider is configured.
    pub fn any_configured(&self) -> bool {
        self.torbox.is_configured() || self.realdebrid.is_configured()
    }

    /// Get all configured provider names.
    pub fn configured_providers(&self) -> Vec<&str> {
        let mut names = Vec::new();
        if self.torbox.is_configured() {
            names.push("Torbox");
        }
        if self.realdebrid.is_configured() {
            names.push("Real-Debrid");
        }
        names
    }
}
