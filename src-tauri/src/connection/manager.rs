// Tsubasa (翼) — Connection Manager
// Global and per-torrent connection limits.
// UPnP/NAT-PMP port forwarding.
// Interface binding.

use std::sync::Arc;

use parking_lot::RwLock;

use crate::settings::schema::ConnectionSettings;

/// Manages connection limits and port configuration.
pub struct ConnectionManager {
    config: Arc<RwLock<ConnectionSettings>>,
}

impl ConnectionManager {
    pub fn new(config: Arc<RwLock<ConnectionSettings>>) -> Self {
        Self { config }
    }

    /// Get current connection settings.
    pub fn settings(&self) -> ConnectionSettings {
        self.config.read().clone()
    }

    /// Update connection settings live.
    /// Most settings take effect immediately.
    /// `listen_port` and `bind_interface` require engine restart.
    pub fn update(&self, new: ConnectionSettings) {
        let old = self.config.read().clone();

        let needs_restart = old.listen_port != new.listen_port
            || old.bind_interface != new.bind_interface;

        *self.config.write() = new;

        if needs_restart {
            tracing::warn!(
                "Connection settings changed: listen_port or bind_interface modified. \
                 Engine restart required for these changes to take effect."
            );
        }
    }

    /// Check if a specific setting requires engine restart.
    pub fn requires_restart(field: &str) -> bool {
        matches!(field, "listen_port" | "bind_interface")
    }

    /// Get the global max connections.
    pub fn global_max_connections(&self) -> u32 {
        self.config.read().global_max_connections
    }

    /// Get per-torrent max connections.
    pub fn per_torrent_max_connections(&self) -> u32 {
        self.config.read().per_torrent_max_connections
    }

    /// Check if UPnP is enabled.
    pub fn upnp_enabled(&self) -> bool {
        self.config.read().enable_upnp
    }

    /// Check if uTP is enabled.
    pub fn utp_enabled(&self) -> bool {
        self.config.read().enable_utp
    }
}
