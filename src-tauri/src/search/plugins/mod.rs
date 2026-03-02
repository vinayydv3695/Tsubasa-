// Tsubasa (翼) — Search Plugins Registry
// Built-in torrent search plugins.

pub mod piratebay;
pub mod yts;
pub mod leet;
pub mod nyaa;
pub mod torrentgalaxy;

use super::plugin::SearchPlugin;

/// Create all built-in search plugins.
pub fn all_plugins() -> Vec<Box<dyn SearchPlugin>> {
    vec![
        Box::new(piratebay::PirateBayPlugin),
        Box::new(yts::YtsPlugin),
        Box::new(leet::LeetPlugin),
        Box::new(nyaa::NyaaPlugin),
        Box::new(torrentgalaxy::TorrentGalaxyPlugin),
    ]
}
