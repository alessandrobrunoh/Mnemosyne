use crate::error::{AppError, AppResult};
use std::fs;

use crate::env::get_base_dir;

pub const TOKEN_FILE: &str = ".daemon-token";

pub struct AuthManager;

impl AuthManager {
    /// Generates a new random token and saves it to the base directory.
    /// Sets restricted permissions on the file.
    pub fn generate_token() -> AppResult<String> {
        let base_dir = get_base_dir()?;
        let token_path = base_dir.join(TOKEN_FILE);

        let token = uuid::Uuid::new_v4().to_string();

        fs::write(&token_path, &token).map_err(|e| AppError::Io {
            path: token_path.clone(),
            source: e,
        })?;

        // Restrict permissions
        Self::restrict_permissions(&token_path)?;

        Ok(token)
    }

    /// Reads the existing token from the base directory.
    pub fn get_token() -> AppResult<String> {
        let base_dir = get_base_dir()?;
        let token_path = base_dir.join(TOKEN_FILE);

        if !token_path.exists() {
            return Err(AppError::NotFound(
                "Auth token not found. Daemon might not be running.".into(),
            ));
        }

        let token = fs::read_to_string(&token_path).map_err(|e| AppError::Io {
            path: token_path,
            source: e,
        })?;

        Ok(token.trim().to_string())
    }

    #[cfg(unix)]
    fn restrict_permissions(path: &std::path::Path) -> AppResult<()> {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(path)
            .map_err(|e| AppError::Io {
                path: path.to_path_buf(),
                source: e,
            })?
            .permissions();
        perms.set_mode(0o600); // User Read/Write only
        fs::set_permissions(path, perms).map_err(|e| AppError::Io {
            path: path.to_path_buf(),
            source: e,
        })?;
        Ok(())
    }

    #[cfg(windows)]
    fn restrict_permissions(_path: &std::path::Path) -> AppResult<()> {
        // Windows permissions are more complex to set via std::fs.
        // For now, we rely on the fact that it's in the user's home directory.
        // TODO: Implement NTFS ACL restriction if critical.
        Ok(())
    }
}
