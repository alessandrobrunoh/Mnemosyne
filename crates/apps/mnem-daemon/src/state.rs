use crate::Monitor;
use dashmap::DashMap;
use lru::LruCache;
use mnem_core::models::Snapshot;
use mnem_core::protocol::{ClientCapabilities, ServerCapabilities};
use mnem_core::Repository;
use parking_lot::RwLock;
use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

const HISTORY_CACHE_SIZE: NonZeroUsize = NonZeroUsize::new(1000).unwrap();

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InitializationState {
    Uninitialized,
    Initializing,
    Initialized,
    Shutdown,
}

/// The global state of the daemon, optimized for high concurrency.
/// Instead of a single giant Mutex, we use granular locking and concurrent maps.
pub struct DaemonState {
    pub start_time: Instant,
    pub auth_token: String,

    /// Active monitors keyed by project path (Concurrent Map)
    pub monitors: DashMap<String, Arc<Monitor>>,

    /// Active repositories keyed by project path (Concurrent Map)
    pub repos: DashMap<String, Arc<Repository>>,

    /// LRU cache for history queries (file_path -> history results)
    pub history_cache: RwLock<LruCache<String, Vec<Snapshot>>>,

    /// Metrics (Atomics for zero-lock updates)
    pub total_requests: AtomicU64,
    pub total_processing_time_us: AtomicU64,
    pub total_saves: AtomicU64,
    pub total_save_time_us: AtomicU64,
    pub cached_history_size: AtomicU64,
    pub cached_total_size: AtomicU64,

    /// Protocol state (Granular RwLock)
    pub init_state: RwLock<InitializationState>,

    /// Server capabilities (set after initialization)
    pub server_capabilities: RwLock<Option<ServerCapabilities>>,

    /// Client capabilities (received during initialization)
    pub client_capabilities: RwLock<Option<ClientCapabilities>>,
}

impl DaemonState {
    pub fn new(auth_token: String) -> Self {
        Self {
            start_time: Instant::now(),
            auth_token,
            monitors: DashMap::new(),
            repos: DashMap::new(),
            history_cache: RwLock::new(LruCache::new(HISTORY_CACHE_SIZE)),
            total_requests: AtomicU64::new(0),
            total_processing_time_us: AtomicU64::new(0),
            total_saves: AtomicU64::new(0),
            total_save_time_us: AtomicU64::new(0),
            cached_history_size: AtomicU64::new(0),
            cached_total_size: AtomicU64::new(0),
            init_state: RwLock::new(InitializationState::Uninitialized),
            server_capabilities: RwLock::new(None),
            client_capabilities: RwLock::new(None),
        }
    }

    pub fn is_initialized(&self) -> bool {
        *self.init_state.read() == InitializationState::Initialized
    }

    pub fn is_shutdown(&self) -> bool {
        *self.init_state.read() == InitializationState::Shutdown
    }

    /// Record a request execution time
    pub fn record_request(&self, duration_us: u64) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        self.total_processing_time_us
            .fetch_add(duration_us, Ordering::Relaxed);
    }

    /// Record a snapshot save execution time
    pub fn record_save(&self, duration_us: u64) {
        self.total_saves.fetch_add(1, Ordering::Relaxed);
        self.total_save_time_us
            .fetch_add(duration_us, Ordering::Relaxed);
    }

    /// Calculate total storage size from all watched projects
    pub fn calculate_total_size(&self) -> u64 {
        let mut total: u64 = 0;

        for repo in self.repos.iter() {
            if let Ok(size) = repo.get_project_size() {
                total += size;
            }
        }

        self.cached_total_size.store(total, Ordering::Relaxed);
        total
    }

    /// Get cached history for a file path
    pub fn get_cached_history(&self, file_path: &str) -> Option<Vec<Snapshot>> {
        let mut cache = self.history_cache.write();
        cache.get(file_path).cloned()
    }

    /// Cache history for a file path
    pub fn cache_history(&self, file_path: String, history: Vec<Snapshot>) {
        let mut cache = self.history_cache.write();
        cache.push(file_path, history);
    }

    /// Clear history cache (useful after file changes)
    pub fn invalidate_history_cache(&self, file_path: Option<&str>) {
        let mut cache = self.history_cache.write();
        match file_path {
            Some(path) => {
                cache.pop(path);
            }
            None => {
                cache.clear();
            }
        }
    }
}
