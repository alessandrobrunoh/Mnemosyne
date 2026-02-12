use super::StorageLayer;
use crate::error::{AppError, AppResult};
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

pub struct WarmLayer {
    root: PathBuf,
}

impl WarmLayer {
    pub fn new(root: PathBuf) -> Self {
        fs::create_dir_all(&root).ok();
        Self { root }
    }

    pub fn scan(&self) -> AppResult<Vec<(String, Duration)>> {
        let mut results = Vec::new();
        if !self.root.exists() {
            return Ok(results);
        }

        for entry in fs::read_dir(&self.root).map_err(AppError::IoGeneric)? {
            let entry = entry.map_err(AppError::IoGeneric)?;
            if let Ok(meta) = entry.metadata() {
                if let Ok(modified) = meta.modified() {
                    if let Ok(age) = SystemTime::now().duration_since(modified) {
                        if let Ok(name) = entry.file_name().into_string() {
                            results.push((name, age));
                        }
                    }
                }
            }
        }
        Ok(results)
    }
}

impl StorageLayer for WarmLayer {
    fn write(&self, hash: &str, content: &[u8]) -> AppResult<()> {
        let path = self.root.join(hash);
        // Zstd Fast Compression (Level 3) - replacement for LZ4
        let compressed = zstd::encode_all(content, 3)
            .map_err(|e| AppError::Internal(format!("Zstd warm compress error: {}", e)))?;
        fs::write(path, compressed).map_err(AppError::IoGeneric)
    }

    fn read(&self, hash: &str) -> AppResult<Option<Vec<u8>>> {
        let path = self.root.join(hash);
        if path.exists() {
            let compressed = fs::read(path).map_err(AppError::IoGeneric)?;
            let decompressed = zstd::decode_all(&compressed[..])
                .map_err(|e| AppError::Internal(format!("Zstd warm decompress error: {}", e)))?;
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
