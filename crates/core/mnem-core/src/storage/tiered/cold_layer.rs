use super::StorageLayer;
use crate::error::{AppError, AppResult};
use std::fs;
use std::path::PathBuf;

pub struct ColdLayer {
    root: PathBuf,
}

impl ColdLayer {
    pub fn new(root: PathBuf) -> Self {
        fs::create_dir_all(&root).ok();
        Self { root }
    }
}

impl StorageLayer for ColdLayer {
    fn write(&self, hash: &str, content: &[u8]) -> AppResult<()> {
        let path = self.root.join(hash);
        // Zstd High Compression (Level 15 is a good balance for cold)
        let compressed = zstd::encode_all(content, 15)
            .map_err(|e| AppError::Internal(format!("Zstd compress error: {}", e)))?;
        fs::write(path, compressed).map_err(AppError::IoGeneric)
    }

    fn read(&self, hash: &str) -> AppResult<Option<Vec<u8>>> {
        let path = self.root.join(hash);
        if path.exists() {
            let compressed = fs::read(path).map_err(AppError::IoGeneric)?;
            let decompressed = zstd::decode_all(&compressed[..])
                .map_err(|e| AppError::Internal(format!("Zstd decompress error: {}", e)))?;
            Ok(Some(decompressed))
        } else {
            Ok(None)
        }
    }

    fn delete(&self, hash: &str) -> AppResult<()> {
        let path = self.root.join(hash);
        if path.exists() {
            fs::remove_file(path).map_err(AppError::IoGeneric)?;
        }
        Ok(())
    }

    fn exists(&self, hash: &str) -> bool {
        self.root.join(hash).exists()
    }

    fn get_size(&self, hash: &str) -> AppResult<u64> {
        let path = self.root.join(hash);
        if path.exists() {
            Ok(fs::metadata(path).map_err(AppError::IoGeneric)?.len())
        } else {
            Err(AppError::IoGeneric(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "not found",
            )))
        }
    }
}
