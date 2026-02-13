use crate::error::{AppError, AppResult};
use crate::models::{
    FileEntry, SemanticSymbol, Session, Snapshot, SymbolReference, TimesheetEntry,
};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params, Connection};
use std::collections::HashSet;
use std::path::PathBuf;

pub struct Database {
    pub pool: Pool<SqliteConnectionManager>,
    pub path: PathBuf,
}

impl Database {
    pub fn new(path: PathBuf) -> AppResult<Self> {
        let manager = SqliteConnectionManager::file(&path);
        let pool = Pool::builder()
            .max_size(10) // Allow up to 10 concurrent connections
            .build(manager)
            .map_err(|e| AppError::Internal(format!("Database pool error: {}", e)))?;

        let conn = pool
            .get()
            .map_err(|e| AppError::Internal(format!("Database connection error: {}", e)))?;

        conn.pragma_update(None, "journal_mode", "WAL")
            .map_err(AppError::Database)?;
        conn.pragma_update(None, "synchronous", "NORMAL")
            .map_err(AppError::Database)?;
        conn.pragma_update(None, "foreign_keys", "ON")
            .map_err(AppError::Database)?;
        conn.pragma_update(None, "busy_timeout", "5000")
            .map_err(AppError::Database)?;

        // ... (rest of the table creation remains the same, using `conn` variable)

        // --- Core tables ---

        conn.execute(
            "CREATE TABLE IF NOT EXISTS snapshots (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                file_path TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                content_hash TEXT NOT NULL,
                git_branch TEXT,
                session_id INTEGER,
                commit_hash TEXT,
                FOREIGN KEY(session_id) REFERENCES sessions(id)
            )",
            [],
        )
        .map_err(AppError::Database)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS git_commits (
                hash TEXT PRIMARY KEY,
                message TEXT,
                author TEXT,
                timestamp TEXT
            )",
            [],
        )
        .map_err(AppError::Database)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS sessions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                start_time TEXT NOT NULL,
                end_time TEXT,
                git_branch TEXT,
                file_count INTEGER DEFAULT 0,
                snapshot_count INTEGER DEFAULT 0
            )",
            [],
        )
        .map_err(AppError::Database)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS project_checkpoints (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                hash TEXT UNIQUE,
                timestamp TEXT NOT NULL,
                description TEXT,
                file_states TEXT NOT NULL
            )",
            [],
        )
        .map_err(AppError::Database)?;

        // --- SHP (Semantic History Protocol) Extension ---

        conn.execute(
            "CREATE TABLE IF NOT EXISTS chunks (
                hash TEXT PRIMARY KEY,
                kind TEXT NOT NULL
            )",
            [],
        )
        .map_err(AppError::Database)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS snapshot_chunks (
                snapshot_id INTEGER,
                chunk_hash TEXT,
                position INTEGER,
                PRIMARY KEY (snapshot_id, position),
                FOREIGN KEY (snapshot_id) REFERENCES snapshots(id) ON DELETE CASCADE,
                FOREIGN KEY (chunk_hash) REFERENCES chunks(hash) ON DELETE CASCADE
            )",
            [],
        )
        .map_err(AppError::Database)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS symbols (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        name TEXT NOT NULL,
                        kind TEXT NOT NULL,
                        scope TEXT,
                        snapshot_id INTEGER,
                        chunk_hash TEXT,
                        structural_hash TEXT,
                        start_line INTEGER,
                        end_line INTEGER,
                        start_byte INTEGER,
                        end_byte INTEGER,
                        parent_id INTEGER,
                        FOREIGN KEY(snapshot_id) REFERENCES snapshots(id) ON DELETE CASCADE,
                        FOREIGN KEY(chunk_hash) REFERENCES chunks(hash) ON DELETE CASCADE,
                        FOREIGN KEY(parent_id) REFERENCES symbols(id) ON DELETE CASCADE
                    )",
            [],
        )
        .map_err(AppError::Database)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS symbol_references (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        symbol_name TEXT NOT NULL,
                        snapshot_id INTEGER,
                        start_line INTEGER,
                        start_byte INTEGER,
                        FOREIGN KEY(snapshot_id) REFERENCES snapshots(id) ON DELETE CASCADE
                    )",
            [],
        )
        .map_err(AppError::Database)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS symbol_deltas (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                from_snapshot_id INTEGER,
                to_snapshot_id INTEGER NOT NULL,
                symbol_name TEXT NOT NULL,
                new_name TEXT,
                delta_kind TEXT NOT NULL,
                structural_hash TEXT NOT NULL,
                FOREIGN KEY(from_snapshot_id) REFERENCES snapshots(id) ON DELETE CASCADE,
                FOREIGN KEY(to_snapshot_id) REFERENCES snapshots(id) ON DELETE CASCADE
            )",
            [],
        )
        .map_err(AppError::Database)?;

        // --- Indexes ---

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_file_path ON snapshots(file_path)",
            [],
        )
        .map_err(AppError::Database)?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_content_hash ON snapshots(content_hash)",
            [],
        )
        .map_err(AppError::Database)?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_history ON snapshots(file_path, id DESC)",
            [],
        )
        .map_err(AppError::Database)?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_snapshot_session ON snapshots(session_id)",
            [],
        )
        .map_err(AppError::Database)?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_session_time ON sessions(start_time DESC)",
            [],
        )
        .map_err(AppError::Database)?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_symbol_struct ON symbols(structural_hash)",
            [],
        )
        .map_err(AppError::Database)?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_ref_name ON symbol_references(symbol_name)",
            [],
        )
        .map_err(AppError::Database)?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_delta_symbol ON symbol_deltas(symbol_name)",
            [],
        )
        .map_err(AppError::Database)?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_delta_struct ON symbol_deltas(structural_hash)",
            [],
        )
        .map_err(AppError::Database)?;

        // --- Migrations (safe to re-run) ---
        let _ = conn.execute("ALTER TABLE snapshots ADD COLUMN git_branch TEXT", []);
        let _ = conn.execute("ALTER TABLE snapshots ADD COLUMN session_id INTEGER", []);
        let _ = conn.execute("ALTER TABLE snapshots ADD COLUMN commit_hash TEXT", []);
        let _ = conn.execute("ALTER TABLE symbols ADD COLUMN structural_hash TEXT", []);
        let _ = conn.execute("ALTER TABLE symbols ADD COLUMN scope TEXT", []);
        let _ = conn.execute("ALTER TABLE project_checkpoints ADD COLUMN hash TEXT", []);
        let _ = conn.execute(
            "CREATE UNIQUE INDEX IF NOT EXISTS idx_checkpoint_hash ON project_checkpoints(hash)",
            [],
        );

        Ok(Self { pool, path })
    }

    /// Acquires a connection from the pool.
    fn conn(&self) -> r2d2::PooledConnection<SqliteConnectionManager> {
        self.pool.get().expect("Database connection pool exhausted")
    }

    pub fn get_git_commit(&self, hash: &str) -> AppResult<Option<(String, String, String)>> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare("SELECT message, author, timestamp FROM git_commits WHERE hash = ?1")
            .map_err(AppError::Database)?;
        let mut rows = stmt
            .query_map([hash], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
            .map_err(AppError::Database)?;
        if let Some(row) = rows.next() {
            Ok(Some(row.map_err(AppError::Database)?))
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
        let conn = self.conn();
        conn.execute(
            "INSERT OR IGNORE INTO git_commits (hash, message, author, timestamp) VALUES (?1, ?2, ?3, ?4)",
            params![hash, message, author, timestamp],
        )
        .map_err(AppError::Database)?;
        Ok(())
    }

    pub fn link_snapshot_to_commit(&self, snapshot_id: i64, commit_hash: &str) -> AppResult<()> {
        let conn = self.conn();
        conn.execute(
            "UPDATE snapshots SET commit_hash = ?1 WHERE id = ?2",
            params![commit_hash, snapshot_id],
        )
        .map_err(AppError::Database)?;
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
        let conn = self.conn();
        conn.execute(
            "INSERT INTO snapshots (file_path, timestamp, content_hash, git_branch, session_id) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![file_path, timestamp, content_hash, git_branch, session_id],
        )
        .map_err(AppError::Database)?;
        Ok(conn.last_insert_rowid())
    }

    pub fn get_last_hash(&self, file_path: &str) -> AppResult<Option<String>> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare(
                "SELECT content_hash FROM snapshots WHERE file_path = ?1 ORDER BY id DESC LIMIT 1",
            )
            .map_err(AppError::Database)?;
        let mut rows = stmt
            .query_map([file_path], |row| row.get(0))
            .map_err(AppError::Database)?;
        if let Some(row) = rows.next() {
            Ok(Some(row.map_err(AppError::Database)?))
        } else {
            Ok(None)
        }
    }

    pub fn get_history(&self, file_path: &str) -> AppResult<Vec<Snapshot>> {
        let conn = self.conn();
        let pattern_win = format!("%{}%", file_path.replace('/', "\\"));
        let pattern_unix = format!("%{}%", file_path.replace('\\', "/"));
        let mut stmt = conn
            .prepare(
                "SELECT id, file_path, timestamp, content_hash, git_branch, session_id, commit_hash
             FROM snapshots WHERE file_path LIKE ?1 ESCAPE '/' OR file_path LIKE ?2 ESCAPE '/' ORDER BY id DESC",
            )
            .map_err(AppError::Database)?;

        let rows = stmt
            .query_map([&pattern_win, &pattern_unix], |row| {
                Ok(Snapshot {
                    id: row.get(0)?,
                    file_path: row.get(1)?,
                    timestamp: row.get(2)?,
                    content_hash: row.get(3)?,
                    git_branch: row.get(4)?,
                    session_id: row.get(5)?,
                    commit_hash: row.get(6)?,
                })
            })
            .map_err(AppError::Database)?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row.map_err(AppError::Database)?);
        }
        Ok(results)
    }

    pub fn get_global_history(&self, limit: usize) -> AppResult<Vec<Snapshot>> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare(
                "SELECT id, file_path, timestamp, content_hash, git_branch, session_id, commit_hash
             FROM snapshots ORDER BY id DESC LIMIT ?1",
            )
            .map_err(AppError::Database)?;

        let rows = stmt
            .query_map([limit], |row| {
                Ok(Snapshot {
                    id: row.get(0)?,
                    file_path: row.get(1)?,
                    timestamp: row.get(2)?,
                    content_hash: row.get(3)?,
                    git_branch: row.get(4)?,
                    session_id: row.get(5)?,
                    commit_hash: row.get(6)?,
                })
            })
            .map_err(AppError::Database)?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row.map_err(AppError::Database)?);
        }
        Ok(results)
    }

    pub fn get_history_by_hash(&self, content_hash: &str) -> AppResult<Vec<Snapshot>> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare(
                "SELECT id, file_path, timestamp, content_hash, git_branch, session_id, commit_hash
             FROM snapshots WHERE content_hash = ?1 ORDER BY id DESC",
            )
            .map_err(AppError::Database)?;

        let rows = stmt
            .query_map([content_hash], |row| {
                Ok(Snapshot {
                    id: row.get(0)?,
                    file_path: row.get(1)?,
                    timestamp: row.get(2)?,
                    content_hash: row.get(3)?,
                    git_branch: row.get(4)?,
                    session_id: row.get(5)?,
                    commit_hash: row.get(6)?,
                })
            })
            .map_err(AppError::Database)?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row.map_err(AppError::Database)?);
        }
        Ok(results)
    }

    pub fn get_max_snapshot_id(&self) -> AppResult<i64> {
        let conn = self.conn();
        let id: i64 = conn
            .query_row("SELECT COALESCE(MAX(id), 0) FROM snapshots", [], |row| {
                row.get(0)
            })
            .map_err(AppError::Database)?;
        Ok(id)
    }

    pub fn get_recent_files(
        &self,
        limit: usize,
        filter: Option<&str>,
        branch: Option<&str>,
    ) -> AppResult<Vec<FileEntry>> {
        let conn = self.conn();
        let mut conditions = Vec::new();
        if let Some(b) = branch {
            conditions.push(format!("git_branch = '{}'", b));
        }
        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!(" WHERE {}", conditions.join(" AND "))
        };
        let mut query = format!(
            "SELECT file_path, MAX(timestamp) FROM snapshots{} GROUP BY file_path",
            where_clause
        );
        if let Some(f) = filter {
            query.push_str(&format!(" HAVING file_path LIKE '%{}%'", f));
        }
        query.push_str(" ORDER BY MAX(timestamp) DESC LIMIT ?1");
        let mut stmt = conn.prepare(&query).map_err(AppError::Database)?;
        let rows = stmt
            .query_map([limit as i64], |row| {
                Ok(FileEntry {
                    path: row.get(0)?,
                    last_update: row.get(1)?,
                })
            })
            .map_err(AppError::Database)?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row.map_err(AppError::Database)?);
        }
        Ok(results)
    }

    pub fn get_distinct_branches(&self) -> AppResult<Vec<String>> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare("SELECT DISTINCT git_branch FROM snapshots WHERE git_branch IS NOT NULL ORDER BY git_branch")
            .map_err(AppError::Database)?;
        let rows = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(AppError::Database)?;
        let mut branches = Vec::new();
        for row in rows {
            branches.push(row.map_err(AppError::Database)?);
        }
        Ok(branches)
    }

    pub fn get_recent_activity(&self, limit: usize) -> AppResult<Vec<Snapshot>> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare("SELECT id, file_path, timestamp, content_hash, git_branch, session_id, commit_hash FROM snapshots ORDER BY id DESC LIMIT ?1")
            .map_err(AppError::Database)?;
        let rows = stmt
            .query_map([limit as i64], |row| {
                Ok(Snapshot {
                    id: row.get(0)?,
                    file_path: row.get(1)?,
                    timestamp: row.get(2)?,
                    content_hash: row.get(3)?,
                    git_branch: row.get(4)?,
                    session_id: row.get(5)?,
                    commit_hash: row.get(6)?,
                })
            })
            .map_err(AppError::Database)?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row.map_err(AppError::Database)?);
        }
        Ok(results)
    }

    pub fn resolve_hash(&self, short_hash: &str) -> AppResult<Option<String>> {
        let short_hash = short_hash.to_lowercase();
        let conn = self.conn();
        let mut stmt = conn
            .prepare(
                "SELECT DISTINCT content_hash FROM snapshots WHERE LOWER(content_hash) LIKE ?1 LIMIT 2",
            )
            .map_err(AppError::Database)?;
        let mut rows = stmt
            .query([format!("{}%", short_hash)])
            .map_err(AppError::Database)?;
        let mut results = Vec::new();
        while let Some(row) = rows.next().map_err(AppError::Database)? {
            results.push(row.get::<_, String>(0)?);
        }
        if results.len() == 1 {
            Ok(Some(results[0].clone()))
        } else {
            Ok(None)
        }
    }

    pub fn get_all_unique_snapshots(&self) -> AppResult<Vec<Snapshot>> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare("SELECT id, file_path, timestamp, content_hash, git_branch, session_id, commit_hash FROM snapshots GROUP BY file_path ORDER BY timestamp DESC")
            .map_err(AppError::Database)?;
        let rows = stmt
            .query_map([], |row| {
                Ok(Snapshot {
                    id: row.get(0)?,
                    file_path: row.get(1)?,
                    timestamp: row.get(2)?,
                    content_hash: row.get(3)?,
                    git_branch: row.get(4)?,
                    session_id: row.get(5)?,
                    commit_hash: row.get(6)?,
                })
            })
            .map_err(AppError::Database)?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row.map_err(AppError::Database)?);
        }
        Ok(results)
    }

    /// Returns all snapshots deduplicated by content_hash.
    /// Each unique content blob appears once, keeping the most recent metadata.
    pub fn get_all_snapshots_deduped(&self) -> AppResult<Vec<Snapshot>> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare(
                "SELECT id, file_path, timestamp, content_hash, git_branch, session_id, commit_hash
                 FROM snapshots
                 GROUP BY content_hash
                 ORDER BY timestamp DESC",
            )
            .map_err(AppError::Database)?;
        let rows = stmt
            .query_map([], |row| {
                Ok(Snapshot {
                    id: row.get(0)?,
                    file_path: row.get(1)?,
                    timestamp: row.get(2)?,
                    content_hash: row.get(3)?,
                    git_branch: row.get(4)?,
                    session_id: row.get(5)?,
                    commit_hash: row.get(6)?,
                })
            })
            .map_err(AppError::Database)?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row.map_err(AppError::Database)?);
        }
        Ok(results)
    }

    pub fn get_all_content_hashes(&self) -> AppResult<HashSet<String>> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare("SELECT DISTINCT content_hash FROM snapshots")
            .map_err(AppError::Database)?;
        let rows = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(AppError::Database)?;
        let mut hashes = HashSet::new();
        for row in rows {
            hashes.insert(row.map_err(AppError::Database)?);
        }
        Ok(hashes)
    }

    pub fn prune_snapshots(&self, days: u64) -> AppResult<usize> {
        let conn = self.conn();
        let now = chrono::Local::now();
        let cutoff_date = (now - chrono::Duration::days(days as i64)).to_rfc3339();

        // 1. Hard Cleanup: Delete everything older than the absolute retention limit
        // BUT: Keep snapshots that are tied to a Git commit (milestones)
        let _ = conn.execute(
            "DELETE FROM snapshot_chunks WHERE snapshot_id IN (
                SELECT id FROM snapshots WHERE timestamp < ?1 AND commit_hash IS NULL
            )",
            [cutoff_date.clone()],
        );
        let _ = conn.execute(
            "DELETE FROM symbols WHERE snapshot_id IN (
                SELECT id FROM snapshots WHERE timestamp < ?1 AND commit_hash IS NULL
            )",
            [cutoff_date.clone()],
        );
        let hard_pruned = conn
            .execute(
                "DELETE FROM snapshots WHERE timestamp < ?1 AND commit_hash IS NULL",
                [cutoff_date],
            )
            .map_err(AppError::Database)?;

        // 2. Intelligent Thinning (Exponential Backoff):
        // Rule: Keep ALL < 24h.
        // For > 24h: We want to keep samples.
        // We'll use a SQLite trick: Group by time windows and keep only the MAX(id) in each window.

        let thinning_count = self.thin_history(&conn, 1, 7, 3600)?; // > 24h, < 7d: 1 per hour
        let thinning_count2 = self.thin_history(&conn, 7, 30, 12 * 3600)?; // > 7d, < 30d: 1 per 12h
        let thinning_count3 = self.thin_history(&conn, 30, 9999, 24 * 3600)?; // > 30d: 1 per day

        let count = hard_pruned + thinning_count + thinning_count2 + thinning_count3;

        // 3. Compact database to reclaim space
        if count > 0 {
            let _ = conn.execute("PRAGMA wal_checkpoint(TRUNCATE)", []);
            let _ = conn.execute("VACUUM", []);
        }

        Ok(count)
    }

    fn thin_history(
        &self,
        conn: &Connection,
        min_days: i64,
        max_days: i64,
        window_seconds: i64,
    ) -> AppResult<usize> {
        let now = chrono::Local::now();
        let start_date = (now - chrono::Duration::days(max_days)).to_rfc3339();
        let end_date = (now - chrono::Duration::days(min_days)).to_rfc3339();

        // Find snapshots in the range that ARE NOT the "latest" in their window
        // and DO NOT have a commit hash.
        // Windowing is done by: strftime('%s', timestamp) / window_seconds
        let query = format!(
            "SELECT id FROM snapshots
             WHERE timestamp >= ?1 AND timestamp < ?2
             AND commit_hash IS NULL
             AND id NOT IN (
                SELECT MAX(id) FROM snapshots
                WHERE timestamp >= ?1 AND timestamp < ?2
                GROUP BY file_path, (strftime('%s', timestamp) / {})
             )",
            window_seconds
        );

        let mut stmt = conn.prepare(&query).map_err(AppError::Database)?;
        let ids: Vec<i64> = stmt
            .query_map([start_date.clone(), end_date.clone()], |row| row.get(0))
            .map_err(AppError::Database)?
            .filter_map(|r| r.ok())
            .collect();

        if ids.is_empty() {
            return Ok(0);
        }

        // Delete associated data first
        for chunk in ids.chunks(500) {
            let id_list = chunk
                .iter()
                .map(|id| id.to_string())
                .collect::<Vec<_>>()
                .join(",");
            let _ = conn.execute(
                &format!(
                    "DELETE FROM snapshot_chunks WHERE snapshot_id IN ({})",
                    id_list
                ),
                [],
            );
            let _ = conn.execute(
                &format!("DELETE FROM symbols WHERE snapshot_id IN ({})", id_list),
                [],
            );
            let _ = conn.execute(
                &format!("DELETE FROM snapshots WHERE id IN ({})", id_list),
                [],
            );
        }

        Ok(ids.len())
    }

    pub fn get_snapshot_count(&self) -> AppResult<usize> {
        let conn = self.conn();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM snapshots", [], |row| row.get(0))
            .map_err(AppError::Database)?;
        Ok(count as usize)
    }

    pub fn get_symbol_count(&self) -> AppResult<usize> {
        let conn = self.conn();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM symbols", [], |row| row.get(0))
            .map_err(AppError::Database)?;
        Ok(count as usize)
    }

    pub fn delete_all(&self) -> AppResult<usize> {
        let conn = self.conn();
        let _ = conn.execute("DELETE FROM snapshot_chunks", []);
        let _ = conn.execute("DELETE FROM symbols", []);
        let count = conn
            .execute("DELETE FROM snapshots", [])
            .map_err(AppError::Database)?;
        Ok(count)
    }

    pub fn vacuum(&self) -> AppResult<()> {
        let conn = self.conn();
        conn.execute("VACUUM", []).map_err(AppError::Database)?;
        Ok(())
    }

    pub fn get_snapshot_by_id(&self, id: i64) -> AppResult<Option<Snapshot>> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare("SELECT id, file_path, timestamp, content_hash, git_branch, session_id, commit_hash FROM snapshots WHERE id = ?1")
            .map_err(AppError::Database)?;

        let mut rows = stmt
            .query_map([id], |row| {
                Ok(Snapshot {
                    id: row.get(0)?,
                    file_path: row.get(1)?,
                    timestamp: row.get(2)?,
                    content_hash: row.get(3)?,
                    git_branch: row.get(4)?,
                    session_id: row.get(5)?,
                    commit_hash: row.get(6)?,
                })
            })
            .map_err(AppError::Database)?;

        if let Some(row) = rows.next() {
            Ok(Some(row.map_err(AppError::Database)?))
        } else {
            Ok(None)
        }
    }

    pub fn get_latest_state(&self) -> AppResult<Vec<(String, String)>> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare(
                "SELECT file_path, content_hash FROM snapshots s1
                 WHERE id = (SELECT MAX(id) FROM snapshots s2 WHERE s2.file_path = s1.file_path)
                 ORDER BY file_path",
            )
            .map_err(AppError::Database)?;
        let rows = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(AppError::Database)?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row.map_err(AppError::Database)?);
        }
        Ok(results)
    }

    pub fn get_state_at_timestamp(&self, timestamp: &str) -> AppResult<Vec<(String, String)>> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare(
                "SELECT file_path, content_hash FROM snapshots s1
                 WHERE id = (
                     SELECT MAX(id) FROM snapshots s2
                     WHERE s2.file_path = s1.file_path AND s2.timestamp <= ?1
                 )
                 ORDER BY file_path",
            )
            .map_err(AppError::Database)?;
        let rows = stmt
            .query_map([timestamp], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(AppError::Database)?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row.map_err(AppError::Database)?);
        }
        Ok(results)
    }

    pub fn create_session(&self, start_time: &str, git_branch: Option<&str>) -> AppResult<i64> {
        let conn = self.conn();
        conn.execute(
            "INSERT INTO sessions (start_time, git_branch) VALUES (?1, ?2)",
            params![start_time, git_branch],
        )
        .map_err(AppError::Database)?;
        Ok(conn.last_insert_rowid())
    }

    pub fn close_session(
        &self,
        session_id: i64,
        end_time: &str,
        file_count: usize,
        snapshot_count: usize,
    ) -> AppResult<()> {
        let conn = self.conn();
        conn.execute(
            "UPDATE sessions SET end_time = ?1, file_count = ?2, snapshot_count = ?3 WHERE id = ?4",
            params![
                end_time,
                file_count as i64,
                snapshot_count as i64,
                session_id
            ],
        )
        .map_err(AppError::Database)?;
        Ok(())
    }

    pub fn get_active_session(&self) -> AppResult<Option<Session>> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare(
                "SELECT id, start_time, end_time, git_branch, file_count, snapshot_count
                 FROM sessions WHERE end_time IS NULL ORDER BY id DESC LIMIT 1",
            )
            .map_err(AppError::Database)?;
        let mut rows = stmt
            .query_map([], |row| {
                Ok(Session {
                    id: row.get(0)?,
                    start_time: row.get(1)?,
                    end_time: row.get(2)?,
                    git_branch: row.get(3)?,
                    file_count: row.get::<_, i64>(4)? as usize,
                    snapshot_count: row.get::<_, i64>(5)? as usize,
                })
            })
            .map_err(AppError::Database)?;
        if let Some(row) = rows.next() {
            Ok(Some(row.map_err(AppError::Database)?))
        } else {
            Ok(None)
        }
    }

    pub fn list_sessions(&self, limit: usize) -> AppResult<Vec<Session>> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare(
                "SELECT id, start_time, end_time, git_branch, file_count, snapshot_count
                 FROM sessions ORDER BY start_time DESC LIMIT ?1",
            )
            .map_err(AppError::Database)?;
        let rows = stmt
            .query_map([limit as i64], |row| {
                Ok(Session {
                    id: row.get(0)?,
                    start_time: row.get(1)?,
                    end_time: row.get(2)?,
                    git_branch: row.get(3)?,
                    file_count: row.get::<_, i64>(4)? as usize,
                    snapshot_count: row.get::<_, i64>(5)? as usize,
                })
            })
            .map_err(AppError::Database)?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row.map_err(AppError::Database)?);
        }
        Ok(results)
    }

    pub fn get_timesheet(&self, days: u64) -> AppResult<Vec<TimesheetEntry>> {
        let conn = self.conn();
        let cutoff = (chrono::Local::now() - chrono::Duration::days(days as i64)).to_rfc3339();
        let mut stmt = conn
            .prepare(
                "SELECT
                    DATE(start_time) as day,
                    git_branch,
                    SUM(
                        CAST((julianday(COALESCE(end_time, datetime('now'))) - julianday(start_time)) * 24 * 60 AS INTEGER)
                    ) as total_minutes,
                    SUM(file_count) as total_files,
                    SUM(snapshot_count) as total_snapshots
                 FROM sessions
                 WHERE start_time >= ?1
                 GROUP BY day, git_branch
                 ORDER BY day DESC",
            )
            .map_err(AppError::Database)?;
        let rows = stmt
            .query_map([&cutoff], |row| {
                Ok(TimesheetEntry {
                    date: row.get(0)?,
                    branch: row.get(1)?,
                    duration_minutes: row.get::<_, i64>(2)? as u64,
                    file_count: row.get::<_, i64>(3)? as usize,
                    snapshot_count: row.get::<_, i64>(4)? as usize,
                })
            })
            .map_err(AppError::Database)?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row.map_err(AppError::Database)?);
        }
        Ok(results)
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

        let conn = self.conn();
        conn.execute(
            "INSERT INTO project_checkpoints (hash, timestamp, description, file_states) VALUES (?1, ?2, ?3, ?4)",
            params![hash, timestamp, description, file_states_json],
        )
        .map_err(AppError::Database)?;
        Ok(hash)
    }

    pub fn list_checkpoints(&self) -> AppResult<Vec<(String, String, Option<String>)>> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare("SELECT hash, timestamp, description FROM project_checkpoints WHERE hash IS NOT NULL ORDER BY timestamp DESC")
            .map_err(AppError::Database)?;
        let rows = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
            .map_err(AppError::Database)?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row.map_err(AppError::Database)?);
        }
        Ok(results)
    }

    pub fn get_checkpoint_by_hash(
        &self,
        hash_query: &str,
    ) -> AppResult<Option<(String, String, Option<String>)>> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare(
                "SELECT timestamp, file_states, description FROM project_checkpoints WHERE hash LIKE ?1 LIMIT 1",
            )
            .map_err(AppError::Database)?;
        let mut rows = stmt
            .query_map([format!("{}%", hash_query)], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            })
            .map_err(AppError::Database)?;

        if let Some(row) = rows.next() {
            Ok(Some(row.map_err(AppError::Database)?))
        } else {
            Ok(None)
        }
    }

    pub fn get_checkpoint(&self, id: i64) -> AppResult<Option<(String, String)>> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare("SELECT timestamp, file_states FROM project_checkpoints WHERE id = ?1")
            .map_err(AppError::Database)?;
        let mut rows = stmt
            .query_map([id], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(AppError::Database)?;
        if let Some(row) = rows.next() {
            Ok(Some(row.map_err(AppError::Database)?))
        } else {
            Ok(None)
        }
    }

    pub fn delete_checkpoint(&self, hash: &str) -> AppResult<bool> {
        let conn = self.conn();
        let rows_affected = conn
            .execute("DELETE FROM project_checkpoints WHERE hash = ?1", [hash])
            .map_err(AppError::Database)?;
        Ok(rows_affected > 0)
    }

    pub fn get_commits(&self) -> AppResult<Vec<(String, String, String, String, usize)>> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare(
                "SELECT c.hash, c.message, c.author, c.timestamp, COUNT(s.id) as file_count
                 FROM git_commits c
                 INNER JOIN snapshots s ON c.hash = s.commit_hash
                 GROUP BY c.hash, c.message, c.author, c.timestamp
                 ORDER BY c.timestamp DESC",
            )
            .map_err(AppError::Database)?;

        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get(0)?, // commit_hash
                    row.get(1)?, // commit_message
                    row.get(2)?, // commit_author
                    row.get(3)?, // timestamp
                    row.get(4)?, // file_count
                ))
            })
            .map_err(AppError::Database)?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row.map_err(AppError::Database)?);
        }
        Ok(results)
    }

    pub fn get_commit_by_hash(
        &self,
        hash: &str,
    ) -> AppResult<Option<(String, String, String, String)>> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare(
                "SELECT hash, message, author, timestamp
                 FROM git_commits
                 WHERE hash = ?1
                 LIMIT 1",
            )
            .map_err(AppError::Database)?;

        let mut rows = stmt
            .query_map([hash], |row| {
                Ok((
                    row.get(0)?, // commit_hash
                    row.get(1)?, // commit_message
                    row.get(2)?, // commit_author
                    row.get(3)?, // timestamp
                ))
            })
            .map_err(AppError::Database)?;

        if let Some(row) = rows.next() {
            Ok(Some(row.map_err(AppError::Database)?))
        } else {
            Ok(None)
        }
    }

    pub fn get_commit_files(&self, hash: &str) -> AppResult<Vec<(String, String, String)>> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare(
                "SELECT file_path, content_hash, timestamp
                 FROM snapshots
                 WHERE commit_hash = ?1
                 ORDER BY timestamp ASC",
            )
            .map_err(AppError::Database)?;

        let rows = stmt
            .query_map([hash], |row| {
                Ok((
                    row.get(0)?, // file_path
                    row.get(1)?, // content_hash
                    row.get(2)?, // timestamp
                ))
            })
            .map_err(AppError::Database)?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row.map_err(AppError::Database)?);
        }
        Ok(results)
    }

    pub fn insert_chunk(&self, hash: &str, _content: &[u8], kind: &str) -> AppResult<()> {
        let conn = self.conn();
        conn.execute(
            "INSERT OR IGNORE INTO chunks (hash, kind) VALUES (?1, ?2)",
            params![hash, kind],
        )
        .map_err(AppError::Database)?;
        Ok(())
    }

    pub fn link_snapshot_chunk(
        &self,
        snapshot_id: i64,
        chunk_hash: &str,
        position: usize,
    ) -> AppResult<()> {
        let conn = self.conn();
        conn.execute(
            "INSERT INTO snapshot_chunks (snapshot_id, chunk_hash, position) VALUES (?1, ?2, ?3)",
            params![snapshot_id, chunk_hash, position as i64],
        )
        .map_err(AppError::Database)?;
        Ok(())
    }

    pub fn insert_symbol(&self, symbol: &SemanticSymbol) -> AppResult<i64> {
        let conn = self.conn();
        conn.execute(
            "INSERT INTO symbols (name, kind, scope, snapshot_id, chunk_hash, structural_hash, start_line, end_line, start_byte, end_byte, parent_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                symbol.name,
                symbol.kind,
                symbol.scope,
                symbol.snapshot_id,
                symbol.chunk_hash,
                symbol.structural_hash,
                symbol.start_line as i64,
                symbol.end_line as i64,
                symbol.start_byte as i64,
                symbol.end_byte as i64,
                symbol.parent_id,
            ],
        )
        .map_err(AppError::Database)?;
        Ok(conn.last_insert_rowid())
    }

    pub fn insert_symbol_delta(&self, delta: &crate::models::SemanticRecord) -> AppResult<i64> {
        let kind_str = match delta.kind {
            crate::models::RecordKind::Added => "Added",
            crate::models::RecordKind::Modified => "Modified",
            crate::models::RecordKind::Deleted => "Deleted",
            crate::models::RecordKind::Renamed => "Renamed",
        };

        let conn = self.conn();
        conn.execute(
            "INSERT INTO symbol_deltas (from_snapshot_id, to_snapshot_id, symbol_name, new_name, delta_kind, structural_hash)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                delta.from_snapshot_id,
                delta.to_snapshot_id,
                delta.symbol_name,
                delta.new_name,
                kind_str,
                delta.structural_hash,
            ],
        )
        .map_err(AppError::Database)?;
        Ok(conn.last_insert_rowid())
    }

    pub fn get_symbol_deltas(
        &self,
        symbol_name: &str,
    ) -> AppResult<Vec<crate::models::SemanticRecord>> {
        let conn = self.conn();
        let mut stmt = conn.prepare(
            "SELECT id, from_snapshot_id, to_snapshot_id, symbol_name, new_name, delta_kind, structural_hash
             FROM symbol_deltas
             WHERE symbol_name = ?1 OR new_name = ?1
             ORDER BY to_snapshot_id DESC"
        ).map_err(AppError::Database)?;

        let rows = stmt
            .query_map([symbol_name], |row| {
                let kind_str: String = row.get(5)?;
                let kind = match kind_str.as_str() {
                    "Added" => crate::models::RecordKind::Added,
                    "Modified" => crate::models::RecordKind::Modified,
                    "Deleted" => crate::models::RecordKind::Deleted,
                    "Renamed" => crate::models::RecordKind::Renamed,
                    _ => crate::models::RecordKind::Modified,
                };

                Ok(crate::models::SemanticRecord {
                    id: row.get(0)?,
                    project_id: None,
                    from_snapshot_id: row.get(1)?,
                    to_snapshot_id: row.get(2)?,
                    symbol_name: row.get(3)?,
                    new_name: row.get(4)?,
                    kind,
                    structural_hash: row.get(6)?,
                })
            })
            .map_err(AppError::Database)?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row.map_err(AppError::Database)?);
        }
        Ok(results)
    }

    pub fn insert_reference(&self, reference: &SymbolReference) -> AppResult<()> {
        let conn = self.conn();
        conn.execute(
            "INSERT INTO symbol_references (symbol_name, snapshot_id, start_line, start_byte) VALUES (?1, ?2, ?3, ?4)",
            params![
                reference.symbol_name,
                reference.snapshot_id,
                reference.start_line as i64,
                reference.start_byte as i64,
            ],
        )
        .map_err(AppError::Database)?;
        Ok(())
    }

    pub fn get_symbol_history(
        &self,
        symbol_name: &str,
    ) -> AppResult<Vec<(Snapshot, SemanticSymbol)>> {
        let conn = self.conn();
        let latest_hash: String = conn
            .query_row(
                "SELECT structural_hash FROM symbols WHERE name = ?1 ORDER BY id DESC LIMIT 1",
                [symbol_name],
                |row| row.get(0),
            )
            .map_err(AppError::Database)?;
        let mut stmt = conn.prepare(
            "SELECT s.id, s.file_path, s.timestamp, s.content_hash, s.git_branch, s.session_id, s.commit_hash,
                    sym.id, sym.name, sym.kind, sym.scope, sym.snapshot_id, sym.chunk_hash, sym.structural_hash, sym.start_line, sym.end_line, sym.start_byte, sym.end_byte, sym.parent_id
             FROM symbols sym
             JOIN snapshots s ON sym.snapshot_id = s.id
             WHERE sym.structural_hash = ?1 OR sym.name = ?2
             ORDER BY s.timestamp DESC"
        ).map_err(AppError::Database)?;
        let rows = stmt
            .query_map(params![latest_hash, symbol_name], |row| {
                let snap = Snapshot {
                    id: row.get(0)?,
                    file_path: row.get(1)?,
                    timestamp: row.get(2)?,
                    content_hash: row.get(3)?,
                    git_branch: row.get(4)?,
                    session_id: row.get(5)?,
                    commit_hash: row.get(6)?,
                };
                let sym = SemanticSymbol {
                    id: row.get(7)?,
                    name: row.get(8)?,
                    kind: row.get(9)?,
                    scope: row.get(10)?,
                    snapshot_id: row.get(11)?,
                    chunk_hash: row.get(12)?,
                    structural_hash: row.get(13).unwrap_or_default(),
                    start_line: row.get::<_, i64>(14)? as usize,
                    end_line: row.get::<_, i64>(15)? as usize,
                    start_byte: row.get::<_, i64>(16)? as usize,
                    end_byte: row.get::<_, i64>(17)? as usize,
                    parent_id: row.get(18)?,
                };
                Ok((snap, sym))
            })
            .map_err(AppError::Database)?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row.map_err(AppError::Database)?);
        }
        Ok(results)
    }

    pub fn get_chunks_for_hash(&self, content_hash: &str) -> AppResult<Vec<String>> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare(
                "SELECT sc.chunk_hash FROM snapshot_chunks sc
                 JOIN snapshots s ON sc.snapshot_id = s.id
                 WHERE s.content_hash = ?1
                 ORDER BY sc.position ASC",
            )
            .map_err(AppError::Database)?;

        let rows = stmt
            .query_map([content_hash], |row| row.get::<_, String>(0))
            .map_err(AppError::Database)?;

        let mut chunks = Vec::new();
        for row in rows {
            chunks.push(row.map_err(AppError::Database)?);
        }
        Ok(chunks)
    }

    pub fn get_symbols_for_snapshot(&self, snapshot_id: i64) -> AppResult<Vec<SemanticSymbol>> {
        let conn = self.conn();
        let mut stmt = conn.prepare(
            "SELECT id, name, kind, scope, snapshot_id, chunk_hash, structural_hash, start_line, end_line, start_byte, end_byte, parent_id
             FROM symbols WHERE snapshot_id = ?1 ORDER BY start_byte ASC",
        ).map_err(AppError::Database)?;
        let rows = stmt
            .query_map([snapshot_id], |row| {
                Ok(SemanticSymbol {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    kind: row.get(2)?,
                    scope: row.get(3)?,
                    snapshot_id: row.get(4)?,
                    chunk_hash: row.get(5)?,
                    structural_hash: row.get(6).unwrap_or_default(),
                    start_line: row.get::<_, i64>(7)? as usize,
                    end_line: row.get::<_, i64>(8)? as usize,
                    start_byte: row.get::<_, i64>(9)? as usize,
                    end_byte: row.get::<_, i64>(10)? as usize,
                    parent_id: row.get(11)?,
                })
            })
            .map_err(AppError::Database)?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row.map_err(AppError::Database)?);
        }
        Ok(results)
    }

    pub fn find_symbols_by_name(&self, query: &str) -> AppResult<Vec<SemanticSymbol>> {
        let conn = self.conn();
        let mut stmt = conn.prepare(
            "SELECT id, name, kind, scope, snapshot_id, chunk_hash, structural_hash, start_line, end_line, start_byte, end_byte, parent_id
             FROM symbols WHERE name LIKE ?1 ORDER BY id DESC LIMIT 100",
        ).map_err(AppError::Database)?;
        let rows = stmt
            .query_map([format!("%{}%", query)], |row| {
                Ok(SemanticSymbol {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    kind: row.get(2)?,
                    scope: row.get(3)?,
                    snapshot_id: row.get(4)?,
                    chunk_hash: row.get(5)?,
                    structural_hash: row.get(6).unwrap_or_default(),
                    start_line: row.get::<_, i64>(7)? as usize,
                    end_line: row.get::<_, i64>(8)? as usize,
                    start_byte: row.get::<_, i64>(9)? as usize,
                    end_byte: row.get::<_, i64>(10)? as usize,
                    parent_id: row.get(11)?,
                })
            })
            .map_err(AppError::Database)?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row.map_err(AppError::Database)?);
        }
        Ok(results)
    }

    pub fn get_file_count(&self) -> AppResult<usize> {
        let conn = self.conn();
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(DISTINCT file_path) FROM snapshots",
                [],
                |row| row.get(0),
            )
            .map_err(AppError::Database)?;
        Ok(count as usize)
    }

    pub fn get_branch_count(&self) -> AppResult<usize> {
        let conn = self.conn();
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(DISTINCT git_branch) FROM snapshots WHERE git_branch IS NOT NULL",
                [],
                |row| row.get(0),
            )
            .map_err(AppError::Database)?;
        Ok(count as usize)
    }

    pub fn get_commit_count(&self) -> AppResult<usize> {
        let conn = self.conn();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM git_commits", [], |row| row.get(0))
            .map_err(AppError::Database)?;
        Ok(count as usize)
    }

    pub fn get_last_activity_time(&self) -> AppResult<String> {
        let conn = self.conn();
        let time: String = conn
            .query_row("SELECT MAX(timestamp) FROM snapshots", [], |row| row.get(0))
            .map_err(AppError::Database)?;
        Ok(time)
    }

    pub fn get_activity_by_day(&self, limit: usize) -> AppResult<Vec<(String, usize)>> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare("SELECT DATE(timestamp) as day, COUNT(*) as count FROM snapshots GROUP BY day ORDER BY day DESC LIMIT ?1")
            .map_err(AppError::Database)?;
        let rows = stmt
            .query_map([limit as i64], |row| {
                Ok((row.get(0)?, row.get::<_, i64>(1)? as usize))
            })
            .map_err(AppError::Database)?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row.map_err(AppError::Database)?);
        }
        Ok(results)
    }

    pub fn get_activity_by_hour(&self) -> AppResult<Vec<(usize, usize)>> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare("SELECT CAST(strftime('%H', timestamp) as INTEGER) as hour, COUNT(*) as count FROM snapshots GROUP BY hour ORDER BY hour ASC")
            .map_err(AppError::Database)?;
        let rows = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get::<_, i64>(1)? as usize)))
            .map_err(AppError::Database)?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row.map_err(AppError::Database)?);
        }
        Ok(results)
    }

    pub fn get_top_files(&self, limit: usize) -> AppResult<Vec<(String, usize)>> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare("SELECT file_path, COUNT(*) as count FROM snapshots GROUP BY file_path ORDER BY count DESC LIMIT ?1")
            .map_err(AppError::Database)?;
        let rows = stmt
            .query_map([limit as i64], |row| {
                Ok((row.get(0)?, row.get::<_, i64>(1)? as usize))
            })
            .map_err(AppError::Database)?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row.map_err(AppError::Database)?);
        }
        Ok(results)
    }

    pub fn get_top_branches(&self, limit: usize) -> AppResult<Vec<(String, usize)>> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare("SELECT git_branch, COUNT(*) as count FROM snapshots WHERE git_branch IS NOT NULL GROUP BY git_branch ORDER BY count DESC LIMIT ?1")
            .map_err(AppError::Database)?;
        let rows = stmt
            .query_map([limit as i64], |row| {
                Ok((row.get(0)?, row.get::<_, i64>(1)? as usize))
            })
            .map_err(AppError::Database)?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row.map_err(AppError::Database)?);
        }
        Ok(results)
    }

    pub fn get_extension_distribution(&self) -> AppResult<Vec<(String, usize)>> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare("SELECT file_path, COUNT(*) FROM snapshots GROUP BY file_path")
            .map_err(AppError::Database)?;
        let rows = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? as usize))
            })
            .map_err(AppError::Database)?;

        let mut exts = std::collections::HashMap::new();
        for row in rows {
            let (path, count) = row.map_err(AppError::Database)?;
            let ext = std::path::Path::new(&path)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("no extension")
                .to_lowercase();
            *exts.entry(ext).or_insert(0) += count;
        }

        let mut results: Vec<_> = exts.into_iter().collect();
        results.sort_by(|a, b| b.1.cmp(&a.1));
        results.truncate(15);
        Ok(results)
    }
}
