use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Default, Eq)]
pub enum Ide {
    #[default]
    Zed,
    ZedPreview,
    VsCode,
}

impl Ide {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Zed => "Zed",
            Self::ZedPreview => "Zed Preview",
            Self::VsCode => "VsCode",
        }
    }
}

impl FromStr for Ide {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "zed" => Ok(Self::Zed),
            "zed-preview" | "zedpreview" => Ok(Self::ZedPreview),
            "vscode" | "vs-code" => Ok(Self::VsCode),
            _ => Err(()),
        }
    }
}

impl Ide {
    pub fn from_str_opt(s: &str) -> Option<Self> {
        Ide::from_str(s).ok()
    }
}

/// Storage configuration - manages retention and disk usage
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StorageConfig {
    #[serde(default = "default_retention_days")]
    pub retention_days: u64,
    #[serde(default = "default_true")]
    pub compression_enabled: bool,
    #[serde(default = "default_true")]
    pub use_mnemosyneignore: bool,
    #[serde(default = "default_max_file_size")]
    pub max_file_size_mb: u64,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            retention_days: default_retention_days(),
            compression_enabled: true,
            use_mnemosyneignore: true,
            max_file_size_mb: default_max_file_size(),
        }
    }
}

/// UI configuration - manages visual settings and themes
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct UiConfig {
    #[serde(default)]
    pub theme_index: usize,
}

/// Editor configuration - manages IDE integration
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct EditorConfig {
    #[serde(default)]
    pub ide: Ide,
}

// ========== DEFAULT VALUES ==========

fn default_retention_days() -> u64 {
    30
}
fn default_true() -> bool {
    true
}
fn default_max_file_size() -> u64 {
    10
}

// ========== MAIN CONFIG STRUCTURE ==========

/// Main configuration with categorized sections
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Config {
    #[serde(default)]
    pub storage: StorageConfig,
    #[serde(default)]
    pub ui: UiConfig,
    #[serde(default)]
    pub editor: EditorConfig,
}

// ========== LEGACY CONFIG STRUCTURE (for migration) ==========

/// Legacy flat configuration structure (for migration)
#[derive(Debug, Serialize, Deserialize, Clone)]
struct LegacyConfig {
    #[serde(default)]
    retention_days: Option<u64>,
    #[serde(default)]
    compression_enabled: Option<bool>,
    #[serde(default)]
    use_mnemosyneignore: Option<bool>,
    #[serde(default)]
    theme_index: Option<usize>,
    #[serde(default)]
    max_file_size_mb: Option<u64>,
    #[serde(default)]
    ide: Option<String>,
}

impl From<LegacyConfig> for Config {
    fn from(legacy: LegacyConfig) -> Self {
        let mut config = Config::default();

        if let Some(val) = legacy.retention_days {
            config.storage.retention_days = val;
        }
        if let Some(val) = legacy.compression_enabled {
            config.storage.compression_enabled = val;
        }
        if let Some(val) = legacy.use_mnemosyneignore {
            config.storage.use_mnemosyneignore = val;
        }
        if let Some(val) = legacy.max_file_size_mb {
            config.storage.max_file_size_mb = val;
        }
        if let Some(val) = legacy.theme_index {
            config.ui.theme_index = val;
        }
        if let Some(ref val) = legacy.ide
            && let Some(ide) = Ide::from_str_opt(val)
        {
            config.editor.ide = ide;
        }

        config
    }
}

// ========== CONFIG MANAGER ==========

pub struct ConfigManager {
    pub config_path: PathBuf,
    pub config: Config,
}

impl ConfigManager {
    pub fn new(base_dir: &std::path::Path) -> AppResult<Self> {
        let config_path = base_dir.join("config.toml");

        let config: Config = if config_path.exists() {
            let content = std::fs::read_to_string(&config_path).unwrap_or_default();

            // Try to detect if it's legacy flat structure first
            if content.contains("retention_days =") && !content.contains("[storage]") {
                if let Ok(legacy) = toml::from_str::<LegacyConfig>(&content) {
                    let new_config: Config = legacy.into();
                    if let Err(e) = Self::save_to_path(&new_config, &config_path) {
                        eprintln!("Warning: failed to save migrated config: {}", e);
                    }
                    new_config
                } else {
                    Config::default()
                }
            } else {
                toml::from_str(&content).unwrap_or_default()
            }
        } else {
            Config::default()
        };

        // Auto-save default if missing
        if !config_path.exists()
            && let Err(e) = Self::save_to_path(&config, &config_path)
        {
            eprintln!("Warning: failed to save default config: {}", e);
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

    pub fn get_value(&self, key: &str) -> Option<String> {
        match key {
            "storage.retention_days" => Some(self.config.storage.retention_days.to_string()),
            "storage.compression_enabled" => {
                Some(self.config.storage.compression_enabled.to_string())
            }
            "storage.use_mnemosyneignore" => {
                Some(self.config.storage.use_mnemosyneignore.to_string())
            }
            "storage.max_file_size_mb" => Some(self.config.storage.max_file_size_mb.to_string()),
            "ui.theme_index" => Some(self.config.ui.theme_index.to_string()),
            "editor.ide" => Some(self.config.editor.ide.as_str().to_string()),
            _ => None,
        }
    }

    pub fn set_value(&mut self, key: &str, value: &str) -> AppResult<()> {
        match key {
            "storage.retention_days" => {
                self.config.storage.retention_days = value
                    .parse::<u64>()
                    .map_err(|e| AppError::Config(e.to_string()))?;
            }
            "storage.compression_enabled" => {
                self.config.storage.compression_enabled = value
                    .parse::<bool>()
                    .map_err(|e| AppError::Config(e.to_string()))?;
            }
            "storage.use_mnemosyneignore" => {
                self.config.storage.use_mnemosyneignore = value
                    .parse::<bool>()
                    .map_err(|e| AppError::Config(e.to_string()))?;
            }
            "storage.max_file_size_mb" => {
                self.config.storage.max_file_size_mb = value
                    .parse::<u64>()
                    .map_err(|e| AppError::Config(e.to_string()))?;
            }
            "ui.theme_index" => {
                self.config.ui.theme_index = value
                    .parse::<usize>()
                    .map_err(|e| AppError::Config(e.to_string()))?;
            }
            "editor.ide" => {
                self.config.editor.ide = Ide::from_str_opt(value)
                    .ok_or_else(|| AppError::Config("Invalid IDE".into()))?;
            }
            _ => return Err(AppError::Config(format!("Unknown config key: {}", key))),
        }
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

        assert!(dir.path().join("config.toml").exists());
        assert_eq!(config_manager.config.storage.retention_days, 30);
    }

    #[test]
    fn config_persists_across_loads() {
        let dir = TempDir::new().unwrap();
        {
            let mut config_manager = ConfigManager::new(dir.path()).unwrap();
            config_manager.config.storage.retention_days = 45;
            config_manager.save().unwrap();
        }

        let config_manager = ConfigManager::new(dir.path()).unwrap();
        assert_eq!(config_manager.config.storage.retention_days, 45);
    }

    #[test]
    fn test_toggle_compression() {
        let dir = TempDir::new().unwrap();
        let mut config_manager = ConfigManager::new(dir.path()).unwrap();

        config_manager.config.storage.compression_enabled = false;
        config_manager.save().unwrap();

        let config_manager2 = ConfigManager::new(dir.path()).unwrap();
        assert!(!config_manager2.config.storage.compression_enabled);
    }

    #[test]
    fn test_get_value_nested_keys() {
        let dir = TempDir::new().unwrap();
        let config_manager = ConfigManager::new(dir.path()).unwrap();

        assert_eq!(
            config_manager.get_value("storage.retention_days").unwrap(),
            "30"
        );
        assert_eq!(
            config_manager
                .get_value("storage.compression_enabled")
                .unwrap(),
            "true"
        );
        assert_eq!(config_manager.get_value("ui.theme_index").unwrap(), "0");
        assert_eq!(config_manager.get_value("editor.ide").unwrap(), "Zed");
    }

    #[test]
    fn test_set_value_nested_keys() {
        let dir = TempDir::new().unwrap();
        {
            let mut config_manager = ConfigManager::new(dir.path()).unwrap();
            config_manager
                .set_value("storage.retention_days", "60")
                .unwrap();
            config_manager
                .set_value("storage.compression_enabled", "false")
                .unwrap();
            config_manager.set_value("editor.ide", "VsCode").unwrap();
        }

        let config_manager = ConfigManager::new(dir.path()).unwrap();
        assert_eq!(config_manager.config.storage.retention_days, 60);
        assert!(!config_manager.config.storage.compression_enabled);
        assert_eq!(config_manager.config.editor.ide, Ide::VsCode);
    }

    #[test]
    fn test_legacy_migration() {
        let dir = TempDir::new().unwrap();
        let config_path = dir.path().join("config.toml");

        // Write legacy config
        let legacy_content = r#"
retention_days = 15
compression_enabled = false
use_mnemosyneignore = true
theme_index = 2
max_file_size_mb = 20
ide = "VsCode"
"#;
        std::fs::write(&config_path, legacy_content).unwrap();

        // Load and migrate
        let config_manager = ConfigManager::new(dir.path()).unwrap();

        // Verify migration
        assert_eq!(config_manager.config.storage.retention_days, 15);
        assert!(!config_manager.config.storage.compression_enabled);
        assert_eq!(config_manager.config.ui.theme_index, 2);
        assert_eq!(config_manager.config.editor.ide, Ide::VsCode);
    }
}
