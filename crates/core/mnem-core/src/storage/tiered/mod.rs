pub mod cold_layer;
pub mod config;
pub mod hot_layer;
pub mod warm_layer;

pub use cold_layer::ColdLayer;
pub use hot_layer::HotLayer;
pub use warm_layer::WarmLayer;

use crate::error::{AppError, AppResult};
pub use config::TierConfig;
use std::path::PathBuf;

pub trait StorageLayer {
    fn write(&self, hash: &str, content: &[u8]) -> AppResult<()>;
    fn read(&self, hash: &str) -> AppResult<Option<Vec<u8>>>;
    fn delete(&self, hash: &str) -> AppResult<()>;
    fn exists(&self, hash: &str) -> bool;
    fn get_size(&self, hash: &str) -> AppResult<u64>;
}

pub struct TieredStore {
    config: TierConfig,
    // Layers
    hot: hot_layer::HotLayer,
    warm: warm_layer::WarmLayer,
    cold: cold_layer::ColdLayer,
}

impl TieredStore {
    pub fn new(base_dir: PathBuf, config: TierConfig) -> AppResult<Self> {
        let cas_root = base_dir.join("cas");
        std::fs::create_dir_all(&cas_root).map_err(|e| AppError::Io {
            path: cas_root.clone(),
            source: e,
        })?;

        Ok(Self {
            config,
            hot: hot_layer::HotLayer::new(cas_root.join("hot")),
            warm: warm_layer::WarmLayer::new(cas_root.join("warm")),
            cold: cold_layer::ColdLayer::new(cas_root.join("cold")),
        })
    }

    pub fn write(&self, hash: &str, content: &[u8]) -> AppResult<()> {
        // Always write to HOT first for speed
        self.hot.write(hash, content)
    }

    pub fn read(&self, hash: &str) -> AppResult<Vec<u8>> {
        // Waterfall read: Hot -> Warm -> Cold
        if let Some(data) = self.hot.read(hash)? {
            return Ok(data);
        }
        if let Some(data) = self.warm.read(hash)? {
            // Promote to hot? Maybe not on simple read, only on edit.
            return Ok(data);
        }
        if let Some(data) = self.cold.read(hash)? {
            return Ok(data);
        }
        Err(AppError::NotFound(format!("Chunk {} not found", hash)))
    }

    pub fn get_size(&self, hash: &str) -> AppResult<u64> {
        if let Ok(size) = self.hot.get_size(hash) {
            return Ok(size);
        }
        if let Ok(size) = self.warm.get_size(hash) {
            return Ok(size);
        }
        if let Ok(size) = self.cold.get_size(hash) {
            return Ok(size);
        }
        Ok(0)
    }

    pub fn delete(&self, hash: &str) -> AppResult<()> {
        let _ = self.hot.delete(hash);
        let _ = self.warm.delete(hash);
        let _ = self.cold.delete(hash);
        Ok(())
    }

    pub fn exists(&self, hash: &str) -> bool {
        self.hot.exists(hash) || self.warm.exists(hash) || self.cold.exists(hash)
    }

    /// Run the migration logic: move old Hot items to Warm, old Warm to Cold.
    pub fn migrate(&self) -> AppResult<usize> {
        let mut moved = 0;

        // 1. Hot -> Warm
        // Iterate hot files, check age > hot_window
        // If old: compress to warm, delete from hot
        let hot = self.hot.scan()?;
        for (hash, age) in hot {
            if age.as_secs() > self.config.hot_window_hours * 3600 {
                if let Some(content) = self.hot.read(&hash)? {
                    self.warm.write(&hash, &content)?;
                    self.hot.delete(&hash)?;
                    moved += 1;
                }
            }
        }

        // 2. Warm -> Cold
        // Iterate warm files, check age > warm_window
        // If old: compress harder to cold, delete from warm
        let warm = self.warm.scan()?;
        for (hash, age) in warm {
            if age.as_secs() > self.config.warm_window_days * 86400 {
                if let Some(content) = self.warm.read(&hash)? {
                    self.cold.write(&hash, &content)?;
                    self.warm.delete(&hash)?;
                    moved += 1;
                }
            }
        }

        Ok(moved)
    }

    pub fn clean_temp(&self) -> AppResult<usize> {
        // Placeholder for now, maybe clean partial uploads
        Ok(0)
    }
}
