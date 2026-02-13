use crate::error::{AppError, AppResult};
use crate::models::Project;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

const TRACKED_FILENAME: &str = "tracked";

pub struct ProjectRegistry {
    registry_path: PathBuf,
    projects: HashMap<String, Project>,
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

        let blacklist = [
            "/", "/Users", "/Users/", "/var", "/tmp", "/etc", "/bin", "/sbin", "/usr",
        ];
        if blacklist.contains(&path_str.as_ref()) || path_buf.parent().is_none() {
            return Err(AppError::Internal(format!(
                "Path {:?} is protected and cannot be tracked",
                path
            )));
        }

        let mnem_folder = path.join(".mnemosyne");
        fs::create_dir_all(&mnem_folder).map_err(AppError::IoGeneric)?;

        let tracked_file = mnem_folder.join(TRACKED_FILENAME);

        let project_id = if tracked_file.exists() {
            Self::read_project_id_from_file(&tracked_file)?
        } else {
            Project::generate_id(path)
        };

        let existing_by_path = self.projects.values().find(|p| p.path == path_str.as_ref());

        let project = if let Some(existing) = existing_by_path {
            if existing.id != project_id {
                let mut updated = existing.clone();
                updated.id = project_id.clone();
                updated.last_open = chrono::Local::now().to_rfc3339();
                let existing_id = existing.id.clone();
                self.projects.remove(&existing_id);
                self.projects.insert(project_id.clone(), updated.clone());
                self.update_tracked_file(&tracked_file, &updated)?;
                self.save()?;
                updated
            } else {
                let mut p = existing.clone();
                p.last_open = chrono::Local::now().to_rfc3339();
                self.projects.insert(project_id.clone(), p.clone());
                self.update_tracked_file(&tracked_file, &p)?;
                self.save()?;
                p
            }
        } else {
            let new_project = Project::from_id(&project_id, path);
            if new_project.name.is_empty() || new_project.name == "unknown" {
                return Err(AppError::Internal(
                    "Cannot determine a valid project name for this path".into(),
                ));
            }
            self.update_tracked_file(&tracked_file, &new_project)?;
            self.projects
                .insert(project_id.clone(), new_project.clone());
            self.save()?;
            new_project
        };

        self.projects
            .get(&project.id)
            .cloned()
            .ok_or_else(|| AppError::Internal("Failed to retrieve project".to_string()))
    }

    fn read_project_id_from_file(file: &Path) -> AppResult<String> {
        let content = fs::read_to_string(file).map_err(AppError::IoGeneric)?;
        for line in content.lines() {
            if line.starts_with("project_id:") {
                return Ok(line.split(':').nth(1).unwrap_or("").trim().to_string());
            }
        }
        Err(AppError::Internal(
            "No project_id found in tracked file".to_string(),
        ))
    }

    fn update_tracked_file(&self, file: &Path, project: &Project) -> AppResult<()> {
        let content = format!(
            "tracked by mnemosyne\nproject_id: {}\nproject_name: {}\npath: {}\nlast_open: {}\n",
            project.id, project.name, project.path, project.last_open
        );
        fs::write(file, content).map_err(AppError::IoGeneric)?;
        Ok(())
    }

    pub fn find_by_id(&self, id: &str) -> Option<Project> {
        self.projects.get(id).cloned()
    }

    pub fn find_by_path(&self, path: &Path) -> Option<Project> {
        let path_str = path.to_string_lossy();
        self.projects
            .values()
            .find(|p| p.path == path_str.as_ref())
            .cloned()
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

        // Filter out projects that no longer exist or don't have .mnemosyne folder
        list.retain(|p| {
            let project_path = Path::new(&p.path);
            if !project_path.exists() {
                return false;
            }
            // Check for .mnemosyne folder marker in project directory
            let mnem_folder = project_path.join(".mnemosyne");
            mnem_folder.exists() && mnem_folder.is_dir()
        });

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

    pub fn update(&mut self, project: Project) -> AppResult<()> {
        self.projects.insert(project.id.clone(), project);
        self.save()?;
        Ok(())
    }
}
