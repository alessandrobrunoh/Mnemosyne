use super::StorageLayer;
use crate::error::{AppError, AppResult};
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

pub struct HotLayer {
    root: PathBuf,
}

impl HotLayer {
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

impl StorageLayer for HotLayer {
    fn write(&self, hash: &str, content: &[u8]) -> AppResult<()> {
        let path = self.root.join(hash);
        // Use Zstd Level 1 (fastest) for Hot Layer to save space immediately
        let compressed = zstd::encode_all(content, 1)
            .map_err(|e| AppError::Internal(format!("Zstd hot compress error: {}", e)))?;
        fs::write(path, compressed).map_err(AppError::IoGeneric)
    }

    fn read(&self, hash: &str) -> AppResult<Option<Vec<u8>>> {
        let path = self.root.join(hash);
        if path.exists() {
            let data = fs::read(&path).map_err(AppError::IoGeneric)?;
            // Attempt to decompress; if fails (legacy uncompressed), return raw
            match zstd::decode_all(&data[..]) {
                Ok(decompressed) => Ok(Some(decompressed)),
                Err(_) => {
                    // Fallback: assume it relies on legacy uncompressed format
                    Ok(Some(data))
                }
            }
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
