use crate::error::{AppError, AppResult};
use crate::models::{FileEntry, SemanticSymbol, Session, Snapshot, SymbolReference};
use redb::{Database as Redb, ReadableTable, ReadableTableMetadata, TableDefinition};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;

// Table Definitions
const SNAPSHOTS: TableDefinition<u64, &[u8]> = TableDefinition::new("snapshots");
const GIT_COMMITS: TableDefinition<&str, &[u8]> = TableDefinition::new("git_commits");
const SESSIONS: TableDefinition<u64, &[u8]> = TableDefinition::new("sessions");
const CHECKPOINTS: TableDefinition<&str, &[u8]> = TableDefinition::new("project_checkpoints");
const CHUNKS: TableDefinition<&str, &[u8]> = TableDefinition::new("chunks");
const SNAPSHOT_CHUNKS: TableDefinition<(u64, u32), &str> = TableDefinition::new("snapshot_chunks");
const SYMBOLS: TableDefinition<u64, &[u8]> = TableDefinition::new("symbols");
const SYMBOL_REFERENCES: TableDefinition<u64, &[u8]> = TableDefinition::new("symbol_references");
const SYMBOL_DELTAS: TableDefinition<u64, &[u8]> = TableDefinition::new("symbol_deltas");
const METADATA: TableDefinition<&str, u64> = TableDefinition::new("metadata");

// Improvements: String Interning & Trigram Index
const STRINGS: TableDefinition<u32, &str> = TableDefinition::new("strings");
const STRING_INDEX: TableDefinition<&str, u32> = TableDefinition::new("string_index");
const CHUNK_TRIGRAMS: TableDefinition<&str, u64> = TableDefinition::new("chunk_trigrams");

#[derive(Serialize, Deserialize, Clone)]
struct SnapshotData {
    id: i64,
    file_path_id: u32,
    timestamp: String,
    content_hash: String,
    git_branch_id: Option<u32>,
    session_id: Option<i64>,
    commit_hash: Option<String>,
    #[serde(default)]
    commit_message: Option<String>,
}

/// Helper function to safely deserialize SnapshotData, skipping old format or corrupted records
fn deserialize_snapshot_data(value: &[u8]) -> Option<SnapshotData> {
    let data: SnapshotData = bincode::deserialize(value).ok()?;
    if data.file_path_id == 0 {
        return None;
    }
    Some(data)
}

#[derive(Serialize, Deserialize, Clone)]
struct GitCommitData {
    hash: String,
    message: String,
    author: String,
    timestamp: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct SessionData {
    id: i64,
    start_time: String,
    end_time: Option<String>,
    git_branch_id: Option<u32>,
    file_count: usize,
    snapshot_count: usize,
}

#[derive(Serialize, Deserialize, Clone)]
struct CheckpointData {
    hash: String,
    timestamp: String,
    description: Option<String>,
    file_states: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct ChunkData {
    hash: String,
    kind_id: u32,
}

#[derive(Serialize, Deserialize, Clone)]
struct SymbolData {
    id: i64,
    name_id: u32,
    kind_id: u32,
    scope_id: Option<u32>,
    snapshot_id: i64,
    chunk_hash: String,
    structural_hash: String,
    start_line: usize,
    end_line: usize,
    start_byte: usize,
    end_byte: usize,
    parent_id: Option<i64>,
}

#[derive(Serialize, Deserialize, Clone)]
struct ReferenceData {
    id: i64,
    symbol_name_id: u32,
    snapshot_id: i64,
    start_line: usize,
    start_byte: usize,
}

#[derive(Serialize, Deserialize, Clone)]
struct DeltaData {
    id: i64,
    from_snapshot_id: Option<i64>,
    to_snapshot_id: i64,
    symbol_name_id: u32,
    new_name_id: Option<u32>,
    delta_kind: String,
    structural_hash: String,
}

pub struct Database {
    db: Redb,
    pub path: PathBuf,
}

impl Database {
    pub fn new(path: PathBuf) -> AppResult<Self> {
        let db = Redb::builder()
            .create(&path)
            .map_err(|e| AppError::Internal(format!("Failed to open redb: {}", e)))?;

        let write_txn = db
            .begin_write()
            .map_err(|e| AppError::Database(e.to_string()))?;
        {
            let _ = write_txn
                .open_table(SNAPSHOTS)
                .map_err(|e| AppError::Database(e.to_string()))?;
            let _ = write_txn
                .open_table(GIT_COMMITS)
                .map_err(|e| AppError::Database(e.to_string()))?;
            let _ = write_txn
                .open_table(SESSIONS)
                .map_err(|e| AppError::Database(e.to_string()))?;
            let _ = write_txn
                .open_table(CHECKPOINTS)
                .map_err(|e| AppError::Database(e.to_string()))?;
            let _ = write_txn
                .open_table(CHUNKS)
                .map_err(|e| AppError::Database(e.to_string()))?;
            let _ = write_txn
                .open_table(SNAPSHOT_CHUNKS)
                .map_err(|e| AppError::Database(e.to_string()))?;
            let _ = write_txn
                .open_table(SYMBOLS)
                .map_err(|e| AppError::Database(e.to_string()))?;
            let _ = write_txn
                .open_table(SYMBOL_REFERENCES)
                .map_err(|e| AppError::Database(e.to_string()))?;
            let _ = write_txn
                .open_table(SYMBOL_DELTAS)
                .map_err(|e| AppError::Database(e.to_string()))?;
            let _ = write_txn
                .open_table(STRINGS)
                .map_err(|e| AppError::Database(e.to_string()))?;
            let _ = write_txn
                .open_table(STRING_INDEX)
                .map_err(|e| AppError::Database(e.to_string()))?;
            let _ = write_txn
                .open_table(CHUNK_TRIGRAMS)
                .map_err(|e| AppError::Database(e.to_string()))?;

            let mut meta = write_txn
                .open_table(METADATA)
                .map_err(|e| AppError::Database(e.to_string()))?;

            if meta
                .get("snapshot_id")
                .map_err(|e| AppError::Database(e.to_string()))?
                .is_none()
            {
                meta.insert("snapshot_id", 0)
                    .map_err(|e| AppError::Database(e.to_string()))?;
            }
            if meta
                .get("session_id")
                .map_err(|e| AppError::Database(e.to_string()))?
                .is_none()
            {
                meta.insert("session_id", 0)
                    .map_err(|e| AppError::Database(e.to_string()))?;
            }
            if meta
                .get("symbol_id")
                .map_err(|e| AppError::Database(e.to_string()))?
                .is_none()
            {
                meta.insert("symbol_id", 0)
                    .map_err(|e| AppError::Database(e.to_string()))?;
            }
            if meta
                .get("reference_id")
                .map_err(|e| AppError::Database(e.to_string()))?
                .is_none()
            {
                meta.insert("reference_id", 0)
                    .map_err(|e| AppError::Database(e.to_string()))?;
            }
            if meta
                .get("delta_id")
                .map_err(|e| AppError::Database(e.to_string()))?
                .is_none()
            {
                meta.insert("delta_id", 0)
                    .map_err(|e| AppError::Database(e.to_string()))?;
            }
            if meta
                .get("string_id")
                .map_err(|e| AppError::Database(e.to_string()))?
                .is_none()
            {
                meta.insert("string_id", 0)
                    .map_err(|e| AppError::Database(e.to_string()))?;
            }
        }
        write_txn
            .commit()
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(Self { db, path })
    }

    fn next_id(&self, key: &str) -> AppResult<u64> {
        let write_txn = self
            .db
            .begin_write()
            .map_err(|e| AppError::Database(e.to_string()))?;
        let next = {
            let mut meta = write_txn
                .open_table(METADATA)
                .map_err(|e| AppError::Database(e.to_string()))?;
            let current = meta
                .get(key)
                .map_err(|e| AppError::Database(e.to_string()))?
                .map(|v| v.value())
                .unwrap_or(0);
            let next = current + 1;
            meta.insert(key, next)
                .map_err(|e| AppError::Database(e.to_string()))?;
            next
        };
        write_txn
            .commit()
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(next)
    }

    fn intern_string(&self, s: &str) -> AppResult<u32> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| AppError::Database(e.to_string()))?;
        let index = read_txn
            .open_table(STRING_INDEX)
            .map_err(|e| AppError::Database(e.to_string()))?;
        if let Some(id) = index
            .get(s)
            .map_err(|e| AppError::Database(e.to_string()))?
        {
            return Ok(id.value());
        }
        drop(read_txn);
        let write_txn = self
            .db
            .begin_write()
            .map_err(|e| AppError::Database(e.to_string()))?;
        let id = {
            let mut meta = write_txn
                .open_table(METADATA)
                .map_err(|e| AppError::Database(e.to_string()))?;
            let current = meta
                .get("string_id")
                .map_err(|e| AppError::Database(e.to_string()))?
                .map(|v| v.value())
                .unwrap_or(0) as u32;
            let next = current + 1;
            meta.insert("string_id", next as u64)
                .map_err(|e| AppError::Database(e.to_string()))?;
            let mut strings = write_txn
                .open_table(STRINGS)
                .map_err(|e| AppError::Database(e.to_string()))?;
            let mut index = write_txn
                .open_table(STRING_INDEX)
                .map_err(|e| AppError::Database(e.to_string()))?;
            strings
                .insert(next, s)
                .map_err(|e| AppError::Database(e.to_string()))?;
            index
                .insert(s, next)
                .map_err(|e| AppError::Database(e.to_string()))?;
            next
        };
        write_txn
            .commit()
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(id)
    }

    fn lookup_string(&self, id: u32) -> AppResult<String> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| AppError::Database(e.to_string()))?;
        let table = read_txn
            .open_table(STRINGS)
            .map_err(|e| AppError::Database(e.to_string()))?;
        let s = table
            .get(id)
            .map_err(|e| AppError::Database(e.to_string()))?
            .ok_or_else(|| AppError::Internal(format!("String ID {} not found", id)))?;
        Ok(s.value().to_string())
    }

    pub fn get_git_commit(&self, hash: &str) -> AppResult<Option<(String, String, String)>> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| AppError::Database(e.to_string()))?;
        let table = read_txn
            .open_table(GIT_COMMITS)
            .map_err(|e| AppError::Database(e.to_string()))?;
        if let Some(v) = table
            .get(hash)
            .map_err(|e| AppError::Database(e.to_string()))?
        {
            let data: GitCommitData =
                bincode::deserialize(v.value()).map_err(|e| AppError::Internal(e.to_string()))?;
            Ok(Some((data.message, data.author, data.timestamp)))
        } else {
            Ok(None)
        }
    }

    pub fn insert_git_commit(
        &self,
        hash: &str,
        message: &str,
        author: &str,
        timestamp: &str,
    ) -> AppResult<()> {
        let write_txn = self
            .db
            .begin_write()
            .map_err(|e| AppError::Database(e.to_string()))?;
        {
            let mut table = write_txn
                .open_table(GIT_COMMITS)
                .map_err(|e| AppError::Database(e.to_string()))?;
            let data = GitCommitData {
                hash: hash.to_string(),
                message: message.to_string(),
                author: author.to_string(),
                timestamp: timestamp.to_string(),
            };
            let bytes = bincode::serialize(&data).map_err(|e| AppError::Internal(e.to_string()))?;
            table
                .insert(hash, &*bytes)
                .map_err(|e| AppError::Database(e.to_string()))?;
        }
        write_txn
            .commit()
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(())
    }

    pub fn link_snapshot_to_commit(&self, snapshot_id: i64, commit_hash: &str) -> AppResult<()> {
        let write_txn = self
            .db
            .begin_write()
            .map_err(|e| AppError::Database(e.to_string()))?;
        {
            let mut table = write_txn
                .open_table(SNAPSHOTS)
                .map_err(|e| AppError::Database(e.to_string()))?;
            let data = if let Some(v) = table
                .get(snapshot_id as u64)
                .map_err(|e| AppError::Database(e.to_string()))?
            {
                let mut data: SnapshotData = bincode::deserialize(v.value())
                    .map_err(|e| AppError::Internal(e.to_string()))?;
                data.commit_hash = Some(commit_hash.to_string());
                Some(data)
            } else {
                None
            };
            if let Some(d) = data {
                let bytes =
                    bincode::serialize(&d).map_err(|e| AppError::Internal(e.to_string()))?;
                table
                    .insert(snapshot_id as u64, &*bytes)
                    .map_err(|e| AppError::Database(e.to_string()))?;
            }
        }
        write_txn
            .commit()
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(())
    }

    pub fn insert_snapshot(
        &self,
        file_path: &str,
        timestamp: &str,
        content_hash: &str,
        git_branch: Option<&str>,
        session_id: Option<i64>,
    ) -> AppResult<i64> {
        let id = self.next_id("snapshot_id")?;
        let file_path_id = self.intern_string(file_path)?;
        let git_branch_id = if let Some(b) = git_branch {
            Some(self.intern_string(b)?)
        } else {
            None
        };
        let write_txn = self
            .db
            .begin_write()
            .map_err(|e| AppError::Database(e.to_string()))?;
        {
            let mut table = write_txn
                .open_table(SNAPSHOTS)
                .map_err(|e| AppError::Database(e.to_string()))?;
            let data = SnapshotData {
                id: id as i64,
                file_path_id,
                timestamp: timestamp.to_string(),
                content_hash: content_hash.to_string(),
                git_branch_id,
                session_id,
                commit_hash: None,
                commit_message: None,
            };
            let bytes = bincode::serialize(&data).map_err(|e| AppError::Internal(e.to_string()))?;
            table
                .insert(id, &*bytes)
                .map_err(|e| AppError::Database(e.to_string()))?;
        }
        write_txn
            .commit()
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(id as i64)
    }

    pub fn get_last_hash(&self, file_path: &str) -> AppResult<Option<String>> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| AppError::Database(e.to_string()))?;
        let table = read_txn
            .open_table(SNAPSHOTS)
            .map_err(|e| AppError::Database(e.to_string()))?;
        let index = read_txn
            .open_table(STRING_INDEX)
            .map_err(|e| AppError::Database(e.to_string()))?;
        let file_path_id = match index
            .get(file_path)
            .map_err(|e| AppError::Database(e.to_string()))?
        {
            Some(id) => id.value(),
            None => return Ok(None),
        };
        let mut last_hash = None;
        let mut last_id = 0;
        for res in table
            .iter()
            .map_err(|e| AppError::Database(e.to_string()))?
        {
            let (id, v) = res.map_err(|e| AppError::Database(e.to_string()))?;
            let data: SnapshotData =
                bincode::deserialize(v.value()).map_err(|e| AppError::Internal(e.to_string()))?;
            if data.file_path_id == file_path_id && id.value() >= last_id {
                last_id = id.value();
                last_hash = Some(data.content_hash);
            }
        }
        Ok(last_hash)
    }

    pub fn get_history(&self, file_path: &str) -> AppResult<Vec<Snapshot>> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| AppError::Database(e.to_string()))?;
        let table = read_txn
            .open_table(SNAPSHOTS)
            .map_err(|e| AppError::Database(e.to_string()))?;
        let mut history = Vec::new();
        for res in table
            .iter()
            .map_err(|e| AppError::Database(e.to_string()))?
        {
            let (_, v) = res.map_err(|e| AppError::Database(e.to_string()))?;
            // Skip records that can't be parsed (old format or corrupted)
            let Ok(data) = bincode::deserialize::<SnapshotData>(v.value()) else {
                continue;
            };
            // Skip records with invalid file_path_id
            if data.file_path_id == 0 {
                continue;
            }
            let path = match self.lookup_string(data.file_path_id) {
                Ok(p) => p,
                Err(_) => continue,
            };
            if path.contains(file_path) {
                let branch = if let Some(bid) = data.git_branch_id {
                    Some(self.lookup_string(bid)?)
                } else {
                    None
                };
                history.push(Snapshot {
                    id: data.id,
                    file_path: path,
                    timestamp: data.timestamp,
                    content_hash: data.content_hash,
                    git_branch: branch,
                    session_id: data.session_id,
                    commit_hash: data.commit_hash,
                    commit_message: data.commit_message,
                });
            }
        }
        history.sort_by(|a, b| b.id.cmp(&a.id));
        Ok(history)
    }

    pub fn get_global_history(&self, limit: usize) -> AppResult<Vec<Snapshot>> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| AppError::Database(e.to_string()))?;
        let table = read_txn
            .open_table(SNAPSHOTS)
            .map_err(|e| AppError::Database(e.to_string()))?;
        let mut history = Vec::new();
        for res in table
            .iter()
            .map_err(|e| AppError::Database(e.to_string()))?
        {
            let (_, v) = res.map_err(|e| AppError::Database(e.to_string()))?;
            let Some(data) = deserialize_snapshot_data(v.value()) else {
                continue;
            };
            let path = match self.lookup_string(data.file_path_id) {
                Ok(p) => p,
                Err(_) => continue,
            };
            let branch = if let Some(bid) = data.git_branch_id {
                Some(self.lookup_string(bid)?)
            } else {
                None
            };
            history.push(Snapshot {
                id: data.id,
                file_path: path,
                timestamp: data.timestamp,
                content_hash: data.content_hash,
                git_branch: branch,
                session_id: data.session_id,
                commit_hash: data.commit_hash,
                commit_message: data.commit_message,
            });
        }
        history.sort_by(|a, b| b.id.cmp(&a.id));
        history.truncate(limit);
        Ok(history)
    }

    pub fn get_history_by_hash(&self, content_hash: &str) -> AppResult<Vec<Snapshot>> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| AppError::Database(e.to_string()))?;
        let table = read_txn
            .open_table(SNAPSHOTS)
            .map_err(|e| AppError::Database(e.to_string()))?;
        let mut history = Vec::new();
        for res in table
            .iter()
            .map_err(|e| AppError::Database(e.to_string()))?
        {
            let (_, v) = res.map_err(|e| AppError::Database(e.to_string()))?;
            let data: SnapshotData =
                bincode::deserialize(v.value()).map_err(|e| AppError::Internal(e.to_string()))?;
            if data.content_hash == content_hash {
                let path = self.lookup_string(data.file_path_id)?;
                let branch = if let Some(bid) = data.git_branch_id {
                    Some(self.lookup_string(bid)?)
                } else {
                    None
                };
                history.push(Snapshot {
                    id: data.id,
                    file_path: path,
                    timestamp: data.timestamp,
                    content_hash: data.content_hash,
                    git_branch: branch,
                    session_id: data.session_id,
                    commit_hash: data.commit_hash,
                    commit_message: data.commit_message,
                });
            }
        }
        history.sort_by(|a, b| b.id.cmp(&a.id));
        Ok(history)
    }

    pub fn get_max_snapshot_id(&self) -> AppResult<i64> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| AppError::Database(e.to_string()))?;
        let meta = read_txn
            .open_table(METADATA)
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(meta
            .get("snapshot_id")
            .map_err(|e| AppError::Database(e.to_string()))?
            .map(|v| v.value() as i64)
            .unwrap_or(0))
    }

    pub fn get_recent_files(
        &self,
        limit: usize,
        filter: Option<&str>,
        branch: Option<&str>,
    ) -> AppResult<Vec<FileEntry>> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| AppError::Database(e.to_string()))?;
        let table = read_txn
            .open_table(SNAPSHOTS)
            .map_err(|e| AppError::Database(e.to_string()))?;
        let mut files: std::collections::HashMap<u32, String> = std::collections::HashMap::new();
        let branch_id = if let Some(b) = branch {
            let index = read_txn
                .open_table(STRING_INDEX)
                .map_err(|e| AppError::Database(e.to_string()))?;
            match index
                .get(b)
                .map_err(|e| AppError::Database(e.to_string()))?
            {
                Some(id) => Some(id.value()),
                None => return Ok(Vec::new()),
            }
        } else {
            None
        };
        for res in table
            .iter()
            .map_err(|e| AppError::Database(e.to_string()))?
        {
            let (_, v) = res.map_err(|e| AppError::Database(e.to_string()))?;
            let data: SnapshotData =
                bincode::deserialize(v.value()).map_err(|e| AppError::Internal(e.to_string()))?;
            if let Some(bid) = branch_id {
                if data.git_branch_id != Some(bid) {
                    continue;
                }
            }
            let path = self.lookup_string(data.file_path_id)?;
            if let Some(f) = filter {
                if !path.contains(f) {
                    continue;
                }
            }
            let entry = files
                .entry(data.file_path_id)
                .or_insert_with(|| data.timestamp.clone());
            if data.timestamp > *entry {
                *entry = data.timestamp;
            }
        }
        let mut results = Vec::new();
        for (pid, last_update) in files {
            results.push(FileEntry {
                path: self.lookup_string(pid)?,
                last_update,
            });
        }
        results.sort_by(|a, b| b.last_update.cmp(&a.last_update));
        results.truncate(limit);
        Ok(results)
    }

    pub fn get_distinct_branches(&self) -> AppResult<Vec<String>> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| AppError::Database(e.to_string()))?;
        let table = read_txn
            .open_table(SNAPSHOTS)
            .map_err(|e| AppError::Database(e.to_string()))?;
        let mut branches = HashSet::new();
        for res in table
            .iter()
            .map_err(|e| AppError::Database(e.to_string()))?
        {
            let (_, v) = res.map_err(|e| AppError::Database(e.to_string()))?;
            let data: SnapshotData =
                bincode::deserialize(v.value()).map_err(|e| AppError::Internal(e.to_string()))?;
            if let Some(bid) = data.git_branch_id {
                branches.insert(bid);
            }
        }
        let mut results = Vec::new();
        for bid in branches {
            results.push(self.lookup_string(bid)?);
        }
        results.sort();
        Ok(results)
    }

    pub fn get_recent_activity(&self, limit: usize) -> AppResult<Vec<Snapshot>> {
        self.get_global_history(limit)
    }

    pub fn resolve_hash(&self, short_hash: &str) -> AppResult<Option<String>> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| AppError::Database(e.to_string()))?;
        let table = read_txn
            .open_table(SNAPSHOTS)
            .map_err(|e| AppError::Database(e.to_string()))?;
        let short = short_hash.to_lowercase();
        let mut found = HashSet::new();
        for res in table
            .iter()
            .map_err(|e| AppError::Database(e.to_string()))?
        {
            let (_, v) = res.map_err(|e| AppError::Database(e.to_string()))?;
            let data: SnapshotData =
                bincode::deserialize(v.value()).map_err(|e| AppError::Internal(e.to_string()))?;
            if data.content_hash.to_lowercase().starts_with(&short) {
                found.insert(data.content_hash);
                if found.len() > 1 {
                    return Ok(None);
                }
            }
        }
        Ok(if found.len() == 1 {
            found.into_iter().next()
        } else {
            None
        })
    }

    pub fn get_all_unique_snapshots(&self) -> AppResult<Vec<Snapshot>> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| AppError::Database(e.to_string()))?;
        let table = read_txn
            .open_table(SNAPSHOTS)
            .map_err(|e| AppError::Database(e.to_string()))?;
        let mut files: std::collections::HashMap<u32, SnapshotData> =
            std::collections::HashMap::new();
        for res in table
            .iter()
            .map_err(|e| AppError::Database(e.to_string()))?
        {
            let (_, v) = res.map_err(|e| AppError::Database(e.to_string()))?;
            let data: SnapshotData =
                bincode::deserialize(v.value()).map_err(|e| AppError::Internal(e.to_string()))?;
            let entry = files
                .entry(data.file_path_id)
                .or_insert_with(|| data.clone());
            if data.timestamp > entry.timestamp {
                *entry = data;
            }
        }
        let mut results = Vec::new();
        for (_, data) in files {
            let path = self.lookup_string(data.file_path_id)?;
            let branch = if let Some(bid) = data.git_branch_id {
                Some(self.lookup_string(bid)?)
            } else {
                None
            };
            results.push(Snapshot {
                id: data.id,
                file_path: path,
                timestamp: data.timestamp,
                content_hash: data.content_hash,
                git_branch: branch,
                session_id: data.session_id,
                commit_hash: data.commit_hash,
                commit_message: data.commit_message,
            });
        }
        results.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        Ok(results)
    }

    pub fn get_all_snapshots_deduped(&self) -> AppResult<Vec<Snapshot>> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| AppError::Database(e.to_string()))?;
        let table = read_txn
            .open_table(SNAPSHOTS)
            .map_err(|e| AppError::Database(e.to_string()))?;
        let mut deduplicated: std::collections::HashMap<String, SnapshotData> =
            std::collections::HashMap::new();
        for res in table
            .iter()
            .map_err(|e| AppError::Database(e.to_string()))?
        {
            let (_, v) = res.map_err(|e| AppError::Database(e.to_string()))?;
            let data: SnapshotData =
                bincode::deserialize(v.value()).map_err(|e| AppError::Internal(e.to_string()))?;
            let entry = deduplicated
                .entry(data.content_hash.clone())
                .or_insert_with(|| data.clone());
            if data.timestamp > entry.timestamp {
                *entry = data;
            }
        }
        let mut results = Vec::new();
        for (_, data) in deduplicated {
            let path = self.lookup_string(data.file_path_id)?;
            let branch = if let Some(bid) = data.git_branch_id {
                Some(self.lookup_string(bid)?)
            } else {
                None
            };
            results.push(Snapshot {
                id: data.id,
                file_path: path,
                timestamp: data.timestamp,
                content_hash: data.content_hash,
                git_branch: branch,
                session_id: data.session_id,
                commit_hash: data.commit_hash,
                commit_message: data.commit_message,
            });
        }
        results.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        Ok(results)
    }

    pub fn get_all_content_hashes(&self) -> AppResult<HashSet<String>> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| AppError::Database(e.to_string()))?;
        let table = read_txn
            .open_table(SNAPSHOTS)
            .map_err(|e| AppError::Database(e.to_string()))?;
        let mut hashes = HashSet::new();
        for res in table
            .iter()
            .map_err(|e| AppError::Database(e.to_string()))?
        {
            let (_, v) = res.map_err(|e| AppError::Database(e.to_string()))?;
            if let Some(data) = deserialize_snapshot_data(v.value()) {
                hashes.insert(data.content_hash);
            }
        }
        Ok(hashes)
    }

    pub fn prune_snapshots(&self, days: u64) -> AppResult<usize> {
        let cutoff = (chrono::Local::now() - chrono::Duration::days(days as i64)).to_rfc3339();
        let write_txn = self
            .db
            .begin_write()
            .map_err(|e| AppError::Database(e.to_string()))?;
        let mut count = 0;
        {
            let mut snapshots = write_txn
                .open_table(SNAPSHOTS)
                .map_err(|e| AppError::Database(e.to_string()))?;
            let mut snapshot_chunks = write_txn
                .open_table(SNAPSHOT_CHUNKS)
                .map_err(|e| AppError::Database(e.to_string()))?;
            let mut symbols = write_txn
                .open_table(SYMBOLS)
                .map_err(|e| AppError::Database(e.to_string()))?;
            let mut to_delete = Vec::new();
            for res in snapshots
                .iter()
                .map_err(|e| AppError::Database(e.to_string()))?
            {
                let (id, v) = res.map_err(|e| AppError::Database(e.to_string()))?;
                let data: SnapshotData = bincode::deserialize(v.value())
                    .map_err(|e| AppError::Internal(e.to_string()))?;
                if data.timestamp < cutoff && data.commit_hash.is_none() {
                    to_delete.push(id.value());
                }
            }
            for id in &to_delete {
                snapshots
                    .remove(id)
                    .map_err(|e| AppError::Database(e.to_string()))?;
                count += 1;
                let mut chunk_keys = Vec::new();
                for res in snapshot_chunks
                    .iter()
                    .map_err(|e| AppError::Database(e.to_string()))?
                {
                    let (key, _) = res.map_err(|e| AppError::Database(e.to_string()))?;
                    if key.value().0 == *id {
                        chunk_keys.push(key.value());
                    }
                }
                for k in chunk_keys {
                    snapshot_chunks
                        .remove(k)
                        .map_err(|e| AppError::Database(e.to_string()))?;
                }
                let mut sym_keys = Vec::new();
                for res in symbols
                    .iter()
                    .map_err(|e| AppError::Database(e.to_string()))?
                {
                    let (key, v) = res.map_err(|e| AppError::Database(e.to_string()))?;
                    let sym: SymbolData = bincode::deserialize(v.value())
                        .map_err(|e| AppError::Internal(e.to_string()))?;
                    if sym.snapshot_id == *id as i64 {
                        sym_keys.push(key.value());
                    }
                }
                for k in sym_keys {
                    symbols
                        .remove(k)
                        .map_err(|e| AppError::Database(e.to_string()))?;
                }
            }
        }
        write_txn
            .commit()
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(count)
    }

    pub fn get_snapshot_count(&self) -> AppResult<usize> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| AppError::Database(e.to_string()))?;
        let table = read_txn
            .open_table(SNAPSHOTS)
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(table.len().map_err(|e| AppError::Database(e.to_string()))? as usize)
    }

    pub fn get_symbol_count(&self) -> AppResult<usize> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| AppError::Database(e.to_string()))?;
        let table = read_txn
            .open_table(SYMBOLS)
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(table.len().map_err(|e| AppError::Database(e.to_string()))? as usize)
    }

    pub fn delete_all(&self) -> AppResult<usize> {
        let write_txn = self
            .db
            .begin_write()
            .map_err(|e| AppError::Database(e.to_string()))?;
        let count;
        {
            let mut s = write_txn
                .open_table(SNAPSHOTS)
                .map_err(|e| AppError::Database(e.to_string()))?;
            count = s.len().map_err(|e| AppError::Database(e.to_string()))? as usize;
            let snapshot_keys: Vec<u64> = s
                .iter()
                .map_err(|e| AppError::Database(e.to_string()))?
                .filter_map(|res| res.ok().map(|(k, _)| k.value()))
                .collect();
            for k in snapshot_keys {
                s.remove(k).map_err(|e| AppError::Database(e.to_string()))?;
            }
            let mut sc = write_txn
                .open_table(SNAPSHOT_CHUNKS)
                .map_err(|e| AppError::Database(e.to_string()))?;
            let chunk_keys: Vec<(u64, u32)> = sc
                .iter()
                .map_err(|e| AppError::Database(e.to_string()))?
                .filter_map(|res| res.ok().map(|(k, _)| k.value()))
                .collect();
            for k in chunk_keys {
                sc.remove(k)
                    .map_err(|e| AppError::Database(e.to_string()))?;
            }
            let mut sym = write_txn
                .open_table(SYMBOLS)
                .map_err(|e| AppError::Database(e.to_string()))?;
            let symbol_keys: Vec<u64> = sym
                .iter()
                .map_err(|e| AppError::Database(e.to_string()))?
                .filter_map(|res| res.ok().map(|(k, _)| k.value()))
                .collect();
            for k in symbol_keys {
                sym.remove(k)
                    .map_err(|e| AppError::Database(e.to_string()))?;
            }
        }
        write_txn
            .commit()
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(count)
    }

    pub fn vacuum(&self) -> AppResult<()> {
        Ok(())
    }

    pub fn get_snapshot_by_id(&self, id: i64) -> AppResult<Option<Snapshot>> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| AppError::Database(e.to_string()))?;
        let table = read_txn
            .open_table(SNAPSHOTS)
            .map_err(|e| AppError::Database(e.to_string()))?;
        if let Some(v) = table
            .get(id as u64)
            .map_err(|e| AppError::Database(e.to_string()))?
        {
            let data: SnapshotData =
                bincode::deserialize(v.value()).map_err(|e| AppError::Internal(e.to_string()))?;
            let path = self.lookup_string(data.file_path_id)?;
            let branch = if let Some(bid) = data.git_branch_id {
                Some(self.lookup_string(bid)?)
            } else {
                None
            };
            Ok(Some(Snapshot {
                id: data.id,
                file_path: path,
                timestamp: data.timestamp,
                content_hash: data.content_hash,
                git_branch: branch,
                session_id: data.session_id,
                commit_hash: data.commit_hash,
                commit_message: data.commit_message,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn get_latest_state(&self) -> AppResult<Vec<(String, String)>> {
        let snapshots = self.get_all_unique_snapshots()?;
        Ok(snapshots
            .into_iter()
            .map(|s| (s.file_path, s.content_hash))
            .collect())
    }

    pub fn get_state_at_timestamp(&self, timestamp: &str) -> AppResult<Vec<(String, String)>> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| AppError::Database(e.to_string()))?;
        let table = read_txn
            .open_table(SNAPSHOTS)
            .map_err(|e| AppError::Database(e.to_string()))?;
        let mut latest_per_file: std::collections::HashMap<u32, (String, u64)> =
            std::collections::HashMap::new();
        for res in table
            .iter()
            .map_err(|e| AppError::Database(e.to_string()))?
        {
            let (id, v) = res.map_err(|e| AppError::Database(e.to_string()))?;
            let data: SnapshotData =
                bincode::deserialize(v.value()).map_err(|e| AppError::Internal(e.to_string()))?;
            if data.timestamp.as_str() <= timestamp {
                let entry = latest_per_file
                    .entry(data.file_path_id)
                    .or_insert((data.content_hash.clone(), id.value()));
                if id.value() > entry.1 {
                    *entry = (data.content_hash, id.value());
                }
            }
        }
        let mut results = Vec::new();
        for (pid, (hash, _)) in latest_per_file {
            results.push((self.lookup_string(pid)?, hash));
        }
        Ok(results)
    }

    pub fn create_session(&self, start_time: &str, git_branch: Option<&str>) -> AppResult<i64> {
        let id = self.next_id("session_id")?;
        let git_branch_id = if let Some(b) = git_branch {
            Some(self.intern_string(b)?)
        } else {
            None
        };
        let write_txn = self
            .db
            .begin_write()
            .map_err(|e| AppError::Database(e.to_string()))?;
        {
            let mut table = write_txn
                .open_table(SESSIONS)
                .map_err(|e| AppError::Database(e.to_string()))?;
            let data = SessionData {
                id: id as i64,
                start_time: start_time.to_string(),
                end_time: None,
                git_branch_id,
                file_count: 0,
                snapshot_count: 0,
            };
            let bytes = bincode::serialize(&data).map_err(|e| AppError::Internal(e.to_string()))?;
            table
                .insert(id, &*bytes)
                .map_err(|e| AppError::Database(e.to_string()))?;
        }
        write_txn
            .commit()
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(id as i64)
    }

    pub fn close_session(
        &self,
        session_id: i64,
        end_time: &str,
        file_count: usize,
        snapshot_count: usize,
    ) -> AppResult<()> {
        let write_txn = self
            .db
            .begin_write()
            .map_err(|e| AppError::Database(e.to_string()))?;
        {
            let mut table = write_txn
                .open_table(SESSIONS)
                .map_err(|e| AppError::Database(e.to_string()))?;
            let data = if let Some(v) = table
                .get(session_id as u64)
                .map_err(|e| AppError::Database(e.to_string()))?
            {
                let mut data: SessionData = bincode::deserialize(v.value())
                    .map_err(|e| AppError::Internal(e.to_string()))?;
                data.end_time = Some(end_time.to_string());
                data.file_count = file_count;
                data.snapshot_count = snapshot_count;
                Some(data)
            } else {
                None
            };
            if let Some(d) = data {
                let bytes =
                    bincode::serialize(&d).map_err(|e| AppError::Internal(e.to_string()))?;
                table
                    .insert(session_id as u64, &*bytes)
                    .map_err(|e| AppError::Database(e.to_string()))?;
            }
        }
        write_txn
            .commit()
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(())
    }

    pub fn get_active_session(&self) -> AppResult<Option<Session>> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| AppError::Database(e.to_string()))?;
        let table = read_txn
            .open_table(SESSIONS)
            .map_err(|e| AppError::Database(e.to_string()))?;
        let mut active = None;
        for res in table
            .iter()
            .map_err(|e| AppError::Database(e.to_string()))?
        {
            let (_, v) = res.map_err(|e| AppError::Database(e.to_string()))?;
            let data: SessionData =
                bincode::deserialize(v.value()).map_err(|e| AppError::Internal(e.to_string()))?;
            if data.end_time.is_none() {
                let branch = if let Some(bid) = data.git_branch_id {
                    Some(self.lookup_string(bid)?)
                } else {
                    None
                };
                active = Some(Session {
                    id: data.id,
                    start_time: data.start_time,
                    end_time: data.end_time,
                    git_branch: branch,
                    file_count: data.file_count,
                    snapshot_count: data.snapshot_count,
                });
            }
        }
        Ok(active)
    }

    pub fn list_sessions(&self, limit: usize) -> AppResult<Vec<Session>> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| AppError::Database(e.to_string()))?;
        let table = read_txn
            .open_table(SESSIONS)
            .map_err(|e| AppError::Database(e.to_string()))?;
        let mut sessions = Vec::new();
        for res in table
            .iter()
            .map_err(|e| AppError::Database(e.to_string()))?
        {
            let (_, v) = res.map_err(|e| AppError::Database(e.to_string()))?;
            let data: SessionData =
                bincode::deserialize(v.value()).map_err(|e| AppError::Internal(e.to_string()))?;
            let branch = if let Some(bid) = data.git_branch_id {
                Some(self.lookup_string(bid)?)
            } else {
                None
            };
            sessions.push(Session {
                id: data.id,
                start_time: data.start_time,
                end_time: data.end_time,
                git_branch: branch,
                file_count: data.file_count,
                snapshot_count: data.snapshot_count,
            });
        }
        sessions.sort_by(|a, b| b.start_time.cmp(&a.start_time));
        sessions.truncate(limit);
        Ok(sessions)
    }

    pub fn insert_chunk(&self, hash: &str, kind: &str) -> AppResult<()> {
        let kind_id = self.intern_string(kind)?;
        let write_txn = self
            .db
            .begin_write()
            .map_err(|e| AppError::Database(e.to_string()))?;
        {
            let mut table = write_txn
                .open_table(CHUNKS)
                .map_err(|e| AppError::Database(e.to_string()))?;
            let data = ChunkData {
                hash: hash.to_string(),
                kind_id,
            };
            let bytes = bincode::serialize(&data).map_err(|e| AppError::Internal(e.to_string()))?;
            table
                .insert(hash, &*bytes)
                .map_err(|e| AppError::Database(e.to_string()))?;
        }
        write_txn
            .commit()
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(())
    }

    pub fn link_snapshot_chunk(
        &self,
        snapshot_id: i64,
        chunk_hash: &str,
        position: usize,
    ) -> AppResult<()> {
        let write_txn = self
            .db
            .begin_write()
            .map_err(|e| AppError::Database(e.to_string()))?;
        {
            let mut table = write_txn
                .open_table(SNAPSHOT_CHUNKS)
                .map_err(|e| AppError::Database(e.to_string()))?;
            table
                .insert((snapshot_id as u64, position as u32), chunk_hash)
                .map_err(|e| AppError::Database(e.to_string()))?;
        }
        write_txn
            .commit()
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(())
    }

    pub fn insert_symbol(&self, symbol: &SemanticSymbol) -> AppResult<i64> {
        let id = self.next_id("symbol_id")?;
        let name_id = self.intern_string(&symbol.name)?;
        let kind_id = self.intern_string(&symbol.kind)?;
        let scope_id = if let Some(s) = &symbol.scope {
            Some(self.intern_string(s)?)
        } else {
            None
        };
        let write_txn = self
            .db
            .begin_write()
            .map_err(|e| AppError::Database(e.to_string()))?;
        {
            let mut table = write_txn
                .open_table(SYMBOLS)
                .map_err(|e| AppError::Database(e.to_string()))?;
            let data = SymbolData {
                id: id as i64,
                name_id,
                kind_id,
                scope_id,
                snapshot_id: symbol.snapshot_id,
                chunk_hash: symbol.chunk_hash.clone(),
                structural_hash: symbol.structural_hash.clone(),
                start_line: symbol.start_line,
                end_line: symbol.end_line,
                start_byte: symbol.start_byte,
                end_byte: symbol.end_byte,
                parent_id: symbol.parent_id,
            };
            let bytes = bincode::serialize(&data).map_err(|e| AppError::Internal(e.to_string()))?;
            table
                .insert(id, &*bytes)
                .map_err(|e| AppError::Database(e.to_string()))?;
        }
        write_txn
            .commit()
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(id as i64)
    }

    pub fn insert_symbol_delta(&self, delta: &crate::models::SemanticRecord) -> AppResult<i64> {
        let id = self.next_id("delta_id")?;
        let symbol_name_id = self.intern_string(&delta.symbol_name)?;
        let new_name_id = if let Some(n) = &delta.new_name {
            Some(self.intern_string(n)?)
        } else {
            None
        };
        let kind_str = match delta.kind {
            crate::models::RecordKind::Added => "Added",
            crate::models::RecordKind::Modified => "Modified",
            crate::models::RecordKind::Deleted => "Deleted",
            crate::models::RecordKind::Renamed => "Renamed",
        };
        let write_txn = self
            .db
            .begin_write()
            .map_err(|e| AppError::Database(e.to_string()))?;
        {
            let mut table = write_txn
                .open_table(SYMBOL_DELTAS)
                .map_err(|e| AppError::Database(e.to_string()))?;
            let data = DeltaData {
                id: id as i64,
                from_snapshot_id: delta.from_snapshot_id,
                to_snapshot_id: delta.to_snapshot_id,
                symbol_name_id,
                new_name_id,
                delta_kind: kind_str.to_string(),
                structural_hash: delta.structural_hash.clone(),
            };
            let bytes = bincode::serialize(&data).map_err(|e| AppError::Internal(e.to_string()))?;
            table
                .insert(id, &*bytes)
                .map_err(|e| AppError::Database(e.to_string()))?;
        }
        write_txn
            .commit()
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(id as i64)
    }

    pub fn get_symbol_deltas(
        &self,
        symbol_name: &str,
    ) -> AppResult<Vec<crate::models::SemanticRecord>> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| AppError::Database(e.to_string()))?;
        let index = read_txn
            .open_table(STRING_INDEX)
            .map_err(|e| AppError::Database(e.to_string()))?;
        let target_id = match index
            .get(symbol_name)
            .map_err(|e| AppError::Database(e.to_string()))?
        {
            Some(id) => id.value(),
            None => return Ok(Vec::new()),
        };
        let table = read_txn
            .open_table(SYMBOL_DELTAS)
            .map_err(|e| AppError::Database(e.to_string()))?;
        let mut results = Vec::new();
        for res in table
            .iter()
            .map_err(|e| AppError::Database(e.to_string()))?
        {
            let (_, v) = res.map_err(|e| AppError::Database(e.to_string()))?;
            let data: DeltaData =
                bincode::deserialize(v.value()).map_err(|e| AppError::Internal(e.to_string()))?;
            if data.symbol_name_id == target_id || data.new_name_id == Some(target_id) {
                let kind = match data.delta_kind.as_str() {
                    "Added" => crate::models::RecordKind::Added,
                    "Modified" => crate::models::RecordKind::Modified,
                    "Deleted" => crate::models::RecordKind::Deleted,
                    "Renamed" => crate::models::RecordKind::Renamed,
                    _ => crate::models::RecordKind::Modified,
                };
                let name = self.lookup_string(data.symbol_name_id)?;
                let new_name = if let Some(nid) = data.new_name_id {
                    Some(self.lookup_string(nid)?)
                } else {
                    None
                };
                results.push(crate::models::SemanticRecord {
                    id: data.id,
                    project_id: None,
                    from_snapshot_id: data.from_snapshot_id,
                    to_snapshot_id: data.to_snapshot_id,
                    symbol_name: name,
                    new_name,
                    kind,
                    structural_hash: data.structural_hash,
                });
            }
        }
        results.sort_by(|a, b| b.to_snapshot_id.cmp(&a.to_snapshot_id));
        Ok(results)
    }

    pub fn insert_reference(&self, reference: &SymbolReference) -> AppResult<()> {
        let id = self.next_id("reference_id")?;
        let symbol_name_id = self.intern_string(&reference.symbol_name)?;
        let write_txn = self
            .db
            .begin_write()
            .map_err(|e| AppError::Database(e.to_string()))?;
        {
            let mut table = write_txn
                .open_table(SYMBOL_REFERENCES)
                .map_err(|e| AppError::Database(e.to_string()))?;
            let data = ReferenceData {
                id: id as i64,
                symbol_name_id,
                snapshot_id: reference.snapshot_id,
                start_line: reference.start_line,
                start_byte: reference.start_byte,
            };
            let bytes = bincode::serialize(&data).map_err(|e| AppError::Internal(e.to_string()))?;
            table
                .insert(id, &*bytes)
                .map_err(|e| AppError::Database(e.to_string()))?;
        }
        write_txn
            .commit()
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(())
    }

    pub fn batch_insert_semantic_data(
        &self,
        symbols: Vec<SemanticSymbol>,
        deltas: Vec<crate::models::SemanticRecord>,
        references: Vec<SymbolReference>,
    ) -> AppResult<()> {
        for s in &symbols {
            self.intern_string(&s.name)?;
            self.intern_string(&s.kind)?;
            if let Some(sc) = &s.scope {
                self.intern_string(sc)?;
            }
        }
        for d in &deltas {
            self.intern_string(&d.symbol_name)?;
            if let Some(n) = &d.new_name {
                self.intern_string(n)?;
            }
        }
        for r in &references {
            self.intern_string(&r.symbol_name)?;
        }
        for s in symbols {
            self.insert_symbol(&s)?;
        }
        for d in deltas {
            self.insert_symbol_delta(&d)?;
        }
        for r in references {
            self.insert_reference(&r)?;
        }
        Ok(())
    }

    pub fn get_symbol_history(
        &self,
        symbol_name: &str,
    ) -> AppResult<Vec<(Snapshot, SemanticSymbol)>> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| AppError::Database(e.to_string()))?;
        let index = read_txn
            .open_table(STRING_INDEX)
            .map_err(|e| AppError::Database(e.to_string()))?;
        let target_id = match index
            .get(symbol_name)
            .map_err(|e| AppError::Database(e.to_string()))?
        {
            Some(id) => id.value(),
            None => return Ok(Vec::new()),
        };
        let sym_table = read_txn
            .open_table(SYMBOLS)
            .map_err(|e| AppError::Database(e.to_string()))?;
        let snap_table = read_txn
            .open_table(SNAPSHOTS)
            .map_err(|e| AppError::Database(e.to_string()))?;
        let mut latest_hash = String::new();
        let mut max_sym_id = 0;
        for res in sym_table
            .iter()
            .map_err(|e| AppError::Database(e.to_string()))?
        {
            let (id, v) = res.map_err(|e| AppError::Database(e.to_string()))?;
            let data: SymbolData =
                bincode::deserialize(v.value()).map_err(|e| AppError::Internal(e.to_string()))?;
            if data.name_id == target_id && id.value() > max_sym_id {
                max_sym_id = id.value();
                latest_hash = data.structural_hash;
            }
        }
        let mut results = Vec::new();
        for res in sym_table
            .iter()
            .map_err(|e| AppError::Database(e.to_string()))?
        {
            let (_, v) = res.map_err(|e| AppError::Database(e.to_string()))?;
            let sym_data: SymbolData =
                bincode::deserialize(v.value()).map_err(|e| AppError::Internal(e.to_string()))?;
            if sym_data.structural_hash == latest_hash || sym_data.name_id == target_id {
                if let Some(sv) = snap_table
                    .get(sym_data.snapshot_id as u64)
                    .map_err(|e| AppError::Database(e.to_string()))?
                {
                    let snap_data: SnapshotData = bincode::deserialize(sv.value())
                        .map_err(|e| AppError::Internal(e.to_string()))?;
                    let path = self.lookup_string(snap_data.file_path_id)?;
                    let branch = if let Some(bid) = snap_data.git_branch_id {
                        Some(self.lookup_string(bid)?)
                    } else {
                        None
                    };
                    let snap = Snapshot {
                        id: snap_data.id,
                        file_path: path,
                        timestamp: snap_data.timestamp,
                        content_hash: snap_data.content_hash,
                        git_branch: branch,
                        session_id: snap_data.session_id,
                        commit_hash: snap_data.commit_hash,
                        commit_message: snap_data.commit_message,
                    };
                    let name = self.lookup_string(sym_data.name_id)?;
                    let kind = self.lookup_string(sym_data.kind_id)?;
                    let scope = if let Some(sid) = sym_data.scope_id {
                        Some(self.lookup_string(sid)?)
                    } else {
                        None
                    };
                    let sym = SemanticSymbol {
                        id: sym_data.id,
                        name,
                        kind,
                        scope,
                        snapshot_id: sym_data.snapshot_id,
                        chunk_hash: sym_data.chunk_hash,
                        structural_hash: sym_data.structural_hash,
                        start_line: sym_data.start_line,
                        end_line: sym_data.end_line,
                        start_byte: sym_data.start_byte,
                        end_byte: sym_data.end_byte,
                        parent_id: sym_data.parent_id,
                    };
                    results.push((snap, sym));
                }
            }
        }
        results.sort_by(|a, b| b.0.timestamp.cmp(&a.0.timestamp));
        Ok(results)
    }

    pub fn get_chunks_for_hash(&self, content_hash: &str) -> AppResult<Vec<String>> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| AppError::Database(e.to_string()))?;
        let snap_table = read_txn
            .open_table(SNAPSHOTS)
            .map_err(|e| AppError::Database(e.to_string()))?;
        let sc_table = read_txn
            .open_table(SNAPSHOT_CHUNKS)
            .map_err(|e| AppError::Database(e.to_string()))?;
        let mut snapshot_id = None;
        for res in snap_table
            .iter()
            .map_err(|e| AppError::Database(e.to_string()))?
        {
            let (id, v) = res.map_err(|e| AppError::Database(e.to_string()))?;
            let data: SnapshotData =
                bincode::deserialize(v.value()).map_err(|e| AppError::Internal(e.to_string()))?;
            if data.content_hash == content_hash {
                snapshot_id = Some(id.value());
                break;
            }
        }
        if let Some(sid) = snapshot_id {
            let mut chunks = Vec::new();
            for res in sc_table
                .iter()
                .map_err(|e| AppError::Database(e.to_string()))?
            {
                let (k, v) = res.map_err(|e| AppError::Database(e.to_string()))?;
                if k.value().0 == sid {
                    chunks.push((k.value().1, v.value().to_string()));
                }
            }
            chunks.sort_by_key(|a| a.0);
            Ok(chunks.into_iter().map(|(_, h)| h).collect())
        } else {
            Ok(Vec::new())
        }
    }

    pub fn get_symbols_for_snapshot(&self, snapshot_id: i64) -> AppResult<Vec<SemanticSymbol>> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| AppError::Database(e.to_string()))?;
        let table = read_txn
            .open_table(SYMBOLS)
            .map_err(|e| AppError::Database(e.to_string()))?;
        let mut results = Vec::new();
        for res in table
            .iter()
            .map_err(|e| AppError::Database(e.to_string()))?
        {
            let (_, v) = res.map_err(|e| AppError::Database(e.to_string()))?;
            let data: SymbolData =
                bincode::deserialize(v.value()).map_err(|e| AppError::Internal(e.to_string()))?;
            if data.snapshot_id == snapshot_id {
                let name = self.lookup_string(data.name_id)?;
                let kind = self.lookup_string(data.kind_id)?;
                let scope = if let Some(sid) = data.scope_id {
                    Some(self.lookup_string(sid)?)
                } else {
                    None
                };
                results.push(SemanticSymbol {
                    id: data.id,
                    name,
                    kind,
                    scope,
                    snapshot_id: data.snapshot_id,
                    chunk_hash: data.chunk_hash,
                    structural_hash: data.structural_hash,
                    start_line: data.start_line,
                    end_line: data.end_line,
                    start_byte: data.start_byte,
                    end_byte: data.end_byte,
                    parent_id: data.parent_id,
                });
            }
        }
        results.sort_by_key(|s| s.start_byte);
        Ok(results)
    }

    pub fn find_symbols_by_name(&self, query: &str) -> AppResult<Vec<SemanticSymbol>> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| AppError::Database(e.to_string()))?;
        let table = read_txn
            .open_table(SYMBOLS)
            .map_err(|e| AppError::Database(e.to_string()))?;
        let mut results = Vec::new();
        let query_lower = query.to_lowercase();
        for res in table
            .iter()
            .map_err(|e| AppError::Database(e.to_string()))?
        {
            let (_, v) = res.map_err(|e| AppError::Database(e.to_string()))?;
            let data: SymbolData =
                bincode::deserialize(v.value()).map_err(|e| AppError::Internal(e.to_string()))?;
            let name = self.lookup_string(data.name_id)?;
            if name.to_lowercase().contains(&query_lower) {
                let kind = self.lookup_string(data.kind_id)?;
                let scope = if let Some(sid) = data.scope_id {
                    Some(self.lookup_string(sid)?)
                } else {
                    None
                };
                results.push(SemanticSymbol {
                    id: data.id,
                    name,
                    kind,
                    scope,
                    snapshot_id: data.snapshot_id,
                    chunk_hash: data.chunk_hash,
                    structural_hash: data.structural_hash,
                    start_line: data.start_line,
                    end_line: data.end_line,
                    start_byte: data.start_byte,
                    end_byte: data.end_byte,
                    parent_id: data.parent_id,
                });
            }
            if results.len() >= 100 {
                break;
            }
        }
        Ok(results)
    }

    pub fn get_file_count(&self) -> AppResult<usize> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| AppError::Database(e.to_string()))?;
        let table = read_txn
            .open_table(SNAPSHOTS)
            .map_err(|e| AppError::Database(e.to_string()))?;
        let mut paths = HashSet::new();
        for res in table
            .iter()
            .map_err(|e| AppError::Database(e.to_string()))?
        {
            let (_, v) = res.map_err(|e| AppError::Database(e.to_string()))?;
            let data: SnapshotData =
                bincode::deserialize(v.value()).map_err(|e| AppError::Internal(e.to_string()))?;
            paths.insert(data.file_path_id);
        }
        Ok(paths.len())
    }

    pub fn get_commit_count(&self) -> AppResult<usize> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| AppError::Database(e.to_string()))?;
        let table = read_txn
            .open_table(GIT_COMMITS)
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(table.len().map_err(|e| AppError::Database(e.to_string()))? as usize)
    }

    pub fn get_top_files(&self, limit: usize) -> AppResult<Vec<(String, usize)>> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| AppError::Database(e.to_string()))?;
        let table = read_txn
            .open_table(SNAPSHOTS)
            .map_err(|e| AppError::Database(e.to_string()))?;
        let mut files = std::collections::HashMap::new();
        for res in table
            .iter()
            .map_err(|e| AppError::Database(e.to_string()))?
        {
            let (_, v) = res.map_err(|e| AppError::Database(e.to_string()))?;
            let data: SnapshotData =
                bincode::deserialize(v.value()).map_err(|e| AppError::Internal(e.to_string()))?;
            *files.entry(data.file_path_id).or_insert(0) += 1;
        }
        let mut results = Vec::new();
        for (pid, count) in files {
            results.push((self.lookup_string(pid)?, count));
        }
        results.sort_by(|a, b| b.1.cmp(&a.1));
        results.truncate(limit);
        Ok(results)
    }

    pub fn get_top_branches(&self, limit: usize) -> AppResult<Vec<(String, usize)>> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| AppError::Database(e.to_string()))?;
        let table = read_txn
            .open_table(SNAPSHOTS)
            .map_err(|e| AppError::Database(e.to_string()))?;
        let mut branches = std::collections::HashMap::new();
        for res in table
            .iter()
            .map_err(|e| AppError::Database(e.to_string()))?
        {
            let (_, v) = res.map_err(|e| AppError::Database(e.to_string()))?;
            let data: SnapshotData =
                bincode::deserialize(v.value()).map_err(|e| AppError::Internal(e.to_string()))?;
            if let Some(bid) = data.git_branch_id {
                *branches.entry(bid).or_insert(0) += 1;
            }
        }
        let mut results = Vec::new();
        for (bid, count) in branches {
            results.push((self.lookup_string(bid)?, count));
        }
        results.sort_by(|a, b| b.1.cmp(&a.1));
        results.truncate(limit);
        Ok(results)
    }

    pub fn get_extension_distribution(&self) -> AppResult<Vec<(String, usize)>> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| AppError::Database(e.to_string()))?;
        let table = read_txn
            .open_table(SNAPSHOTS)
            .map_err(|e| AppError::Database(e.to_string()))?;
        let mut exts = std::collections::HashMap::new();
        for res in table
            .iter()
            .map_err(|e| AppError::Database(e.to_string()))?
        {
            let (_, v) = res.map_err(|e| AppError::Database(e.to_string()))?;
            let data: SnapshotData =
                bincode::deserialize(v.value()).map_err(|e| AppError::Internal(e.to_string()))?;
            let path = self.lookup_string(data.file_path_id)?;
            let ext = std::path::Path::new(&path)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("no extension")
                .to_lowercase();
            *exts.entry(ext).or_insert(0) += 1;
        }
        let mut results: Vec<_> = exts.into_iter().collect();
        results.sort_by(|a, b| b.1.cmp(&a.1));
        results.truncate(15);
        Ok(results)
    }

    pub fn get_commits(&self) -> AppResult<Vec<(String, String, String, String, usize)>> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| AppError::Database(e.to_string()))?;
        let commits_table = read_txn
            .open_table(GIT_COMMITS)
            .map_err(|e| AppError::Database(e.to_string()))?;
        let snapshots_table = read_txn
            .open_table(SNAPSHOTS)
            .map_err(|e| AppError::Database(e.to_string()))?;
        let mut snapshot_counts: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();
        for res in snapshots_table
            .iter()
            .map_err(|e| AppError::Database(e.to_string()))?
        {
            let (_, v) = res.map_err(|e| AppError::Database(e.to_string()))?;
            let data: SnapshotData =
                bincode::deserialize(v.value()).map_err(|e| AppError::Internal(e.to_string()))?;
            if let Some(ch) = data.commit_hash {
                *snapshot_counts.entry(ch).or_insert(0) += 1;
            }
        }
        let mut results = Vec::new();
        for res in commits_table
            .iter()
            .map_err(|e| AppError::Database(e.to_string()))?
        {
            let (_, v) = res.map_err(|e| AppError::Database(e.to_string()))?;
            let data: GitCommitData =
                bincode::deserialize(v.value()).map_err(|e| AppError::Internal(e.to_string()))?;
            let count = *snapshot_counts.get(&data.hash).unwrap_or(&0);
            results.push((data.hash, data.message, data.author, data.timestamp, count));
        }
        results.sort_by(|a, b| b.3.cmp(&a.3));
        Ok(results)
    }

    pub fn get_commit_by_hash(
        &self,
        hash: &str,
    ) -> AppResult<Option<(String, String, String, String)>> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| AppError::Database(e.to_string()))?;
        let table = read_txn
            .open_table(GIT_COMMITS)
            .map_err(|e| AppError::Database(e.to_string()))?;
        if let Some(v) = table
            .get(hash)
            .map_err(|e| AppError::Database(e.to_string()))?
        {
            let data: GitCommitData =
                bincode::deserialize(v.value()).map_err(|e| AppError::Internal(e.to_string()))?;
            Ok(Some((data.hash, data.message, data.author, data.timestamp)))
        } else {
            Ok(None)
        }
    }

    pub fn get_commit_files(&self, hash: &str) -> AppResult<Vec<(String, String, String)>> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| AppError::Database(e.to_string()))?;
        let table = read_txn
            .open_table(SNAPSHOTS)
            .map_err(|e| AppError::Database(e.to_string()))?;
        let mut files = Vec::new();
        for res in table
            .iter()
            .map_err(|e| AppError::Database(e.to_string()))?
        {
            let (_, v) = res.map_err(|e| AppError::Database(e.to_string()))?;
            let data: SnapshotData =
                bincode::deserialize(v.value()).map_err(|e| AppError::Internal(e.to_string()))?;
            if data.commit_hash.as_deref() == Some(hash) {
                let path = self.lookup_string(data.file_path_id)?;
                files.push((path, data.content_hash, data.timestamp));
            }
        }
        files.sort_by(|a, b| a.2.cmp(&b.2));
        Ok(files)
    }

    pub fn get_checkpoint_by_hash(
        &self,
        hash_query: &str,
    ) -> AppResult<Option<(String, String, Option<String>)>> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| AppError::Database(e.to_string()))?;
        let table = read_txn
            .open_table(CHECKPOINTS)
            .map_err(|e| AppError::Database(e.to_string()))?;
        let query = hash_query.to_lowercase();
        for res in table
            .iter()
            .map_err(|e| AppError::Database(e.to_string()))?
        {
            let (k, v) = res.map_err(|e| AppError::Database(e.to_string()))?;
            if k.value().to_lowercase().starts_with(&query) {
                let data: CheckpointData = bincode::deserialize(v.value())
                    .map_err(|e| AppError::Internal(e.to_string()))?;
                return Ok(Some((data.timestamp, data.file_states, data.description)));
            }
        }
        Ok(None)
    }

    pub fn save_checkpoint(
        &self,
        timestamp: &str,
        description: Option<&str>,
        file_states_json: &str,
    ) -> AppResult<String> {
        let mut hasher = blake3::Hasher::new();
        hasher.update(timestamp.as_bytes());
        hasher.update(file_states_json.as_bytes());
        if let Some(d) = description {
            hasher.update(d.as_bytes());
        }
        let hash = hasher.finalize().to_hex().to_string();
        let write_txn = self
            .db
            .begin_write()
            .map_err(|e| AppError::Database(e.to_string()))?;
        {
            let mut table = write_txn
                .open_table(CHECKPOINTS)
                .map_err(|e| AppError::Database(e.to_string()))?;
            let data = CheckpointData {
                hash: hash.clone(),
                timestamp: timestamp.to_string(),
                description: description.map(|s| s.to_string()),
                file_states: file_states_json.to_string(),
            };
            let bytes = bincode::serialize(&data).map_err(|e| AppError::Internal(e.to_string()))?;
            table
                .insert(hash.as_str(), &*bytes)
                .map_err(|e| AppError::Database(e.to_string()))?;
        }
        write_txn
            .commit()
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(hash)
    }

    pub fn list_checkpoints(&self) -> AppResult<Vec<(String, String, Option<String>)>> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| AppError::Database(e.to_string()))?;
        let table = read_txn
            .open_table(CHECKPOINTS)
            .map_err(|e| AppError::Database(e.to_string()))?;
        let mut results = Vec::new();
        for res in table
            .iter()
            .map_err(|e| AppError::Database(e.to_string()))?
        {
            let (_, v) = res.map_err(|e| AppError::Database(e.to_string()))?;
            let data: CheckpointData =
                bincode::deserialize(v.value()).map_err(|e| AppError::Internal(e.to_string()))?;
            results.push((data.hash, data.timestamp, data.description));
        }
        results.sort_by(|a, b| b.1.cmp(&a.1));
        Ok(results)
    }

    pub fn delete_checkpoint(&self, hash: &str) -> AppResult<bool> {
        let write_txn = self
            .db
            .begin_write()
            .map_err(|e| AppError::Database(e.to_string()))?;
        let found;
        {
            let mut table = write_txn
                .open_table(CHECKPOINTS)
                .map_err(|e| AppError::Database(e.to_string()))?;
            found = table
                .remove(hash)
                .map_err(|e| AppError::Database(e.to_string()))?
                .is_some();
        }
        write_txn
            .commit()
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(found)
    }

    pub fn update_chunk_trigrams(&self, chunk_hash: &str, content: &[u8]) -> AppResult<()> {
        let mut trigrams = HashSet::new();
        if content.len() >= 3 {
            for i in 0..content.len() - 2 {
                let trigram = (content[i] as u64) << 16
                    | (content[i + 1] as u64) << 8
                    | (content[i + 2] as u64);
                trigrams.insert(trigram);
            }
        }
        let mut bloom: u64 = 0;
        for t in trigrams {
            let h = blake3::hash(&t.to_be_bytes());
            let bit = (h.as_bytes()[0] as u64) % 64;
            bloom |= 1 << bit;
        }
        let write_txn = self
            .db
            .begin_write()
            .map_err(|e| AppError::Database(e.to_string()))?;
        {
            let mut table = write_txn
                .open_table(CHUNK_TRIGRAMS)
                .map_err(|e| AppError::Database(e.to_string()))?;
            table
                .insert(chunk_hash, bloom)
                .map_err(|e| AppError::Database(e.to_string()))?;
        }
        write_txn
            .commit()
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(())
    }

    pub fn filter_chunks_by_trigrams(&self, query: &str) -> AppResult<HashSet<String>> {
        if query.len() < 3 {
            return Ok(HashSet::new());
        }
        let mut query_bloom: u64 = 0;
        let bytes = query.as_bytes();
        for i in 0..bytes.len() - 2 {
            let trigram =
                (bytes[i] as u64) << 16 | (bytes[i + 1] as u64) << 8 | (bytes[i + 2] as u64);
            let h = blake3::hash(&trigram.to_be_bytes());
            let bit = (h.as_bytes()[0] as u64) % 64;
            query_bloom |= 1 << bit;
        }
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| AppError::Database(e.to_string()))?;
        let table = read_txn
            .open_table(CHUNK_TRIGRAMS)
            .map_err(|e| AppError::Database(e.to_string()))?;
        let mut matches = HashSet::new();
        for res in table
            .iter()
            .map_err(|e| AppError::Database(e.to_string()))?
        {
            let (hash, bloom) = res.map_err(|e| AppError::Database(e.to_string()))?;
            if (bloom.value() & query_bloom) == query_bloom {
                matches.insert(hash.value().to_string());
            }
        }
        Ok(matches)
    }
}
