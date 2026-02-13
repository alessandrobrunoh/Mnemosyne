use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,        // Unique ID (hashed path or UUID)
    pub path: String,      // Absolute path to the project root
    pub name: String,      // Human and TUI friendly name (folder name)
    pub last_open: String, // ISO 8601 timestamp
}

impl Project {
    pub fn new(path: &std::path::Path) -> Self {
        let path_str = path.to_string_lossy().to_string();
        let id = Self::generate_id(path);
        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        Self {
            id,
            path: path_str,
            name,
            last_open: chrono::Local::now().to_rfc3339(),
        }
    }

    pub fn generate_id(path: &std::path::Path) -> String {
        let path_str = path.to_string_lossy().to_string();
        blake3::hash(path_str.as_bytes()).to_hex().to_string()[..16].to_string()
    }

    pub fn from_id(id: &str, path: &std::path::Path) -> Self {
        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        Self {
            id: id.to_string(),
            path: path.to_string_lossy().to_string(),
            name,
            last_open: chrono::Local::now().to_rfc3339(),
        }
    }
}
