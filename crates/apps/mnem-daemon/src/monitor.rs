use ignore::gitignore::GitignoreBuilder;
use lru::LruCache;
use mnem_core::{AppError, AppResult, Repository};
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use rayon::prelude::*;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

const DEBOUNCER_CACHE_SIZE: NonZeroUsize = NonZeroUsize::new(5000).unwrap();

pub struct Monitor {
    root_path: PathBuf,
    repo: Arc<Repository>,
    state: Option<Arc<crate::state::DaemonState>>,
}

struct FileEvent {
    path: PathBuf,
}

impl Monitor {
    pub fn new(root_path: PathBuf, repo: Arc<Repository>) -> Self {
        Self { root_path, repo, state: None }
    }

    pub fn with_state(root_path: PathBuf, repo: Arc<Repository>, state: Arc<crate::state::DaemonState>) -> Self {
        Self { root_path, repo, state: Some(state) }
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
        let mut debouncers: LruCache<PathBuf, Instant> = LruCache::new(DEBOUNCER_CACHE_SIZE);
        let mut interval = tokio::time::interval(Duration::from_millis(500));

        let mnemignore = self.get_mnemignore()?;
        let config_max_size = self.repo.config.lock()
            .unwrap_or_else(|p| p.into_inner())
            .config.max_file_size_mb;
        let max_file_size = config_max_size * 1024 * 1024;

        log::info!("Monitor loop started for {:?}", self.root_path);
        loop {
            tokio::select! {

                Some(event) = rx.recv() => {
                    log::info!("File event received: {:?}", event.path);
                    if !self.is_ignored(&event.path, Some(&mnemignore)) {
                        // LRU cache automatically evicts oldest entry when full
                        debouncers.push(event.path, Instant::now() + Duration::from_secs(1));
                    }
                }
                _ = interval.tick() => {
                    let now = Instant::now();
                    let mut to_save = Vec::new();

                    // Collect expired entries and remove them from cache
                    let keys_to_remove: Vec<PathBuf> = debouncers
                        .iter()
                        .filter(|(_, deadline)| now >= **deadline)
                        .map(|(path, _)| path.clone())
                        .collect();

                    // Remove expired entries and collect paths to save
                    for path in keys_to_remove {
                        if debouncers.pop(&path).is_some() {
                            to_save.push(path);
                        }
                    }

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

        log::info!("File {:?} is_binary: {}", path, is_binary);

        if !is_binary {
            log::info!("Processing file: {:?}", path);
            let start = Instant::now();
            match self.repo.save_snapshot_from_file(path) {
                Ok(hash) => {
                    let duration = start.elapsed().as_micros() as u64;
                    if let Some(ref state) = self.state {
                        state.record_save(duration);
                        // Invalidate history cache for this file
                        state.invalidate_history_cache(Some(&path.to_string_lossy()));
                    }
                    log::info!("Saved file {:?} with hash {}", path, &hash[..8]);
                }
                Err(e) => log::error!("Failed to save {:?}: {:?}", path, e),
            }
        } else {
            log::info!("Skipping binary file: {:?}", path);
        }

    }

    /// Parallel initial scan using rayon for large codebases (audit 2.4).
    /// Processes files in chunks with throttling to avoid saturating IO (audit 4.5).
    pub fn initial_scan(&self) -> AppResult<()> {
        let mnemignore = self.get_mnemignore()?;
        use ignore::WalkBuilder;
        let walker = WalkBuilder::new(&self.root_path)
            .hidden(true)
            .git_ignore(false)
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
            .filter(|e| e.path().is_file() && !self.is_ignored(e.path(), Some(&mnemignore)))
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
    fn is_ignored(&self, path: &Path, mnemignore: Option<&ignore::gitignore::Gitignore>) -> bool {
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

        if let Some(mi) = mnemignore {
            if mi.matched(relative, false).is_ignore() {
                return true;
            }
        }
        false
    }

    fn get_mnemignore(&self) -> AppResult<ignore::gitignore::Gitignore> {
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

        // 2. Project-level .mnemignore
        if config.use_mnemosyneignore {
            if let Some(ignore_path) =
                Some(self.root_path.join(".mnemosyneignore")).filter(|p| p.exists())
            {
                builder.add(ignore_path);
            }
        }

        Ok(builder
            .build()
            .map_err(|e| AppError::Config(format!("Mnemignore build failed: {}", e)))?)
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
