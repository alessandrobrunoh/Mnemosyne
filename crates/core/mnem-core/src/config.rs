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

// ========== CATEGORIZED CONFIG STRUCTURES ==========

/// Storage configuration - manages memory, disk, and data retention
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StorageConfig {
    #[serde(default = "default_retention_days")]
    pub retention_days: u64,
    #[serde(default = "default_compression_enabled")]
    pub compression_enabled: bool,
    #[serde(default = "default_use_mnemosyneignore")]
    pub use_mnemosyneignore: bool,
    #[serde(default = "default_max_file_size_mb")]
    pub max_file_size_mb: u64,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            retention_days: default_retention_days(),
            compression_enabled: default_compression_enabled(),
            use_mnemosyneignore: default_use_mnemosyneignore(),
            max_file_size_mb: default_max_file_size_mb(),
        }
    }
}

/// UI configuration - manages visual settings and themes
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UiConfig {
    #[serde(default)]
    pub theme_index: usize,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self { theme_index: 0 }
    }
}

/// Editor configuration - manages IDE integration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EditorConfig {
    #[serde(default)]
    pub ide: Ide,
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            ide: Ide::default(),
        }
    }
}

// ========== DEFAULT VALUES ==========

fn default_retention_days() -> u64 {
    30
}

fn default_compression_enabled() -> bool {
    true
}

fn default_use_mnemosyneignore() -> bool {
    true
}

fn default_max_file_size_mb() -> u64 {
    10
}

// ========== MAIN CONFIG STRUCTURE ==========

/// Main configuration with categorized sections
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    #[serde(default)]
    pub storage: StorageConfig,
    #[serde(default)]
    pub ui: UiConfig,
    #[serde(default)]
    pub editor: EditorConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            storage: StorageConfig::default(),
            ui: UiConfig::default(),
            editor: EditorConfig::default(),
        }
    }
}

// ========== LEGACY CONFIG STRUCTURE (for migration) ==========

/// Legacy flat configuration structure (for migration)
#[derive(Debug, Serialize, Deserialize, Clone)]
struct LegacyConfig {
    pub retention_days: u64,
    pub compression_enabled: bool,
    pub use_mnemosyneignore: bool,
    pub theme_index: usize,
    pub max_file_size_mb: u64,
    pub ide: Ide,
}

impl From<LegacyConfig> for Config {
    fn from(legacy: LegacyConfig) -> Self {
        Self {
            storage: StorageConfig {
                retention_days: legacy.retention_days,
                compression_enabled: legacy.compression_enabled,
                use_mnemosyneignore: legacy.use_mnemosyneignore,
                max_file_size_mb: legacy.max_file_size_mb,
            },
            ui: UiConfig {
                theme_index: legacy.theme_index,
            },
            editor: EditorConfig { ide: legacy.ide },
        }
    }
}

// ========== CONFIG MANAGER ==========

pub struct ConfigManager {
    config_path: PathBuf,
    pub config: Config,
}

impl ConfigManager {
    pub fn new(base_dir: &std::path::Path) -> AppResult<Self> {
        let config_path = base_dir.join("config.toml");

        let config = if config_path.exists() {
            let content = std::fs::read_to_string(&config_path).map_err(AppError::IoGeneric)?;

            // Try new categorized structure first
            let new_result: Result<Config, _> = toml::from_str(&content);

            if let Ok(cfg) = new_result {
                // Successfully loaded new structure
                cfg
            } else {
                // Try legacy flat structure
                let legacy_result: Result<LegacyConfig, _> = toml::from_str(&content);

                if let Ok(legacy) = legacy_result {
                    // Migrate from legacy to new structure
                    let new_config: Config = legacy.into();
                    // Save migrated config
                    if let Err(e) = Self::save_to_path(&new_config, &config_path) {
                        eprintln!("Warning: failed to save migrated config: {}", e);
                    }
                    new_config
                } else {
                    // Both failed, use defaults
                    eprintln!("Warning: failed to parse config, using defaults");
                    Config::default()
                }
            }
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

    // ========== NESTED KEY GET/SET METHODS ==========

    /// Get a config value by nested key (e.g., "storage.retention_days")
    pub fn get_value(&self, key: &str) -> AppResult<String> {
        let parts: Vec<&str> = key.split('.').collect();

        match parts.as_slice() {
            ["storage", "retention_days"] => Ok(self.config.storage.retention_days.to_string()),
            ["storage", "compression_enabled"] => {
                Ok(self.config.storage.compression_enabled.to_string())
            }
            ["storage", "use_mnemosyneignore"] => {
                Ok(self.config.storage.use_mnemosyneignore.to_string())
            }
            ["storage", "max_file_size_mb"] => Ok(self.config.storage.max_file_size_mb.to_string()),
            ["ui", "theme_index"] => Ok(self.config.ui.theme_index.to_string()),
            ["editor", "ide"] => Ok(self.config.editor.ide.as_str().to_string()),
            _ => Err(AppError::Config(format!("Unknown config key: {}", key))),
        }
    }

    /// Set a config value by nested key (e.g., "storage.retention_days")
    pub fn set_value(&mut self, key: &str, value: &str) -> AppResult<()> {
        let parts: Vec<&str> = key.split('.').collect();

        match parts.as_slice() {
            ["storage", "retention_days"] => {
                let days = value.parse::<u64>().map_err(|_| {
                    AppError::Config(format!("Invalid retention_days value: {}", value))
                })?;
                self.config.storage.retention_days = days;
            }
            ["storage", "compression_enabled"] => {
                let enabled = value.parse::<bool>().map_err(|_| {
                    AppError::Config(format!("Invalid compression_enabled value: {}", value))
                })?;
                self.config.storage.compression_enabled = enabled;
            }
            ["storage", "use_mnemosyneignore"] => {
                let enabled = value.parse::<bool>().map_err(|_| {
                    AppError::Config(format!("Invalid use_mnemosyneignore value: {}", value))
                })?;
                self.config.storage.use_mnemosyneignore = enabled;
            }
            ["storage", "max_file_size_mb"] => {
                let size = value.parse::<u64>().map_err(|_| {
                    AppError::Config(format!("Invalid max_file_size_mb value: {}", value))
                })?;
                self.config.storage.max_file_size_mb = size;
            }
            ["ui", "theme_index"] => {
                let index = value.parse::<usize>().map_err(|_| {
                    AppError::Config(format!("Invalid theme_index value: {}", value))
                })?;
                self.config.ui.theme_index = index;
            }
            ["editor", "ide"] => {
                let ide = match value.to_lowercase().as_str() {
                    "zed" => Ide::Zed,
                    "zed preview" | "zedpreview" => Ide::ZedPreview,
                    "vscode" | "visual studio code" => Ide::VsCode,
                    _ => return Err(AppError::Config(format!("Invalid IDE value: {}", value))),
                };
                self.config.editor.ide = ide;
            }
            _ => return Err(AppError::Config(format!("Unknown config key: {}", key))),
        }

        self.save()
    }

    // ========== CONVENIENCE UPDATE METHODS ==========

    pub fn update_storage_retention(&mut self, days: u64) -> AppResult<()> {
        self.config.storage.retention_days = days;
        self.save()
    }

    pub fn update_storage_compression(&mut self, enabled: bool) -> AppResult<()> {
        self.config.storage.compression_enabled = enabled;
        self.save()
    }

    pub fn update_storage_use_ignore(&mut self, enabled: bool) -> AppResult<()> {
        self.config.storage.use_mnemosyneignore = enabled;
        self.save()
    }

    pub fn update_storage_max_file_size(&mut self, size_mb: u64) -> AppResult<()> {
        self.config.storage.max_file_size_mb = size_mb;
        self.save()
    }

    pub fn update_ui_theme(&mut self, index: usize) -> AppResult<()> {
        self.config.ui.theme_index = index;
        self.save()
    }

    pub fn update_editor_ide(&mut self, ide: Ide) -> AppResult<()> {
        self.config.editor.ide = ide;
        self.save()
    }

    // ========== LEGACY METHODS (for backwards compatibility) ==========

    pub fn update_retention(&mut self, days: u64) -> AppResult<()> {
        self.update_storage_retention(days)
    }

    pub fn toggle_compression(&mut self) -> AppResult<()> {
        self.config.storage.compression_enabled = !self.config.storage.compression_enabled;
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
        assert_eq!(config_manager.config.storage.retention_days, 30);
        assert!(dir.path().join("config.toml").exists());
    }

    #[test]
    fn config_persists_across_loads() {
        let dir = TempDir::new().unwrap();
        {
            let mut config_manager = ConfigManager::new(dir.path()).unwrap();
            config_manager.update_storage_retention(7).unwrap();
        }
        let config_manager = ConfigManager::new(dir.path()).unwrap();
        assert_eq!(config_manager.config.storage.retention_days, 7);
    }

    #[test]
    fn test_toggle_compression() {
        let dir = TempDir::new().unwrap();
        let mut config_manager = ConfigManager::new(dir.path()).unwrap();
        let initial = config_manager.config.storage.compression_enabled;
        config_manager.toggle_compression().unwrap();
        assert_eq!(config_manager.config.storage.compression_enabled, !initial);
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
        assert_eq!(config_manager.config.storage.compression_enabled, false);
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
        assert_eq!(config_manager.config.storage.compression_enabled, false);
        assert_eq!(config_manager.config.ui.theme_index, 2);
        assert_eq!(config_manager.config.editor.ide, Ide::VsCode);
    }
}
