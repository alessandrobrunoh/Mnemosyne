use crate::Monitor;
use dashmap::DashMap;
use mnem_core::protocol::{ClientCapabilities, ServerCapabilities};
use mnem_core::Repository;
use parking_lot::RwLock;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

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

    /// Metrics (Atomics for zero-lock updates)
    pub total_requests: AtomicU64,
    pub total_processing_time_us: AtomicU64,
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
            total_requests: AtomicU64::new(0),
            total_processing_time_us: AtomicU64::new(0),
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
}
