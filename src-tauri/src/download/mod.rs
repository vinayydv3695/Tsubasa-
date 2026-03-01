// Tsubasa (翼) — Download Policy
// Defines how each torrent chooses its download path.

pub mod state_machine;
pub mod orchestrator;

pub use state_machine::{DownloadPolicy, TorrentState};
pub use orchestrator::DownloadOrchestrator;
