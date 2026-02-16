use serde::{Deserialize, Serialize};

#[cfg(unix)]
pub use crate::os::unix::{SOCKET_DIR, SOCKET_NAME};
#[cfg(windows)]
pub use crate::os::windows::{SOCKET_DIR, SOCKET_NAME};

pub const PID_FILE: &str = "mnem-daemon.pid";
pub use crate::os::get_socket_path;

// Protocol version - follows semver
pub const PROTOCOL_VERSION: &str = "1.0.0";

// ---------------------------------------------------------------------------
// JSON-RPC 2.0 Error Codes
// ---------------------------------------------------------------------------

/// Standard JSON-RPC 2.0 error codes
pub mod jsonrpc_errors {
    // Standard JSON-RPC 2.0 errors
    pub const PARSE_ERROR: i32 = -32700;
    pub const INVALID_REQUEST: i32 = -32600;
    pub const METHOD_NOT_FOUND: i32 = -32601;
    pub const INVALID_PARAMS: i32 = -32602;
    pub const INTERNAL_ERROR: i32 = -32603;
    pub const SERVER_ERROR_START: i32 = -32000;
    pub const SERVER_ERROR_END: i32 = -32099;
}

/// Mnemosyne-specific error codes (range -32100 to -32199)
pub mod mnem_errors {
    pub const SERVER_NOT_INITIALIZED: i32 = -32100;
    pub const ALREADY_INITIALIZED: i32 = -32101;
    pub const UNAUTHORIZED: i32 = -32102;
    pub const PROJECT_NOT_FOUND: i32 = -32103;
    pub const SNAPSHOT_NOT_FOUND: i32 = -32104;
    pub const SYMBOL_NOT_FOUND: i32 = -32105;
    pub const STORAGE_ERROR: i32 = -32106;
    pub const INVALID_PATH: i32 = -32107;
    pub const PROJECT_ALREADY_WATCHED: i32 = -32108;
    pub const SHUTDOWN_IN_PROGRESS: i32 = -32109;
}

// ---------------------------------------------------------------------------
// JSON-RPC 2.0 envelope
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<u64>,
    pub method: String,
    #[serde(default)]
    pub params: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
}

impl JsonRpcRequest {
    pub fn new(id: u64, method: &str, params: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            id: Some(id),
            method: method.into(),
            params,
            auth_token: None,
        }
    }

    /// Notification (no id, no response expected).
    pub fn notification(method: &str, params: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            id: None,
            method: method.into(),
            params,
            auth_token: None,
        }
    }
}

impl JsonRpcResponse {
    pub fn success(id: Option<u64>, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            id,
            result: Some(result),
            error: None,
        }
    }

    pub fn error(id: Option<u64>, code: i32, message: String) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            id,
            result: None,
            error: Some(JsonRpcError { code, message }),
        }
    }
}

// ---------------------------------------------------------------------------
// Lifecycle Management (LSP-style)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeParams {
    /// Information about the client
    pub client_info: Option<ClientInfo>,
    /// Client capabilities
    pub capabilities: ClientCapabilities,
    /// Workspace folders (optional, for multi-root workspaces)
    pub workspace_folders: Option<Vec<WorkspaceFolder>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    pub name: String,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClientCapabilities {
    /// Whether the client supports semantic analysis features
    #[serde(default)]
    pub semantic_analysis: bool,
    /// Whether the client supports git integration features
    #[serde(default)]
    pub git_integration: bool,
    /// Whether the client supports progress notifications
    #[serde(default)]
    pub progress_notifications: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceFolder {
    pub uri: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeResult {
    /// Information about the server
    pub server_info: ServerInfo,
    /// Server capabilities
    pub capabilities: ServerCapabilities,
    /// Protocol version
    pub protocol_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerCapabilities {
    /// Protocol version
    pub protocol_version: String,
    /// List of supported method names
    pub supported_methods: Vec<String>,
    /// Whether semantic analysis is available
    pub semantic_analysis: bool,
    /// Whether git integration is available
    pub git_integration: bool,
    /// Maximum batch request size (0 if batching not supported)
    pub max_batch_size: usize,
    /// Whether server supports streaming results
    pub supports_streaming: bool,
    /// List of supported programming languages
    pub supported_languages: Vec<String>,
}

impl Default for ServerCapabilities {
    fn default() -> Self {
        Self {
            protocol_version: PROTOCOL_VERSION.to_string(),
            supported_methods: vec![],
            semantic_analysis: true,
            git_integration: true,
            max_batch_size: 0, // No batching support in v1.0
            supports_streaming: false,
            supported_languages: vec![
                "rust".to_string(),
                "python".to_string(),
                "javascript".to_string(),
                "typescript".to_string(),
                "go".to_string(),
                "java".to_string(),
                "c".to_string(),
                "cpp".to_string(),
                "csharp".to_string(),
                "ruby".to_string(),
                "php".to_string(),
                "json".to_string(),
                "html".to_string(),
                "css".to_string(),
                "markdown".to_string(),
            ],
        }
    }
}

// ---------------------------------------------------------------------------
// RPC methods (exhaustive list)
// ---------------------------------------------------------------------------

/// All supported RPC method names as constants.
pub mod methods {
    // Lifecycle (new in v1.0)
    pub const INITIALIZE: &str = "initialize";
    pub const INITIALIZED: &str = "initialized";
    pub const SHUTDOWN: &str = "shutdown";
    pub const EXIT: &str = "exit";

    // Legacy methods (deprecated, use v1 names)
    // These are kept for backward compatibility
    pub const WATCH: &str = "project/watch";
    pub const UNWATCH: &str = "project/unwatch";
    pub const GET_WATCHED_PROJECTS: &str = "project/get_watched";
    pub const STATUS: &str = "daemon/status";
    pub const PROJECT_ACTIVITY: &str = "project/activity";
    pub const PROJECT_MAP: &str = "project/map";
    pub const PROJECT_STATISTICS: &str = "project/statistics";
    pub const SNAPSHOT_SAVE: &str = "snapshot/save";
    pub const SNAPSHOT_HISTORY: &str = "snapshot/history";
    pub const SNAPSHOT_CONTENT: &str = "snapshot/content";
    pub const SNAPSHOT_RESTORE: &str = "snapshot/restore";
    pub const SNAPSHOT_RESTORE_SYMBOL: &str = "snapshot/restore_symbol";
    pub const SYMBOL_HISTORY: &str = "symbol/history";
    pub const SYMBOL_DIFF: &str = "symbol/diff";
    pub const SYMBOL_FIND: &str = "symbol/find";
    pub const FILE_LIST: &str = "file/list";
    pub const FILE_SEARCH: &str = "file/search";
    pub const CONTENT_SEARCH: &str = "content/search";
    pub const FILE_DIFF: &str = "file/diff";
    pub const FILE_INFO: &str = "file/info";
    pub const BRANCH_LIST: &str = "branch/list";
    pub const BRANCH_CURRENT: &str = "branch/current";
    pub const SESSION_LIST: &str = "session/list";
    pub const SESSION_ACTIVE: &str = "session/active";
    pub const SESSION_TIMESHEET: &str = "session/timesheet";
    pub const PROJECT_CHECKPOINT: &str = "project/checkpoint";
    pub const PROJECT_REVERT: &str = "project/revert";
    pub const GC_RUN: &str = "gc/run";
    pub const CONFIG_GET: &str = "config/get";
    pub const CONFIG_SET: &str = "config/set";
    pub const TIER_CONFIG_GET: &str = "tier/config/get";
    pub const TIER_CONFIG_SET: &str = "tier/config/set";

    // Version 1.0 standardized names (preferred)
    // Use these for new implementations
    pub const PROJECT_WATCH: &str = "mnem/project/watch";
    pub const PROJECT_UNWATCH: &str = "mnem/project/unwatch";
    pub const PROJECT_LIST: &str = "mnem/project/list";
    pub const PROJECT_GET_ACTIVITY: &str = "mnem/project/activity";
    pub const PROJECT_GET_MAP: &str = "mnem/project/map";
    pub const PROJECT_GET_STATISTICS: &str = "mnem/project/statistics";
    pub const SNAPSHOT_CREATE: &str = "mnem/snapshot/create";
    pub const SNAPSHOT_LIST: &str = "mnem/snapshot/list";
    pub const SNAPSHOT_GET: &str = "mnem/snapshot/get";
    pub const SNAPSHOT_RESTORE_V1: &str = "mnem/snapshot/restore";
    pub const SNAPSHOT_RESTORE_SYMBOL_V1: &str = "mnem/snapshot/restoreSymbol";
    pub const SYMBOL_GET_HISTORY: &str = "mnem/symbol/history";
    pub const SYMBOL_GET_DIFF: &str = "mnem/symbol/diff";
    pub const SYMBOL_SEARCH: &str = "mnem/symbol/search";
    pub const SYMBOL_GET_SEMANTIC_HISTORY: &str = "mnem/symbol/semantic_history";
    pub const FILE_GET_LIST: &str = "mnem/file/list";
    pub const FILE_SEARCH_V1: &str = "mnem/file/search";
    pub const CONTENT_SEARCH_V1: &str = "mnem/content/search";
    pub const FILE_GET_DIFF: &str = "mnem/file/diff";
    pub const FILE_GET_INFO: &str = "mnem/file/info";
    pub const BRANCH_GET_LIST: &str = "mnem/branch/list";
    pub const BRANCH_GET_CURRENT: &str = "mnem/branch/current";
    pub const SESSION_GET_LIST: &str = "mnem/session/list";
    pub const SESSION_GET_ACTIVE: &str = "mnem/session/active";
    pub const SESSION_GET_TIMESHEET: &str = "mnem/session/timesheet";
    pub const PROJECT_CREATE_CHECKPOINT: &str = "mnem/project/checkpoint";
    pub const PROJECT_REVERT_V1: &str = "mnem/project/revert";
    pub const PROJECT_RELOAD: &str = "mnem/project/reload";
    pub const MAINTENANCE_GC: &str = "mnem/maintenance/gc";
    pub const CONFIG_GET_V1: &str = "mnem/config/get";
    pub const CONFIG_SET_V1: &str = "mnem/config/set";
    pub const TIER_CONFIG_GET_V1: &str = "mnem/tier/config/get";
    pub const TIER_CONFIG_SET_V1: &str = "mnem/tier/config/set";
    pub const DAEMON_GET_STATUS: &str = "mnem/daemon/status";

    // MCP Server
    pub const MCP_START: &str = "mnem/mcp/start";
    pub const MCP_STOP: &str = "mnem/mcp/stop";
    pub const MCP_STATUS: &str = "mnem/mcp/status";
}

/// Maps legacy method names to their v1.0 equivalents
/// Returns the standardized method name, or the original if not a legacy name
pub fn normalize_method_name(method: &str) -> &str {
    match method {
        // Legacy to v1.0 mapping
        methods::WATCH => methods::PROJECT_WATCH,
        methods::UNWATCH => methods::PROJECT_UNWATCH,
        methods::GET_WATCHED_PROJECTS => methods::PROJECT_LIST,
        methods::STATUS => methods::DAEMON_GET_STATUS,
        methods::PROJECT_ACTIVITY => methods::PROJECT_GET_ACTIVITY,
        methods::PROJECT_MAP => methods::PROJECT_GET_MAP,
        methods::PROJECT_STATISTICS => methods::PROJECT_GET_STATISTICS,
        methods::SNAPSHOT_SAVE => methods::SNAPSHOT_CREATE,
        methods::SNAPSHOT_HISTORY => methods::SNAPSHOT_LIST,
        methods::SNAPSHOT_CONTENT => methods::SNAPSHOT_GET,
        methods::SNAPSHOT_RESTORE => methods::SNAPSHOT_RESTORE_V1,
        methods::SNAPSHOT_RESTORE_SYMBOL => methods::SNAPSHOT_RESTORE_SYMBOL_V1,
        methods::SYMBOL_HISTORY => methods::SYMBOL_GET_HISTORY,
        methods::SYMBOL_DIFF => methods::SYMBOL_GET_DIFF,
        methods::SYMBOL_FIND => methods::SYMBOL_SEARCH,
        methods::FILE_LIST => methods::FILE_GET_LIST,
        methods::FILE_SEARCH => methods::FILE_SEARCH_V1,
        methods::CONTENT_SEARCH => methods::CONTENT_SEARCH_V1,
        methods::FILE_DIFF => methods::FILE_GET_DIFF,
        methods::FILE_INFO => methods::FILE_GET_INFO,
        methods::BRANCH_LIST => methods::BRANCH_GET_LIST,
        methods::BRANCH_CURRENT => methods::BRANCH_GET_CURRENT,
        methods::SESSION_LIST => methods::SESSION_GET_LIST,
        methods::SESSION_ACTIVE => methods::SESSION_GET_ACTIVE,
        methods::SESSION_TIMESHEET => methods::SESSION_GET_TIMESHEET,
        methods::PROJECT_CHECKPOINT => methods::PROJECT_CREATE_CHECKPOINT,
        methods::PROJECT_REVERT => methods::PROJECT_REVERT_V1,
        methods::GC_RUN => methods::MAINTENANCE_GC,
        methods::CONFIG_GET => methods::CONFIG_GET_V1,
        methods::CONFIG_SET => methods::CONFIG_SET_V1,
        methods::TIER_CONFIG_GET => methods::TIER_CONFIG_GET_V1,
        methods::TIER_CONFIG_SET => methods::TIER_CONFIG_SET_V1,
        // Already standardized or unknown
        _ => method,
    }
}

// ---------------------------------------------------------------------------
// Typed request/response params
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct WatchParams {
    pub project_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UnwatchParams {
    pub project_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SnapshotSaveParams {
    pub file_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SnapshotHistoryParams {
    pub file_path: String,
    pub branch: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SnapshotContentParams {
    pub content_hash: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SnapshotActivityParams {
    pub limit: usize,
    pub project_path: Option<String>,
    pub branch: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SnapshotRestoreParams {
    pub content_hash: String,
    pub target_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SnapshotRestoreSymbolParams {
    pub content_hash: String,
    pub target_path: String,
    pub symbol_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SymbolHistoryParams {
    pub symbol_name: String,
    pub branch: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SymbolHistoryResponse {
    pub history: Vec<SymbolHistoryEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SymbolHistoryEntry {
    pub snapshot: SnapshotInfo,
    pub symbol_name: String,
    pub symbol_kind: String,
    pub structural_hash: String,
    pub start_line: usize,
    pub end_line: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SymbolLocation {
    pub name: String,
    pub kind: String,
    pub file_path: String,
    pub start_line: usize,
    pub end_line: usize,
    pub structural_hash: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SymbolDiffParams {
    pub file_path: String,
    pub symbol_name: String,
    pub base_hash: Option<String>,
    pub target_hash: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SymbolFindParams {
    pub query: String,
    pub project_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SemanticHistoryParams {
    pub symbol_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SemanticHistoryResponse {
    pub deltas: Vec<crate::models::SemanticRecord>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectMapParams {
    pub project_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectStatisticsParams {
    pub project_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectStatisticsResponse {
    pub total_snapshots: usize,
    pub total_files: usize,
    pub total_branches: usize,
    pub total_commits: usize,
    pub size_bytes: u64,
    pub last_activity: String,
    pub activity_by_day: Vec<(String, usize)>,
    pub activity_by_hour: Vec<(usize, usize)>,
    pub top_files: Vec<(String, usize)>,
    pub top_branches: Vec<(String, usize)>,
    pub extensions: Vec<(String, usize)>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileListParams {
    pub filter: Option<String>,
    pub branch: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileInfoResponse {
    pub path: String,
    pub snapshot_count: usize,
    pub total_bytes: u64,
    pub last_modified: String,
    pub first_seen: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileDiffParams {
    pub file_path: String,
    pub base_hash: Option<String>,
    pub target_hash: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileDiffResponse {
    pub diff: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileSearchParams {
    pub query: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContentSearchParams {
    pub query: String,
    pub path_filter: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectRevertParams {
    pub timestamp: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigSetParams {
    pub key: String,
    pub value: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WatchedProject {
    pub project_path: String,
    pub watched_at: String,
    pub last_activity: String,
    pub file_count: usize,
    pub snapshot_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetWatchedProjectsResponse {
    pub projects: Vec<WatchedProject>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TierConfig {
    pub hot_max_age_days: u64,
    pub warm_max_age_days: u64,
    pub hot_max_memory_mb: u64,
    pub compression_level: u32, // 0-21 for zstd
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TierConfigSetParams {
    pub config: TierConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TierConfigGetResponse {
    pub config: TierConfig,
}

// Responses

#[derive(Debug, Serialize, Deserialize)]
pub struct StatusResponse {
    pub version: String,
    pub uptime_secs: u64,
    pub watched_projects: Vec<String>,
    pub active_sessions: usize,
    #[serde(default)]
    pub history_size_bytes: u64,
    #[serde(default)]
    pub total_size_bytes: u64,
    pub avg_response_time_ms: f64,
    #[serde(default)]
    pub avg_save_time_ms: f64,
    #[serde(default)]
    pub total_saves: u64,
    #[serde(default)]
    pub total_snapshots: u64,
    #[serde(default)]
    pub total_symbols: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SnapshotInfo {
    pub id: i64,
    pub file_path: String,
    pub timestamp: String,
    pub content_hash: String,
    pub git_branch: Option<String>,
    pub commit_hash: Option<String>,
    pub commit_message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionResponse {
    pub id: i64,
    pub start_time: String,
    pub end_time: Option<String>,
    pub branch: Option<String>,
    pub file_count: usize,
    pub snapshot_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TimesheetEntry {
    pub date: String,
    pub branch: Option<String>,
    pub duration_minutes: u64,
    pub file_count: usize,
    pub snapshot_count: usize,
}

/// Parameters for starting the MCP server
#[derive(Debug, Serialize, Deserialize)]
pub struct McpStartParams {
    /// Optional: specific transport to use (default: stdio)
    #[serde(default)]
    pub transport: Option<String>,
}

/// Parameters for stopping the MCP server
#[derive(Debug, Serialize, Deserialize)]
pub struct McpStopParams {
    /// Force stop even if there are active connections
    #[serde(default)]
    pub force: bool,
}

/// Response for MCP server status
#[derive(Debug, Serialize, Deserialize)]
pub struct McpStatusResponse {
    pub running: bool,
    pub pid: Option<u32>,
}
