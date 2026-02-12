use crate::error::{AppError, AppResult};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

/// BLAKE3 hashes are exactly 64 hex characters (256 bits).
/// Validates that a hash string is well-formed before using it as a filesystem path,
/// preventing path traversal attacks (audit 1.1).
fn validate_hash(hash: &str) -> AppResult<()> {
    if hash.len() != 64 || !hash.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(AppError::Internal(format!("Invalid hash format: {}", hash)));
    }
    Ok(())
}

/// Heuristic check: if the data starts with known magic bytes of compressed formats,
/// skip re-compression to avoid wasted CPU (audit 3.3).
fn is_already_compressed(data: &[u8]) -> bool {
    if data.len() < 4 {
        return false;
    }
    // ZIP / DOCX / XLSX
    if data[0] == 0x50 && data[1] == 0x4B {
        return true;
    }
    // GZIP
    if data[0] == 0x1F && data[1] == 0x8B {
        return true;
    }
    // ZSTD
    if data[0] == 0x28 && data[1] == 0xB5 && data[2] == 0x2F && data[3] == 0xFD {
        return true;
    }
    // PNG
    if data[0] == 0x89 && data[1] == 0x50 && data[2] == 0x4E && data[3] == 0x47 {
        return true;
    }
    // JPEG
    if data[0] == 0xFF && data[1] == 0xD8 {
        return true;
    }
    // BZ2
    if data[0] == 0x42 && data[1] == 0x5A && data[2] == 0x68 {
        return true;
    }
    // XZ
    if data[0] == 0xFD && data[1] == 0x37 && data[2] == 0x7A && data[3] == 0x58 {
        return true;
    }
    false
}

pub struct CasStorage {
    base_dir: PathBuf,
}

impl CasStorage {
    pub fn new(base_dir: PathBuf) -> AppResult<Self> {
        let objects_dir = base_dir.join("objects");
        let temp_dir = base_dir.join("tmp");
        fs::create_dir_all(&objects_dir).map_err(AppError::IoGeneric)?;
        fs::create_dir_all(&temp_dir).map_err(AppError::IoGeneric)?;

        // Security: Set directory permissions to 700 (Unix only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(metadata) = fs::metadata(&base_dir) {
                let mut perms = metadata.permissions();
                perms.set_mode(0o700);
                if let Err(e) = fs::set_permissions(&base_dir, perms) {
                    eprintln!(
                        "Warning: failed to set permissions on {:?}: {}",
                        base_dir, e
                    );
                }
            }
        }

        Ok(Self { base_dir })
    }

    /// Build the sharded object path: `objects/ab/cdef...` (audit 3.5).
    /// Uses 2-char prefix directories to avoid flat directory performance issues.
    fn object_path(&self, hash: &str) -> PathBuf {
        let (prefix, rest) = hash.split_at(2);
        self.base_dir.join("objects").join(prefix).join(rest)
    }

    /// Compute hash of a file without writing anything (for dedup-first pattern, audit 5.4).
    /// Returns the BLAKE3 hex hash of the file's raw content.
    pub fn compute_hash(&self, input_path: &Path) -> AppResult<String> {
        let mut file = fs::File::open(input_path).map_err(AppError::IoGeneric)?;
        let mut hasher = blake3::Hasher::new();
        let mut buffer = [0u8; 64 * 1024];
        loop {
            let count = file.read(&mut buffer).map_err(AppError::IoGeneric)?;
            if count == 0 {
                break;
            }
            hasher.update(&buffer[..count]);
        }
        Ok(hasher.finalize().to_hex().to_string())
    }

    pub fn get_path(&self, hash: &str) -> PathBuf {
        self.object_path(hash)
    }

    /// Streaming write: Reads from input, compresses, hashes, and writes atomically.
    /// Returns the hash of the ORIGINAL content.
    pub fn write_stream(&self, input_path: &Path) -> AppResult<String> {
        let mut input_file = fs::File::open(input_path).map_err(AppError::IoGeneric)?;

        // Read the first chunk to detect if already compressed
        let mut first_chunk = [0u8; 64 * 1024];
        let first_count = input_file
            .read(&mut first_chunk)
            .map_err(AppError::IoGeneric)?;
        let compression_level = if is_already_compressed(&first_chunk[..first_count]) {
            0
        } else {
            3
        };

        // Create a temporary file in dedicated tmp dir
        let temp_dir = self.base_dir.join("tmp");
        let mut temp_file =
            tempfile::NamedTempFile::new_in(&temp_dir).map_err(AppError::IoGeneric)?;

        let mut hasher = blake3::Hasher::new();
        let mut encoder = zstd::stream::write::Encoder::new(&mut temp_file, compression_level)
            .map_err(AppError::IoGeneric)?;

        // Process first chunk
        hasher.update(&first_chunk[..first_count]);
        encoder
            .write_all(&first_chunk[..first_count])
            .map_err(AppError::IoGeneric)?;

        // Buffer for streaming (64KB chunks)
        let mut buffer = [0u8; 64 * 1024];
        loop {
            let count = input_file.read(&mut buffer).map_err(AppError::IoGeneric)?;
            if count == 0 {
                break;
            }
            hasher.update(&buffer[..count]);
            encoder
                .write_all(&buffer[..count])
                .map_err(AppError::IoGeneric)?;
        }

        encoder.finish().map_err(AppError::IoGeneric)?;

        let hash = hasher.finalize().to_hex().to_string();
        let target_path = self.object_path(&hash);

        // Ensure the shard directory exists
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent).map_err(AppError::IoGeneric)?;
        }

        // Atomic rename, avoiding TOCTOU race (audit 5.3)
        match temp_file.persist_noclobber(&target_path) {
            Ok(_) => {}
            Err(e) if e.error.kind() == std::io::ErrorKind::AlreadyExists => {
                // Content-addressed: identical hash means identical content, safe to skip
            }
            Err(e) => return Err(AppError::IoGeneric(e.error)),
        }

        Ok(hash)
    }

    /// Write content from a byte slice. Uses the same sharding and atomic write strategy.
    pub fn write(&self, content: &[u8]) -> AppResult<String> {
        let hash = blake3::hash(content).to_hex().to_string();
        let object_path = self.object_path(&hash);

        if let Some(parent) = object_path.parent() {
            fs::create_dir_all(parent).map_err(AppError::IoGeneric)?;
        }

        // --- Semantic Delta Compression Logic ---
        // Optimization: Use Zstd with a higher compression level for text chunks
        // and optionally train a dictionary (advanced future feature).
        // Currently, we just ensure optimal settings for code.
        let compression_level = if is_already_compressed(content) {
            0
        } else {
            // Level 7-9 is great for small code chunks without being too slow
            9
        };

        // Atomic write with persist_noclobber to avoid TOCTOU race
        let compressed = zstd::stream::encode_all(std::io::Cursor::new(content), compression_level)
            .map_err(AppError::IoGeneric)?;

        let temp_dir = self.base_dir.join("tmp");
        let temp = tempfile::NamedTempFile::new_in(temp_dir).map_err(AppError::IoGeneric)?;
        fs::write(temp.path(), &compressed).map_err(AppError::IoGeneric)?;

        match temp.persist_noclobber(&object_path) {
            Ok(_) => {}
            Err(e) if e.error.kind() == std::io::ErrorKind::AlreadyExists => {
                // CAS: identical hash = identical content, safe to skip
            }
            Err(e) => return Err(AppError::IoGeneric(e.error)),
        }

        Ok(hash)
    }

    /// Read and decompress an object by its hash.
    /// Validates hash format to prevent path traversal (audit 1.1).
    pub fn read(&self, hash: &str) -> AppResult<Vec<u8>> {
        validate_hash(hash)?;
        let object_path = self.object_path(hash);

        // Fallback: check legacy flat path for migration compatibility
        let actual_path = if object_path.exists() {
            object_path
        } else {
            let legacy = self.base_dir.join("objects").join(hash);
            if legacy.exists() {
                legacy
            } else {
                object_path
            }
        };

        let raw = fs::read(&actual_path).map_err(|e| {
            AppError::IoGeneric(std::io::Error::new(
                e.kind(),
                format!("Failed to read object {:?}: {}", actual_path, e),
            ))
        })?;

        // Decompress with size limit to prevent decompression bombs
        const MAX_DECOMPRESSED_SIZE: usize = 256 * 1024 * 1024; // 256 MB

        match zstd::stream::Decoder::new(std::io::Cursor::new(&raw)) {
            Ok(mut decoder) => {
                let mut decompressed = Vec::new();
                let mut buf = [0u8; 64 * 1024];
                loop {
                    let n = decoder
                        .read(&mut buf)
                        .map_err(|_| AppError::Internal("Decompression failed".into()))?;
                    if n == 0 {
                        break;
                    }
                    if decompressed.len() + n > MAX_DECOMPRESSED_SIZE {
                        return Err(AppError::Internal(format!(
                            "Decompressed size exceeds {} MB limit",
                            MAX_DECOMPRESSED_SIZE / (1024 * 1024)
                        )));
                    }
                    decompressed.extend_from_slice(&buf[..n]);
                }
                Ok(decompressed)
            }
            Err(_) => Ok(raw), // Fallback: assume it was uncompressed (legacy file)
        }
    }

    /// Check if an object exists by hash.
    pub fn exists(&self, hash: &str) -> bool {
        if validate_hash(hash).is_err() {
            return false;
        }
        let sharded = self.object_path(hash);
        if sharded.exists() {
            return true;
        }
        // Fallback: check legacy flat path
        self.base_dir.join("objects").join(hash).exists()
    }

    /// Delete an object by hash (used by GC to clean orphan files, audit 3.2).
    pub fn delete(&self, hash: &str) -> AppResult<()> {
        validate_hash(hash)?;
        let sharded = self.object_path(hash);
        if sharded.exists() {
            fs::remove_file(&sharded).map_err(AppError::IoGeneric)?;
        } else {
            // Fallback: try legacy flat path
            let legacy = self.base_dir.join("objects").join(hash);
            if legacy.exists() {
                fs::remove_file(&legacy).map_err(AppError::IoGeneric)?;
            }
        }
        Ok(())
    }

    pub fn clean_temp(&self) -> AppResult<usize> {
        let temp_dir = self.base_dir.join("tmp");
        if !temp_dir.exists() {
            return Ok(0);
        }

        let mut count = 0;
        if let Ok(entries) = fs::read_dir(&temp_dir) {
            for entry in entries.flatten() {
                if let Ok(meta) = entry.metadata() {
                    if meta.is_file() {
                        if let Ok(modified) = meta.modified() {
                            if let Ok(age) = modified.elapsed() {
                                if age > std::time::Duration::from_secs(3600) {
                                    if fs::remove_file(entry.path()).is_ok() {
                                        count += 1;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup() -> (TempDir, CasStorage) {
        let dir = TempDir::new().unwrap();
        let storage = CasStorage::new(dir.path().to_path_buf()).unwrap();
        (dir, storage)
    }

    #[test]
    fn write_and_read_roundtrip() {
        let (_dir, storage) = setup();
        let content = b"Hello, Mnemosyne!";
        let hash = storage.write(content).unwrap();
        let result = storage.read(&hash).unwrap();
        assert_eq!(result, content);
    }

    #[test]
    fn deduplication_same_content() {
        let (_dir, storage) = setup();
        let content = b"same content";
        let hash1 = storage.write(content).unwrap();
        let hash2 = storage.write(content).unwrap();
        assert_eq!(hash1, hash2);

        // Check that only one file exists in objects
        let objects_dir = _dir.path().join("objects");
        let mut count = 0;
        for entry in fs::read_dir(objects_dir).unwrap() {
            let entry = entry.unwrap();
            if entry.path().is_dir() {
                count += fs::read_dir(entry.path()).unwrap().count();
            }
        }
        assert_eq!(count, 1);
    }

    #[test]
    fn read_nonexistent_hash_returns_error() {
        let (_dir, storage) = setup();
        let result =
            storage.read("00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff");
        assert!(result.is_err());
    }

    #[test]
    fn path_traversal_rejected() {
        let (_dir, storage) = setup();
        let result = storage.read("../../etc/passwd");
        assert!(result.is_err());
    }

    #[test]
    fn large_file_streaming() {
        let (_dir, storage) = setup();
        let temp_dir = TempDir::new().unwrap();
        let large_file = temp_dir.path().join("large.txt");

        // Create a 5MB file to test streaming and compression
        let chunk = vec![b'x'; 1024 * 1024];
        let mut f = fs::File::create(&large_file).unwrap();
        for _ in 0..5 {
            f.write_all(&chunk).unwrap();
        }
        drop(f);

        let hash = storage.write_stream(&large_file).unwrap();
        let result = storage.read(&hash).unwrap();
        assert_eq!(result.len(), 5 * 1024 * 1024);
        assert!(result.iter().all(|&b| b == b'x'));
    }

    #[test]
    fn test_already_compressed_detection() {
        // PNG Magic bytes
        let png_data = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        assert!(is_already_compressed(&png_data));

        // Plain text
        let text_data = b"This is plain text and should be compressed";
        assert!(!is_already_compressed(text_data));
    }
}
