use ignore::gitignore::GitignoreBuilder;
use mnem_core::{AppError, AppResult, Repository};
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use rayon::prelude::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

pub struct Monitor {
    root_path: PathBuf,
    repo: Arc<Repository>,
}

struct FileEvent {
    path: PathBuf,
}

impl Monitor {
    pub fn new(root_path: PathBuf, repo: Arc<Repository>) -> Self {
        Self { root_path, repo }
    }

    pub async fn start(&self) -> AppResult<()> {
        if let Err(e) = self.initial_scan() {
            eprintln!("Initial scan failed: {:?}", e);
        }

        let (tx, mut rx) = mpsc::channel::<FileEvent>(100);
        let root_clone = self.root_path.clone();

        // Notify Thread
        let (n_tx, n_rx) = std::sync::mpsc::channel();
        let mut watcher = RecommendedWatcher::new(n_tx, Config::default())
            .map_err(|e| AppError::Config(e.to_string()))?;

        watcher
            .watch(&root_clone, RecursiveMode::Recursive)
            .map_err(|e| AppError::Config(e.to_string()))?;

        tokio::task::spawn_blocking(move || {
            while let Ok(res) = n_rx.recv() {
                if let Ok(event) = res {
                    for path in event.paths {
                        let _ = tx.blocking_send(FileEvent { path });
                    }
                }
            }
        });

        // Event Loop
        let mut debouncers: HashMap<PathBuf, Instant> = HashMap::new();
        let mut interval = tokio::time::interval(Duration::from_millis(500));

        let gitignore = self.get_gitignore()?;
        let config_max_size = self.repo.config.lock()
            .unwrap_or_else(|p| p.into_inner())
            .config.max_file_size_mb;
        let max_file_size = config_max_size * 1024 * 1024;

        log::info!("Monitor loop started for {:?}", self.root_path);
        loop {
            tokio::select! {

                Some(event) = rx.recv() => {
                    log::info!("File event received: {:?}", event.path);
                    if !self.is_ignored(&event.path, Some(&gitignore)) {

                        // Cap memory usage under heavy load (audit 4.5)
                        if debouncers.len() > 10000 {
                            debouncers.clear(); // Safety valve: drop pending debounce to avoid OOM
                            eprintln!("Warning: Too many pending file events, flush triggered");
                        }
                        debouncers.insert(event.path, Instant::now() + Duration::from_secs(1));
                    }
                }
                _ = interval.tick() => {
                    let now = Instant::now();
                    let mut to_save = Vec::new();

                    debouncers.retain(|path, deadline| {
                        if now >= *deadline {
                            to_save.push(path.clone());
                            false
                        } else {
                            true
                        }
                    });

                    if !to_save.is_empty() {
                        // Parallel processing of changed files
                        to_save.par_iter().for_each(|path| {
                            self.process_file(path, max_file_size);
                        });
                    }
                }

            }
        }
    }

    fn process_file(&self, path: &Path, max_file_size: u64) {
        if !path.is_file() {
            return;
        }

        // Symlink protection: resolve and verify the file is inside the project root (audit 1.3)
        if let Ok(canonical) = path.canonicalize() {
            if let Ok(root_canonical) = self.root_path.canonicalize() {
                if !canonical.starts_with(&root_canonical) {
                    return; // Symlink pointing outside the project
                }
            }
        }

        // File size limit (audit 4.6)
        if let Ok(metadata) = path.metadata() {
            if metadata.len() > max_file_size {
                return; // Skip files exceeding size limit
            }
        }

        // Check binary content
        let is_binary = if let Ok(mut file) = std::fs::File::open(path) {
            let mut buffer = [0; 1024];
            if let Ok(n) = std::io::Read::read(&mut file, &mut buffer) {
                content_inspector::inspect(&buffer[..n]).is_binary()
            } else {
                false
            }
        } else {
            false
        };

        if !is_binary {
            log::info!("Processing file: {:?}", path);
            if let Err(e) = self.repo.save_snapshot_from_file(path) {
                log::error!("Failed to save {:?}: {:?}", path, e);
            }
        }

    }

    /// Parallel initial scan using rayon for large codebases (audit 2.4).
    /// Processes files in chunks with throttling to avoid saturating IO (audit 4.5).
    pub fn initial_scan(&self) -> AppResult<()> {
        let gitignore = self.get_gitignore()?;
        use ignore::WalkBuilder;
        let walker = WalkBuilder::new(&self.root_path)
            .hidden(true)
            .git_ignore(true)
            .build();

        let max_file_size = self
            .repo
            .config
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .config
            .max_file_size_mb
            * 1024
            * 1024;

        let entries: Vec<_> = walker
            .filter_map(|r| r.ok())
            .filter(|e| e.path().is_file() && !self.is_ignored(e.path(), Some(&gitignore)))
            .collect();

        // Process in chunks of 100 files with rayon parallelism (audit 4.5)
        for chunk in entries.chunks(100) {
            chunk.par_iter().for_each(|entry| {
                self.process_file(entry.path(), max_file_size);
            });
        }

        Ok(())
    }

    /// Path-based ignore check using path components instead of string contains (audit 5.5).
    fn is_ignored(&self, path: &Path, gitignore: Option<&ignore::gitignore::Gitignore>) -> bool {
        let relative = path.strip_prefix(&self.root_path).unwrap_or(path);

        // Use path components for exact matching (audit 5.5)
        let has_ignored_component = relative.components().any(|c| {
            let s = c.as_os_str().to_string_lossy();
            s == "target"
                || s == ".git"
                || s == "node_modules"
                || s == ".DS_Store"
                || s == ".mnemosyne"
        });

        if has_ignored_component {
            return true;
        }

        if let Some(gi) = gitignore {
            if gi.matched(relative, false).is_ignore() {
                return true;
            }
        }
        false
    }

    fn get_gitignore(&self) -> AppResult<ignore::gitignore::Gitignore> {
        let config = self
            .repo
            .config
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .config
            .clone();
        let mut builder = GitignoreBuilder::new(&self.root_path);

        // 1. Global Ignore (audit 1.2)
        if let Some(home) = dirs::home_dir() {
            let global_ignore = home.join(".mnemosyne").join(".mnemignore");
            if global_ignore.exists() {
                builder.add(global_ignore);
            }
        }

        // 2. Project-level Ignores
        if config.use_gitignore {
            if let Some(ignore_path) =
                Some(self.root_path.join(".gitignore")).filter(|p| p.exists())
            {
                builder.add(ignore_path);
            }
        }

        if config.use_mnemosyneignore {
            if let Some(ignore_path) =
                Some(self.root_path.join(".mnemosyneignore")).filter(|p| p.exists())
            {
                builder.add(ignore_path);
            }
        }

        Ok(builder
            .build()
            .map_err(|e| AppError::Config(format!("Gitignore build failed: {}", e)))?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn setup_monitor() -> (TempDir, Monitor, Arc<Repository>) {
        let dir = TempDir::new().unwrap();
        let base_dir = dir.path().join(".mnemosyne");
        let project_dir = dir.path().join("project");
        fs::create_dir_all(&project_dir).unwrap();

        let repo = Arc::new(Repository::open(base_dir, project_dir.clone()).unwrap());
        let monitor = Monitor::new(project_dir, repo.clone());
        (dir, monitor, repo)
    }

    #[test]
    fn test_is_ignored_standard_dirs() {
        let (_dir, monitor, _) = setup_monitor();

        let target = monitor.root_path.join("target/debug/app");
        assert!(monitor.is_ignored(&target, None));

        let git = monitor.root_path.join(".git/config");
        assert!(monitor.is_ignored(&git, None));

        let node_modules = monitor.root_path.join("node_modules/pkg/index.js");
        assert!(monitor.is_ignored(&node_modules, None));

        let src = monitor.root_path.join("src/main.rs");
        assert!(!monitor.is_ignored(&src, None));
    }

    #[test]
    #[cfg(unix)]
    fn test_symlink_protection() {
        let (dir, monitor, repo) = setup_monitor();
        let project_dir = monitor.root_path.clone();

        // File inside project
        let internal_file = project_dir.join("internal.txt");
        fs::write(&internal_file, "internal").unwrap();

        // File outside project
        let external_file = dir.path().join("external.txt");
        fs::write(&external_file, "external").unwrap();

        // Symlink pointing outside
        let symlink_path = project_dir.join("bad_link");
        std::os::unix::fs::symlink(&external_file, &symlink_path).unwrap();

        // process_file should skip it
        monitor.process_file(&symlink_path, 10 * 1024 * 1024);

        // Verify no snapshot was created for the external file
        let history = repo.get_history(&symlink_path.to_string_lossy()).unwrap();
        assert_eq!(history.len(), 0);
    }
}
