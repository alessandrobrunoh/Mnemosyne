use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub enum Ide {
    Zed,
    ZedPreview,
    VsCode,
}

impl Default for Ide {
    fn default() -> Self {
        Self::Zed
    }
}

impl Ide {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Zed => "Zed",
            Self::ZedPreview => "Zed Preview",
            Self::VsCode => "Visual Studio Code",
        }
    }

    pub fn command_name(&self) -> &'static str {
        match self {
            Self::Zed => "Zed",
            Self::ZedPreview => "Zed Preview",
            Self::VsCode => "Visual Studio Code",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub retention_days: u64,
    pub compression_enabled: bool,
    pub use_gitignore: bool,
    pub use_mnemosyneignore: bool,
    pub theme_index: usize,
    #[serde(default = "default_max_file_size_mb")]
    pub max_file_size_mb: u64,
    #[serde(default)]
    pub ide: Ide,
}

fn default_max_file_size_mb() -> u64 {
    10
}

impl Default for Config {
    fn default() -> Self {
        Self {
            retention_days: 30,
            compression_enabled: true,
            use_gitignore: true,
            use_mnemosyneignore: true,
            theme_index: 0,
            max_file_size_mb: default_max_file_size_mb(),
            ide: Ide::default(),
        }
    }
}

pub struct ConfigManager {
    config_path: PathBuf,
    pub config: Config,
}

impl ConfigManager {
    pub fn new(base_dir: &std::path::Path) -> AppResult<Self> {
        let config_path = base_dir.join("config.toml");
        let config = if config_path.exists() {
            let content = std::fs::read_to_string(&config_path).map_err(AppError::IoGeneric)?;
            toml::from_str(&content).unwrap_or_default()
        } else {
            Config::default()
        };

        // Auto-save default if missing
        if !config_path.exists() {
            if let Err(e) = Self::save_to_path(&config, &config_path) {
                eprintln!("Warning: failed to save default config: {}", e);
            }
        }

        // Create default global .mnemignore if missing
        let ignore_path = base_dir.join(".mnemignore");
        if !ignore_path.exists() {
            let default_ignore = r#"# Mnemosyne Default Global Ignore
# Paths to ignore across ALL projects

# Build artifacts
target/
dist/
build/
out/
bin/
obj/

# Dependencies
node_modules/
vendor/
packages/
.pnp/
.pnp.js

# Environment and secrets
.env
.env.local
*.pem
*.key

# IDEs and OS files
.DS_Store
.idea/
.vscode/
*.sublime-project
*.swp
*.swo

# Lock files (usually large and noisy)
Cargo.lock
package-lock.json
yarn.lock
pnpm-lock.yaml
composer.lock
poetry.lock
Gemfile.lock

# Mnemosyne internal
.mnemosyne/
"#;
            let _ = std::fs::write(&ignore_path, default_ignore);
        }

        Ok(Self {
            config_path,
            config,
        })
    }

    pub fn save(&self) -> AppResult<()> {
        Self::save_to_path(&self.config, &self.config_path)
    }

    fn save_to_path(config: &Config, path: &PathBuf) -> AppResult<()> {
        let content =
            toml::to_string_pretty(config).map_err(|e| AppError::Config(e.to_string()))?;

        // Atomic write: write to tempfile then rename to prevent corruption on crash
        let parent = path.parent().unwrap_or(std::path::Path::new("."));
        let temp = tempfile::NamedTempFile::new_in(parent).map_err(AppError::IoGeneric)?;
        std::fs::write(temp.path(), &content).map_err(AppError::IoGeneric)?;
        temp.persist(path)
            .map_err(|e| AppError::IoGeneric(e.error))?;
        Ok(())
    }

    pub fn update_retention(&mut self, days: u64) -> AppResult<()> {
        self.config.retention_days = days;
        self.save()
    }

    pub fn toggle_compression(&mut self) -> AppResult<()> {
        self.config.compression_enabled = !self.config.compression_enabled;
        self.save()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn default_config_created_when_missing() {
        let dir = TempDir::new().unwrap();
        let config_manager = ConfigManager::new(dir.path()).unwrap();
        assert_eq!(config_manager.config.retention_days, 30);
        assert!(dir.path().join("config.toml").exists());
    }

    #[test]
    fn config_persists_across_loads() {
        let dir = TempDir::new().unwrap();
        {
            let mut config_manager = ConfigManager::new(dir.path()).unwrap();
            config_manager.update_retention(7).unwrap();
        }
        let config_manager = ConfigManager::new(dir.path()).unwrap();
        assert_eq!(config_manager.config.retention_days, 7);
    }

    #[test]
    fn test_toggle_compression() {
        let dir = TempDir::new().unwrap();
        let mut config_manager = ConfigManager::new(dir.path()).unwrap();
        let initial = config_manager.config.compression_enabled;
        config_manager.toggle_compression().unwrap();
        assert_eq!(config_manager.config.compression_enabled, !initial);
    }
}
