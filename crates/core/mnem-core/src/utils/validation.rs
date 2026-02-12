use crate::error::{AppError, AppResult};
use std::path::{Path, PathBuf};

/// Ensures a path is safe to use within a project context.
pub struct PathValidator;

impl PathValidator {
    /// Validates that `target` is within `base`.
    /// Returns the canonicalized target path if safe.
    pub fn validate_within(base: &Path, target: &Path) -> AppResult<PathBuf> {
        let base_canonical = base.canonicalize().map_err(|e| AppError::Io {
            path: base.to_path_buf(),
            source: e,
        })?;

        // If target doesn't exist yet, we check its parent
        let target_canonical = if target.exists() {
            target.canonicalize().map_err(|e| AppError::Io {
                path: target.to_path_buf(),
                source: e,
            })?
        } else {
            let parent = target
                .parent()
                .ok_or_else(|| AppError::Security("Invalid target path".into()))?;
            let parent_canonical = parent.canonicalize().map_err(|e| AppError::Io {
                path: parent.to_path_buf(),
                source: e,
            })?;
            parent_canonical.join(
                target
                    .file_name()
                    .ok_or_else(|| AppError::Security("Invalid filename".into()))?,
            )
        };

        if target_canonical.starts_with(&base_canonical) {
            Ok(target_canonical)
        } else {
            Err(AppError::PathTraversal(target.to_path_buf()))
        }
    }

    /// Checks if a hash string is a valid BLAKE3 hex string (64 chars).
    pub fn is_valid_hash(hash: &str) -> bool {
        hash.len() == 64 && hash.chars().all(|c| c.is_ascii_hexdigit())
    }
}
