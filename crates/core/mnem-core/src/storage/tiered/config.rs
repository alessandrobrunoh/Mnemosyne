use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierConfig {
    /// Items accessed within this window are kept uncompressed or lightly compressed
    pub hot_window_hours: u64,

    /// Items older than this move to high compression
    pub warm_window_days: u64,

    /// Compression level for cold storage (Zstd level 1-21)
    pub cold_compression_level: i32,
}

impl Default for TierConfig {
    fn default() -> Self {
        Self {
            hot_window_hours: 1,        // Keep last hour ultra-fast
            warm_window_days: 3,        // Keep last 3 days reasonably fast (LZ4)
            cold_compression_level: 15, // Everything else: Crush it (Zstd)
        }
    }
}
