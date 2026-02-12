use crate::error::{AppError, AppResult};
use std::path::PathBuf;

/// Environment variable to override the default Mnemosyne data directory.
const ENV_DATA_DIR: &str = "MNEMOSYNE_HOME";

/// Returns the base directory for Mnemosyne data.
///
/// Checks for `MNEMOSYNE_HOME` environment variable first.
/// If not set, falls back to `~/.mnemosyne` (or equivalent on Windows).
///
/// # Returns
/// * `Ok(PathBuf)` - The base directory path
/// * `Err(AppError)` - If home directory cannot be determined
///
/// # Security
/// This function is designed to avoid panicking in environments where
/// the home directory is not available (e.g., Docker containers, systemd services).
pub fn get_base_dir() -> AppResult<PathBuf> {
    // 1. Check for environment variable override
    if let Ok(env_path) = std::env::var(ENV_DATA_DIR) {
        let path = PathBuf::from(env_path);
        // Validate that the path is absolute
        if !path.is_absolute() {
            return Err(AppError::Config(format!(
                "Environment variable {} must be an absolute path, got: {:?}",
                ENV_DATA_DIR, path
            )));
        }
        return Ok(path);
    }

    // 2. Fall back to home directory
    match dirs::home_dir() {
        Some(home) => Ok(home.join(".mnemosyne")),
        None => Err(AppError::Config(
            "Cannot determine home directory. Please set MNEMOSYNE_HOME environment variable."
                .to_string(),
        )),
    }
}

/// Returns the path to the Mnemosyne socket directory.
pub fn get_socket_dir() -> AppResult<PathBuf> {
    get_base_dir()
}

/// Returns the path to the project registry file.
pub fn get_registry_path() -> AppResult<PathBuf> {
    Ok(get_base_dir()?.join("registry.json"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_get_base_dir_env_override() {
        // Use a platform-appropriate absolute path
        let test_path = if cfg!(windows) {
            r"C:\temp\mnemosyne_test"
        } else {
            "/tmp/mnemosyne_test"
        };
        env::set_var(ENV_DATA_DIR, test_path);

        let result = get_base_dir();
        assert!(result.is_ok(), "get_base_dir() failed: {:?}", result);
        assert_eq!(result.unwrap(), PathBuf::from(test_path));

        env::remove_var(ENV_DATA_DIR);
    }

    #[test]
    fn test_get_base_dir_relative_path_rejected() {
        env::set_var(ENV_DATA_DIR, "relative/path");

        let result = get_base_dir();
        assert!(result.is_err());

        env::remove_var(ENV_DATA_DIR);
    }
}
