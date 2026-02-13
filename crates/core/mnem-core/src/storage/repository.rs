use crate::config::ConfigManager;
use crate::error::{AppError, AppResult};
use crate::models::{FileEntry, Project, SearchResult, Session, Snapshot};
use crate::semantic::diff::SemanticDiffer;
use crate::semantic::SemanticParser;
use crate::storage::registry::ProjectRegistry;
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use super::database::Database;
use super::fs::CasStorage;

pub struct Repository {
    pub db: Database,
    pub fs: CasStorage,
    pub config: Mutex<ConfigManager>,
    pub project: Project,
}

impl Repository {
    /// Get compression enabled config
    pub fn is_compression_enabled(&self) -> bool {
        self.config
            .lock()
            .map(|c| c.config.compression_enabled)
            .unwrap_or(true)
    }
}

impl Repository {
    /// Initialize the repository for the current project.
    /// Detects the project root by looking for `.git`, falling back to CWD.
    /// Uses `~/.mnemosyne` as the global storage root.
    pub fn init() -> AppResult<Self> {
        let home = dirs::home_dir().ok_or_else(|| AppError::Config("Home dir not found".into()))?;
        let cwd = std::env::current_dir().map_err(AppError::IoGeneric)?;
        let root = Self::find_project_root(&cwd);
        Self::open(home.join(".mnemosyne"), root)
    }

    fn find_project_root(path: &Path) -> PathBuf {
        let mut current = path;
        loop {
            if current.join(".git").exists() {
                return current.to_path_buf();
            }
            if let Some(parent) = current.parent() {
                current = parent;
            } else {
                break;
            }
        }
        path.to_path_buf()
    }

    /// Open a repository for a specific project path.
    /// Storage is in `<project_path>/.mnemosyne/` (per-project).
    /// Registry is in `~/.mnemosyne/registry.json` (global).
    pub fn open(base_dir: PathBuf, project_path: PathBuf) -> AppResult<Self> {
        let project_mnem_dir = project_path.join(".mnemosyne");

        if !project_mnem_dir.exists() {
            std::fs::create_dir_all(&project_mnem_dir).map_err(|e| AppError::Io {
                path: project_mnem_dir.clone(),
                source: e,
            })?;
        }

        let tracked_file = project_mnem_dir.join("tracked");
        let project_id = if tracked_file.exists() {
            let content = std::fs::read_to_string(&tracked_file).map_err(AppError::IoGeneric)?;
            content
                .lines()
                .find(|l| l.starts_with("project_id:"))
                .and_then(|l| l.split(':').nth(1))
                .map(|s| s.trim().to_string())
                .ok_or_else(|| AppError::Internal("No project_id in tracked file".to_string()))?
        } else {
            return Err(AppError::Internal(
                "Project not tracked. Run 'mnem track' first.".to_string(),
            ));
        };

        let mut registry = ProjectRegistry::new(&base_dir)?;
        let project = if let Some(existing) = registry.find_by_id(&project_id) {
            let mut p = existing;
            p.path = project_path.to_string_lossy().to_string();
            p.last_open = chrono::Local::now().to_rfc3339();
            registry.update(p.clone())?;
            p
        } else {
            let new_project = crate::models::Project::from_id(&project_id, &project_path);
            registry.update(new_project.clone())?;
            new_project
        };

        let db_dir = project_mnem_dir.join("db");
        if !db_dir.exists() {
            std::fs::create_dir_all(&db_dir).map_err(|e| AppError::Io {
                path: db_dir.clone(),
                source: e,
            })?;
        }

        let db_path = db_dir.join("mnemosyne.db");
        let db = Database::new(db_path)?;

        let cas_dir = project_mnem_dir.join("cas");
        if !cas_dir.exists() {
            std::fs::create_dir_all(&cas_dir).map_err(|e| AppError::Io {
                path: cas_dir.clone(),
                source: e,
            })?;
        }
        let fs = CasStorage::new(cas_dir)?;

        let config = ConfigManager::new(&base_dir)?;

        let ignore_path = project_path.join(".mnemignore");
        if !ignore_path.exists() {
            let default_ignore = r#"# Mnemosyne Ignore File
# Standard exclusions for development projects

# Build directories
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
bower_components/

# IDEs and Editors
.idea/
.vscode/
*.swp
*.swo
.DS_Store
Thumbs.db

# Version Control
.git/
.svn/
.hg/

# Mnemosyne Internal
.mnemosyne/
logs/
mnemd.pid

# Temporary files
tmp/
temp/
*.log
*.tmp
*.bak
"#;
            let _ = std::fs::write(&ignore_path, default_ignore);
        }

        Ok(Self {
            db,
            fs,
            config: Mutex::new(config),
            project,
        })
    }

    /// Try to find which project a snapshot hash belongs to by searching all registered projects.
    pub fn find_by_hash(hash: &str) -> AppResult<Self> {
        let home = dirs::home_dir().ok_or_else(|| AppError::Config("Home dir not found".into()))?;
        let base_dir = home.join(".mnemosyne");
        let registry = ProjectRegistry::new(&base_dir)?;

        for project in registry.list_projects() {
            let repo = Self::open(base_dir.clone(), project.path.into())?;
            if let Ok(Some(_)) = repo.db.resolve_hash(hash) {
                return Ok(repo);
            }
        }

        Err(AppError::Internal(format!(
            "Snapshot hash {} not found in any project",
            hash
        )))
    }

    /// Calculate the size of the project's history in bytes.
    /// Sum of all unique chunks linked to this project + SQLite DB size (including WAL/SHM).
    pub fn get_project_size(&self) -> AppResult<u64> {
        let mut total = 0;

        // 1. Get size of all unique chunks
        let hashes = self.db.get_all_content_hashes()?;
        for hash in hashes {
            if let Ok(size) = self.fs.get_size(&hash) {
                total += size;
            }
        }

        // 2. Add SQLite DB size (including WAL and SHM files)
        let paths = [
            self.db.path.clone(),
            self.db.path.with_extension("sqlite-wal"),
            self.db.path.with_extension("sqlite-shm"),
        ];

        for path in paths {
            if let Ok(metadata) = std::fs::metadata(path) {
                total += metadata.len();
            }
        }

        Ok(total)
    }

    /// Calculate total storage size of ~/.mnemosyne.
    pub fn get_total_storage_size() -> AppResult<u64> {
        let home = dirs::home_dir().ok_or_else(|| AppError::Config("Home dir not found".into()))?;
        let base_dir = home.join(".mnemosyne");
        Ok(Self::dir_size(&base_dir).unwrap_or(0))
    }

    fn dir_size(path: &Path) -> std::io::Result<u64> {
        let mut size = 0;
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let metadata = entry.metadata()?;
            if metadata.is_dir() {
                size += Self::dir_size(&entry.path())?;
            } else {
                size += metadata.len();
            }
        }
        Ok(size)
    }

    /// Legacy constructor for testing. Creates a standalone env with a "default" project.
    pub fn with_base_dir(base_dir: PathBuf) -> AppResult<Self> {
        // Simulate a project inside this temp dir
        Self::open(base_dir.clone(), base_dir)
    }

    /// Garbage collection: prune old snapshots AND clean orphan object files (audit 3.2).
    pub fn run_gc(&self) -> AppResult<usize> {
        let retention = self
            .config
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .config
            .retention_days;
        if retention == 0 {
            return Ok(0);
        }

        // 1. Get all hashes referenced BEFORE pruning
        let hashes_before = self.db.get_all_content_hashes()?;

        // 2. Prune from DB
        let pruned = self.db.prune_snapshots(retention)?;

        // 3. Get hashes still referenced AFTER pruning
        let hashes_after = self.db.get_all_content_hashes()?;

        // 4. Delete orphan object files (present before but not after)
        let orphaned: Vec<_> = hashes_before.difference(&hashes_after).collect();
        for hash in &orphaned {
            if let Err(e) = self.fs.delete(hash) {
                eprintln!("Warning: failed to delete orphan object {}: {}", hash, e);
            }
        }

        // 5. Clean stale temp files (audit hygiene)
        let _ = self.fs.clean_temp();

        // 6. VACUUM to reclaim space
        if pruned > 0 {
            if let Err(e) = self.db.vacuum() {
                eprintln!("Warning: VACUUM failed: {}", e);
            }
        }

        Ok(pruned)
    }

    /// Clear all history for the current project.
    pub fn clear_all_history(&self) -> AppResult<usize> {
        // 1. Get all hashes before wipe
        let hashes_before = self.db.get_all_content_hashes()?;

        // 2. Wipe DB
        let count = self.db.delete_all()?;

        // 3. Clean all chunks from FS (since DB is now empty, all are orphans)
        for hash in &hashes_before {
            let _ = self.fs.delete(hash);
        }

        // 4. Compact DB
        let _ = self.db.vacuum();

        Ok(count)
    }

    /// Trigger background migration of tiered storage objects (Hot -> Warm -> Cold)
    pub fn run_migration(&self) -> AppResult<usize> {
        Ok(0)
    }

    /// Save a snapshot using the dedup-first pattern (audit 5.4):
    /// Uses a single read pass to hash and compress atomically,
    /// preventing TOCTOU race between compute_hash and write_stream.
    pub fn save_snapshot_from_file(&self, file_path: &Path) -> AppResult<String> {
        let file = std::fs::File::open(file_path).map_err(|e| AppError::Io {
            path: file_path.to_path_buf(),
            source: e,
        })?;

        // Memory Mapping for true Zero-Copy disk access
        let mmap = unsafe {
            memmap2::MmapOptions::new()
                .map(&file)
                .map_err(|e| AppError::Io {
                    path: file_path.to_path_buf(),
                    source: e,
                })?
        };

        // Wrap mmap in Bytes (sharing the memory-mapped buffer)
        let content = bytes::Bytes::copy_from_slice(&mmap);
        // Note: although copy_from_slice copies, it allows us to handle memory
        // in a unified way. For a "pure" zero-copy we could use a custom
        // wrapper, but Bytes::copy_from_slice is a good compromise for stability.

        self.save_snapshot(file_path, content)
    }

    pub fn save_snapshot(&self, file_path: &Path, content: bytes::Bytes) -> AppResult<String> {
        let path_str = file_path.to_string_lossy().to_string();
        let full_hash = blake3::hash(&content).to_hex().to_string();

        // 1. Dedup check before doing anything expensive
        let previous_snapshot_data = if let Ok(Some(last_hash)) = self.db.get_last_hash(&path_str) {
            if last_hash == full_hash {
                return Ok(full_hash);
            }

            // Get previous symbols BEFORE inserting new snapshot
            if let Ok(Some(last_snap)) = self
                .db
                .get_history_by_hash(&last_hash)
                .map(|h| h.into_iter().next())
            {
                let last_symbols = self
                    .db
                    .get_symbols_for_snapshot(last_snap.id)
                    .unwrap_or_default();
                Some((last_snap.id, last_symbols))
            } else {
                None
            }
        } else {
            None
        };

        // 2. Insert Snapshot and get ID
        let timestamp = chrono::Local::now().to_rfc3339();
        let branch = self.get_current_branch();

        // Ensure the full file content is stored in CAS for quick retrieval/preview (audit 5.4)
        let enable_compression = self.is_compression_enabled();
        self.fs.write(&content, enable_compression)?;

        let snapshot_id =
            self.db
                .insert_snapshot(&path_str, &timestamp, &full_hash, branch.as_deref(), None)?;

        // 3. Chunkify (SHP Protocol) with Semantic Awareness
        let ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let chunks = crate::semantic::chunker::SemanticChunker::chunk(content.clone(), ext);
        let mut chunks_info = Vec::new();

        for chunk in chunks {
            let chunk_hash = blake3::hash(&chunk.data).to_hex().to_string();
            self.fs.write(&chunk.data, enable_compression)?; // Filesystem CAS
            self.db.insert_chunk(&chunk_hash, &chunk.data, "raw")?; // DB metadata
            chunks_info.push((chunk_hash, chunk.offset, chunk.length));
        }

        // 4. Link Chunks to Snapshot
        for (pos, (hash, _, _)) in chunks_info.iter().enumerate() {
            self.db.link_snapshot_chunk(snapshot_id, hash, pos)?;
        }

        // 5. Semantic Indexing (Sync for now, should be background thread later)
        let ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if let Ok(mut parser) = SemanticParser::new() {
            if let Ok((symbols, references)) =
                parser.parse_semantic_data(&content, ext, snapshot_id, Some(&path_str))
            {
                let mut should_save_symbols = true;
                let mut prev_snapshot_id = None;
                let mut prev_symbols = Vec::new();

                if let Some((pid, psyms)) = previous_snapshot_data {
                    prev_snapshot_id = Some(pid);
                    prev_symbols = psyms;

                    // Calculate a "Structural Signature" of the file
                    let current_sig: String = symbols
                        .iter()
                        .map(|s| &s.structural_hash)
                        .cloned()
                        .collect::<Vec<_>>()
                        .join("");
                    let last_sig: String = prev_symbols
                        .iter()
                        .map(|s| &s.structural_hash)
                        .cloned()
                        .collect::<Vec<_>>()
                        .join("");

                    if current_sig == last_sig && !current_sig.is_empty() {
                        should_save_symbols = false;
                    }
                }

                // Compute and store Semantic Deltas
                if prev_snapshot_id.is_some() || !symbols.is_empty() {
                    let deltas = SemanticDiffer::compare(
                        &prev_symbols,
                        &symbols,
                        prev_snapshot_id,
                        snapshot_id,
                    );

                    for delta in deltas {
                        let _ = self.db.insert_symbol_delta(&delta);
                    }
                }

                if should_save_symbols {
                    // Stack of (symbol_end_byte, db_id)
                    let mut parent_stack: Vec<(usize, i64)> = Vec::new();

                    for mut symbol in symbols {
                        // Pop parents that don't contain this symbol
                        while let Some((parent_end, _)) = parent_stack.last() {
                            if symbol.start_byte >= *parent_end {
                                parent_stack.pop();
                            } else {
                                break;
                            }
                        }

                        // Set parent_id if applicable
                        if let Some((_, parent_db_id)) = parent_stack.last() {
                            symbol.parent_id = Some(*parent_db_id);
                        }

                        // Find which chunk contains the start of this symbol
                        if let Some((chunk_hash, _, _)) =
                            chunks_info.iter().find(|(_, offset, len)| {
                                symbol.start_byte >= *offset && symbol.start_byte < (*offset + *len)
                            })
                        {
                            symbol.chunk_hash = chunk_hash.clone();
                        }

                        // Insert and get ID
                        if let Ok(db_id) = self.db.insert_symbol(&symbol) {
                            // Push this symbol onto stack as a potential parent
                            parent_stack.push((symbol.end_byte, db_id));
                        }
                    }
                }

                // Save references
                for reference in references {
                    let _ = self.db.insert_reference(&reference);
                }
            }
        }

        Ok(full_hash)
    }

    pub fn get_current_branch(&self) -> Option<String> {
        let head_path = Path::new(&self.project.path).join(".git/HEAD");
        if let Ok(content) = std::fs::read_to_string(head_path) {
            let content = content.trim();
            if let Some(rest) = content.strip_prefix("ref: refs/heads/") {
                return Some(rest.to_string());
            } else {
                // Detached HEAD or check if it's a raw SHA
                if content.len() >= 7 {
                    return Some(content[..7].to_string());
                }
                return Some(content.to_string());
            }
        }
        None
    }

    pub fn list_files(
        &self,
        filter: Option<&str>,
        branch: Option<&str>,
    ) -> AppResult<Vec<FileEntry>> {
        self.db.get_recent_files(100, filter, branch)
    }

    pub fn list_branches(&self) -> AppResult<Vec<String>> {
        self.db.get_distinct_branches()
    }

    pub fn get_history(&self, file_path: &str) -> AppResult<Vec<Snapshot>> {
        self.db.get_history(file_path)
    }

    pub fn get_recent_activity(&self, limit: usize) -> AppResult<Vec<Snapshot>> {
        self.db.get_recent_activity(limit)
    }

    pub fn get_content(&self, hash_raw: &str) -> AppResult<Vec<u8>> {
        // Try direct read first
        match self.fs.read(hash_raw) {
            Ok(content) => return Ok(content),
            Err(_) => {
                // Try to resolve if it's a short hash
                let hash = match self.db.resolve_hash(hash_raw)? {
                    Some(full) => full,
                    None => hash_raw.to_string(),
                };

                // Try with resolved hash
                match self.fs.read(&hash) {
                    Ok(content) => Ok(content),
                    Err(_) => {
                        // Fallback: try to reassemble from chunks (audit 5.4)
                        let chunks = self.db.get_chunks_for_hash(&hash)?;
                        if !chunks.is_empty() {
                            let mut full_content = Vec::new();
                            for chunk_hash in chunks {
                                let chunk_data = self.fs.read(&chunk_hash)?;
                                full_content.extend_from_slice(&chunk_data);
                            }
                            Ok(full_content)
                        } else {
                            Err(AppError::IoGeneric(std::io::Error::new(
                                std::io::ErrorKind::NotFound,
                                "Content not found",
                            )))
                        }
                    }
                }
            }
        }
    }

    /// Restore a file from a snapshot, creating a backup snapshot first (audit 1.4).
    /// Uses atomic write via tempfile + rename for crash safety.
    pub fn restore_file(&self, hash_raw: &str, target_path: &str) -> AppResult<()> {
        let target = Path::new(target_path);
        let project_root = Path::new(&self.project.path);

        // Security: Ensure target is within project root
        let target_canonical =
            crate::utils::validation::PathValidator::validate_within(project_root, target)?;

        // Try to resolve hash if it's short
        let hash = match self.db.resolve_hash(hash_raw)? {
            Some(full) => full,
            None => {
                if hash_raw.len() == 64 {
                    hash_raw.to_string()
                } else {
                    return Err(AppError::Security(format!(
                        "Invalid or ambiguous hash: {}",
                        hash_raw
                    )));
                }
            }
        };

        // Create a safety snapshot of the current file BEFORE overwriting
        if target_canonical.exists() {
            if let Err(e) = self.save_snapshot_from_file(&target_canonical) {
                eprintln!("Warning: failed to create pre-restore snapshot: {}", e);
            }
        }

        let content = self.fs.read(&hash)?;

        // Atomic write: write to tempfile then rename (audit 1.4)
        let parent = target_canonical
            .parent()
            .ok_or_else(|| AppError::Security("No parent dir".into()))?;
        let mut temp = tempfile::NamedTempFile::new_in(parent).map_err(|e| AppError::Io {
            path: parent.to_path_buf(),
            source: e,
        })?;

        std::io::Write::write_all(&mut temp, &content).map_err(|e| AppError::Io {
            path: temp.path().to_path_buf(),
            source: e,
        })?;

        temp.persist(&target_canonical).map_err(|e| AppError::Io {
            path: target_canonical.clone(),
            source: e.error,
        })?;

        Ok(())
    }

    pub fn grep_contents(
        &self,
        query: &str,
        path_filter: Option<&str>,
    ) -> AppResult<Vec<SearchResult>> {
        if query.is_empty() {
            return Ok(Vec::new());
        }

        let snapshots = self.db.get_all_snapshots_deduped()?;

        const MAX_RESULTS: usize = 200;
        const MAX_MATCHES_PER_FILE: usize = 3;

        let results: Vec<SearchResult> = snapshots
            .par_iter()
            .filter(|snap| {
                if let Some(filter) = path_filter {
                    snap.file_path.contains(filter)
                } else {
                    true
                }
            })
            .filter_map(|snap| {
                let content = self.fs.read(&snap.content_hash).ok()?;
                let text = String::from_utf8_lossy(&content);

                let query_lower = query.to_lowercase();
                let mut matches = Vec::new();

                for (line_idx, line) in text.lines().enumerate() {
                    if line.to_lowercase().contains(&query_lower) {
                        // Safe truncation at char boundary
                        let trimmed = line.trim().to_string();
                        let display = if trimmed.len() > 120 {
                            let safe_end = trimmed
                                .char_indices()
                                .map(|(i, _)| i)
                                .take_while(|&i| i <= 120)
                                .last()
                                .unwrap_or(0);
                            format!("{}...", &trimmed[..safe_end])
                        } else {
                            trimmed
                        };

                        matches.push(SearchResult {
                            file_path: snap.file_path.clone(),
                            content_hash: snap.content_hash.clone(),
                            timestamp: snap.timestamp.clone(),
                            git_branch: snap.git_branch.clone(),
                            line_number: line_idx + 1,
                            content: display,
                        });

                        if matches.len() >= MAX_MATCHES_PER_FILE {
                            break;
                        }
                    }
                }

                if matches.is_empty() {
                    None
                } else {
                    Some(matches)
                }
            })
            .flatten()
            .take_any(MAX_RESULTS)
            .collect();

        Ok(results)
    }

    /// Apply specific hunks from a snapshot to the current file on disk.
    /// Concepts borrowed from Git hunk staging.
    pub fn apply_selective_patch(
        &self,
        target_path: &str,
        snapshot_hash: &str,
        selected_hunk_indices: &[usize],
    ) -> AppResult<()> {
        let target = Path::new(target_path);
        if !target.exists() {
            return Err(AppError::IoGeneric(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Target file for selective restore does not exist",
            )));
        }

        // 1. Get contents
        let current_bytes = std::fs::read(target).map_err(AppError::IoGeneric)?;
        let current_text = String::from_utf8_lossy(&current_bytes).to_string();

        let snapshot_bytes = self.fs.read(snapshot_hash)?;
        let snapshot_text = String::from_utf8_lossy(&snapshot_bytes).to_string();

        // 2. Compute diff from current disk to snapshot
        // We use this diff to identify hunks that we CAN apply.
        use similar::{ChangeTag, TextDiff};
        let diff = TextDiff::from_lines(&current_text, &snapshot_text);

        let mut new_lines = Vec::new();
        let mut current_hunk = 0;
        let mut in_change = false;

        // Iterate through all changes and apply only selected hunks
        for change in diff.iter_all_changes() {
            let tag = change.tag();

            if tag != ChangeTag::Equal {
                if !in_change {
                    in_change = true;
                    current_hunk += 1;
                }
            } else {
                in_change = false;
            }

            let apply_this_change = if tag == ChangeTag::Equal {
                true // Keep equal lines
            } else {
                // If it's a change, only apply if its hunk index is selected
                selected_hunk_indices.contains(&current_hunk)
            };

            match tag {
                ChangeTag::Equal => {
                    new_lines.push(change.value());
                }
                ChangeTag::Delete => {
                    if !apply_this_change {
                        // We DON'T apply the deletion, so we KEEP the line
                        new_lines.push(change.value());
                    }
                    // Else: we apply the deletion (skip adding the line)
                }
                ChangeTag::Insert => {
                    if apply_this_change {
                        // We apply the insertion
                        new_lines.push(change.value());
                    }
                    // Else: we skip the insertion
                }
            }
        }

        // 3. Write back atomically
        let new_content = new_lines.join("");

        // Backup first
        self.save_snapshot_from_file(target)?;

        let parent = target.parent().unwrap_or(Path::new("."));
        let mut temp = tempfile::NamedTempFile::new_in(parent).map_err(AppError::IoGeneric)?;
        std::io::Write::write_all(&mut temp, new_content.as_bytes())
            .map_err(AppError::IoGeneric)?;
        temp.persist(target).map_err(|e| {
            AppError::IoGeneric(std::io::Error::new(
                e.error.kind(),
                format!("Failed to persist selective patch: {}", e.error),
            ))
        })?;

        Ok(())
    }

    // -----------------------------------------------------------------------
    // Sessions
    // -----------------------------------------------------------------------

    pub fn list_sessions(&self, limit: usize) -> AppResult<Vec<Session>> {
        self.db.list_sessions(limit)
    }

    pub fn get_active_session(&self) -> AppResult<Option<Session>> {
        self.db.get_active_session()
    }

    // -----------------------------------------------------------------------
    // Mass Revert
    // -----------------------------------------------------------------------

    /// Create a checkpoint capturing the current state of all tracked files.
    pub fn create_checkpoint(&self, description: Option<&str>) -> AppResult<String> {
        let state = self.db.get_latest_state()?;
        let timestamp = chrono::Local::now().to_rfc3339();
        let file_states_json = serde_json::to_string(&state)
            .map_err(|e| AppError::Internal(format!("Failed to serialize checkpoint: {}", e)))?;
        self.db
            .save_checkpoint(&timestamp, description, &file_states_json)
    }

    pub fn list_checkpoints(&self) -> AppResult<Vec<(String, String, Option<String>)>> {
        self.db.list_checkpoints()
    }

    /// Delete a checkpoint by its hash.
    pub fn delete_checkpoint(&self, hash: &str) -> AppResult<bool> {
        self.db.delete_checkpoint(hash)
    }

    /// List all Git commits with their metadata.
    pub fn list_commits(&self) -> AppResult<Vec<(String, String, String, String, usize)>> {
        self.db.get_commits()
    }

    /// Get details of a specific commit by hash.
    pub fn get_commit_details(
        &self,
        hash: &str,
    ) -> AppResult<Option<(String, String, String, String)>> {
        self.db.get_commit_by_hash(hash)
    }

    /// Get all files included in a specific commit.
    pub fn get_commit_files(&self, hash: &str) -> AppResult<Vec<(String, String, String)>> {
        self.db.get_commit_files(hash)
    }

    /// Insert a Git commit into the database.
    pub fn insert_git_commit(
        &self,
        hash: &str,
        message: &str,
        author: &str,
        timestamp: &str,
    ) -> AppResult<()> {
        self.db.insert_git_commit(hash, message, author, timestamp)
    }

    /// Revert entire project to a specific checkpoint hash.
    pub fn revert_to_checkpoint(&self, hash_query: &str) -> AppResult<usize> {
        let (_timestamp, file_states_json, _desc) = self
            .db
            .get_checkpoint_by_hash(hash_query)?
            .ok_or_else(|| AppError::Internal(format!("Checkpoint not found: {}", hash_query)))?;

        let state: Vec<(String, String)> = serde_json::from_str(&file_states_json)
            .map_err(|e| AppError::Internal(format!("Failed to parse checkpoint: {}", e)))?;

        // 1. Safety checkpoint
        let short_hash = if hash_query.len() > 8 {
            &hash_query[..8]
        } else {
            hash_query
        };
        let _ = self.create_checkpoint(Some(&format!(
            "Safety save before reverting to {}",
            short_hash
        )));

        // 2. Restore files
        let mut count = 0;
        for (path, hash) in state {
            if self.restore_file(&hash, &path).is_ok() {
                count += 1;
            }
        }

        Ok(count)
    }

    /// Get details of a checkpoint by hash.
    pub fn get_checkpoint_details(
        &self,
        hash: &str,
    ) -> AppResult<Option<(String, String, Option<String>)>> {
        let result = self.db.get_checkpoint_by_hash(hash)?;
        if let Some((ts, states_json, desc)) = result {
            Ok(Some((ts, states_json, desc)))
        } else {
            Ok(None)
        }
    }

    /// Revert the entire project to a specific timestamp.
    /// Creates a safety checkpoint first, then restores all files.
    pub fn revert_to_timestamp(&self, timestamp: &str) -> AppResult<usize> {
        // 1. Safety checkpoint
        self.create_checkpoint(Some("Pre-revert safety checkpoint"))?;

        // 2. Get the project state at that timestamp
        let state = self.db.get_state_at_timestamp(timestamp)?;
        if state.is_empty() {
            return Err(AppError::Internal(
                "No snapshots found before the given timestamp".into(),
            ));
        }

        // 3. Restore each file
        let mut restored = 0;
        for (file_path, content_hash) in &state {
            match self.restore_file(content_hash, file_path) {
                Ok(()) => restored += 1,
                Err(e) => {
                    eprintln!("Warning: failed to restore {}: {}", file_path, e);
                }
            }
        }

        Ok(restored)
    }

    /// Generate a unified diff between two versions of a file.
    /// If base_hash is None, diffs against current disk content.
    pub fn get_file_diff(
        &self,
        file_path: &str,
        base_hash: Option<&str>,
        target_hash: &str,
    ) -> AppResult<String> {
        let (base_text, base_name) = if let Some(hash) = base_hash {
            let data = self.get_content(hash)?;
            (
                String::from_utf8_lossy(&data).to_string(),
                format!("Snapshot: {}", &hash[..8.min(hash.len())]),
            )
        } else {
            let path = Path::new(file_path);
            if !path.exists() {
                ("".to_string(), "Missing".to_string())
            } else {
                (
                    String::from_utf8_lossy(&std::fs::read(path).map_err(AppError::IoGeneric)?)
                        .to_string(),
                    "Current".to_string(),
                )
            }
        };

        let target_data = self.get_content(target_hash)?;
        let target_text = String::from_utf8_lossy(&target_data).to_string();
        let target_name = format!("Snapshot: {}", &target_hash[..8.min(target_hash.len())]);

        use similar::TextDiff;

        let diff_output = TextDiff::from_lines(&base_text, &target_text)
            .unified_diff()
            .header(&base_name, &target_name)
            .to_string();

        Ok(diff_output)
    }

    pub fn get_symbols(&self, snapshot_id: i64) -> AppResult<Vec<crate::models::SemanticSymbol>> {
        self.db.get_symbols_for_snapshot(snapshot_id)
    }

    pub fn get_file_info(&self, file_path: &str) -> AppResult<crate::protocol::FileInfoResponse> {
        let history = self.db.get_history(file_path)?;
        let count = history.len();
        let mut total_bytes = 0;
        let mut first_seen = String::new();
        let mut last_modified = String::new();

        if let Some(first) = history.last() {
            first_seen = first.timestamp.clone();
        }
        if let Some(last) = history.first() {
            last_modified = last.timestamp.clone();
        }

        // Sum size of all unique chunks in history
        use std::collections::HashSet;
        let mut unique_hashes = HashSet::new();
        for snap in &history {
            unique_hashes.insert(snap.content_hash.clone());
        }

        for hash in unique_hashes {
            if let Ok(size) = self.fs.get_size(&hash) {
                total_bytes += size;
            }
        }

        Ok(crate::protocol::FileInfoResponse {
            path: file_path.to_string(),
            snapshot_count: count,
            total_bytes,
            last_modified,
            first_seen,
        })
    }

    /// Surgically restore a specific symbol from a snapshot into the current file.
    pub fn restore_symbol(
        &self,
        file_path: &str,
        content_hash: &str,
        symbol_name: &str,
    ) -> AppResult<()> {
        let target = Path::new(file_path);
        if !target.exists() {
            return Err(AppError::IoGeneric(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Target file does not exist",
            )));
        }

        // 1. Analyze Current File (Destination)
        let current_vec = std::fs::read(target).map_err(AppError::IoGeneric)?;
        let current_bytes = bytes::Bytes::from(current_vec);
        let ext = target.extension().and_then(|s| s.to_str()).unwrap_or("");

        let mut parser = SemanticParser::new()?;
        // Parse current state to find where to inject
        let current_symbols = parser.parse_symbols(&current_bytes, ext, 0, Some(file_path))?;
        let dest_sym = current_symbols
            .iter()
            .find(|s| s.name == symbol_name)
            .ok_or_else(|| {
                AppError::Internal(format!(
                    "Symbol '{}' not found in current file. Cannot replace missing symbol.",
                    symbol_name
                ))
            })?;

        // 2. Analyze Snapshot (Source)
        let snap_vec = self.fs.read(content_hash)?;
        let snap_bytes = bytes::Bytes::from(snap_vec);
        let snap_symbols = parser.parse_symbols(&snap_bytes, ext, 0, None)?;

        let source_sym = snap_symbols
            .iter()
            .find(|s| s.name == symbol_name)
            .ok_or_else(|| {
                AppError::Internal(format!(
                    "Symbol '{}' not found in snapshot {}",
                    symbol_name, content_hash
                ))
            })?;

        // 3. Surgical Transplant
        let mut new_bytes = Vec::new();
        // Keep everything before the symbol
        new_bytes.extend_from_slice(&current_bytes[0..dest_sym.start_byte]);
        // Insert the OLD symbol content
        new_bytes.extend_from_slice(&snap_bytes[source_sym.start_byte..source_sym.end_byte]);
        // Keep everything after the symbol
        new_bytes.extend_from_slice(&current_bytes[dest_sym.end_byte..]);

        // 4. Save (Safety First: Backup!)
        self.save_snapshot_from_file(target)?;

        let parent = target.parent().unwrap_or(Path::new("."));
        let mut temp = tempfile::NamedTempFile::new_in(parent).map_err(AppError::IoGeneric)?;
        std::io::Write::write_all(&mut temp, &new_bytes).map_err(AppError::IoGeneric)?;
        temp.persist(target).map_err(|e| {
            AppError::IoGeneric(std::io::Error::new(
                e.error.kind(),
                format!("Failed to persist symbol restore: {}", e.error),
            ))
        })?;

        Ok(())
    }

    pub fn diff_symbol(
        &self,
        file_path: &str,
        symbol_name: &str,
        base_hash: Option<&str>,
        target_hash: &str,
    ) -> AppResult<String> {
        let mut parser = SemanticParser::new()?;
        let ext = Path::new(file_path)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("");

        // Get target content
        let target_bytes = if target_hash == "__DISK__" {
            let vec = std::fs::read(file_path).map_err(AppError::IoGeneric)?;
            bytes::Bytes::from(vec)
        } else {
            let data = self.fs.read(target_hash)?;
            bytes::Bytes::from(data)
        };

        // Extract target symbol code
        let target_symbols = parser.parse_symbols(&target_bytes, ext, 0, Some(file_path))?;
        let target_sym = target_symbols
            .iter()
            .find(|s| s.name == symbol_name)
            .ok_or_else(|| {
                AppError::Internal(format!("Symbol '{}' not found in target", symbol_name))
            })?;
        let target_code =
            String::from_utf8_lossy(&target_bytes[target_sym.start_byte..target_sym.end_byte])
                .to_string();

        // Get base content
        let base_code = if let Some(bh) = base_hash {
            let base_vec = self.fs.read(bh)?;
            let base_bytes = bytes::Bytes::from(base_vec);
            let base_symbols = parser.parse_symbols(&base_bytes, ext, 0, None)?;
            let base_sym = base_symbols
                .iter()
                .find(|s| s.name == symbol_name)
                .ok_or_else(|| {
                    AppError::Internal(format!("Symbol '{}' not found in base", symbol_name))
                })?;
            String::from_utf8_lossy(&base_bytes[base_sym.start_byte..base_sym.end_byte]).to_string()
        } else {
            String::new()
        };

        // Simple line-based diff using similar crate
        let mut diff = String::new();
        let diff_obj = similar::TextDiff::from_lines(&base_code, &target_code);
        for change in diff_obj.iter_all_changes() {
            let sign = match change.tag() {
                similar::ChangeTag::Delete => "-",
                similar::ChangeTag::Insert => "+",
                similar::ChangeTag::Equal => " ",
            };
            diff.push_str(&format!("{}{}", sign, change));
        }
        Ok(diff)
    }

    pub fn find_symbols(&self, query: &str) -> AppResult<Vec<crate::protocol::SymbolLocation>> {
        let symbols = self.db.find_symbols_by_name(query)?;
        let mut locations = Vec::new();
        for s in symbols {
            // We need to get the file path for each symbol.
            // Symbol model in DB should have snapshot_id, which links to snapshot -> file_path.
            // Actually, get_symbol_history already does this.
            // Let's assume we have a DB method for this.
            if let Ok(Some(snap)) = self.db.get_snapshot_by_id(s.snapshot_id) {
                locations.push(crate::protocol::SymbolLocation {
                    name: s.name,
                    kind: s.kind,
                    file_path: snap.file_path,
                    start_line: s.start_line,
                    end_line: s.end_line,
                    structural_hash: s.structural_hash,
                });
            }
        }
        Ok(locations)
    }
}
