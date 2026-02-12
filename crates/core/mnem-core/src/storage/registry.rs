use crate::error::{AppError, AppResult};
use crate::models::Project;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Manages the list of known projects in `~/.mnemosyne/registry.json`.
pub struct ProjectRegistry {
    registry_path: PathBuf,
    projects: HashMap<String, Project>, // Key is project.id
}

impl ProjectRegistry {
    pub fn new(base_dir: &Path) -> AppResult<Self> {
        let registry_path = base_dir.join("registry.json");
        let mut projects: HashMap<String, Project> = if registry_path.exists() {
            let content = fs::read_to_string(&registry_path).map_err(AppError::IoGeneric)?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            HashMap::new()
        };

        // Auto-purge blacklisted or invalid paths (e.g. root)
        let blacklist = [
            "/", "/Users", "/Users/", "/var", "/tmp", "/etc", "/bin", "/sbin", "/usr",
        ];
        projects.retain(|_, p| {
            !blacklist.contains(&p.path.as_str()) && !p.name.is_empty() && p.name != "unknown"
        });

        Ok(Self {
            registry_path,
            projects,
        })
    }

    pub fn get_or_create(&mut self, path: &Path) -> AppResult<Project> {
        let path_str = path.to_string_lossy();
        let path_buf = path.to_path_buf();

        // 1. Rigorous blacklist (audit 1.2)
        let blacklist = [
            "/", "/Users", "/Users/", "/var", "/tmp", "/etc", "/bin", "/sbin", "/usr",
        ];
        if blacklist.contains(&path_str.as_ref()) || path_buf.parent().is_none() {
            return Err(AppError::Internal(format!(
                "Path {:?} is protected and cannot be tracked",
                path
            )));
        }

        let new_project = Project::new(path);
        if new_project.name.is_empty() || new_project.name == "unknown" {
            return Err(AppError::Internal(
                "Cannot determine a valid project name for this path".into(),
            ));
        }

        if let Some(existing) = self.projects.get_mut(&new_project.id) {
            existing.last_open = chrono::Local::now().to_rfc3339();
        } else {
            self.projects
                .insert(new_project.id.clone(), new_project.clone());
        }

        self.save()?;
        self.projects.get(&new_project.id).cloned().ok_or_else(|| {
            AppError::Internal("Failed to retrieve newly created project".to_string())
        })
    }

    fn save(&self) -> AppResult<()> {
        let content = serde_json::to_string_pretty(&self.projects)
            .map_err(|e| AppError::Internal(format!("Failed to serialize registry: {}", e)))?;

        // Atomic write
        let parent = self.registry_path.parent().unwrap_or(Path::new("."));
        let temp = tempfile::NamedTempFile::new_in(parent).map_err(AppError::IoGeneric)?;
        fs::write(temp.path(), &content).map_err(AppError::IoGeneric)?;
        temp.persist(&self.registry_path)
            .map_err(|e| AppError::IoGeneric(e.error))?;

        Ok(())
    }

    pub fn list_projects(&self) -> Vec<Project> {
        let mut list: Vec<Project> = self.projects.values().cloned().collect();
        // Sort by last_open desc
        list.sort_by(|a, b| b.last_open.cmp(&a.last_open));
        list
    }

    pub fn remove(&mut self, id: &str) -> AppResult<Option<Project>> {
        let removed = self.projects.remove(id);
        if removed.is_some() {
            self.save()?;
        }
        Ok(removed)
    }
}
