use std::path::PathBuf;
use std::sync::Arc;
use serde_json::json;
use log::{error, info, warn};
use std::sync::atomic::Ordering;

use mnem_core::env::get_base_dir;
use mnem_core::protocol::{self, JsonRpcRequest, JsonRpcResponse, PROTOCOL_VERSION};
use mnem_core::protocol::{InitializeResult, InitializeParams, ServerInfo, ServerCapabilities};
use mnem_core::protocol::jsonrpc_errors::*;
use mnem_core::protocol::mnem_errors::*;
use mnem_core::Repository;
use crate::Monitor;
use crate::state::{DaemonState, InitializationState};

/// List of methods that can be called before initialization
const UNRESTRICTED_METHODS: &[&str] = &[
    protocol::methods::INITIALIZE,
    protocol::methods::STATUS,
    protocol::methods::DAEMON_GET_STATUS,
    protocol::methods::PROJECT_RELOAD,
];

pub async fn handle_request(req: &JsonRpcRequest, state: &Arc<DaemonState>) -> JsonRpcResponse {
    let start_instant = std::time::Instant::now();
    
    // Normalize method name for backward compatibility
    let normalized_method = protocol::normalize_method_name(&req.method);
    
    // Check initialization state (except for unrestricted methods)
    if !UNRESTRICTED_METHODS.contains(&normalized_method) {
        if !state.is_initialized() {
            return JsonRpcResponse::error(
                req.id,
                SERVER_NOT_INITIALIZED,
                "Server not initialized. Call initialize first.".into()
            );
        }
        if state.is_shutdown() {
            return JsonRpcResponse::error(
                req.id,
                SHUTDOWN_IN_PROGRESS,
                "Server is shutting down.".into()
            );
        }
    }

    let response = match normalized_method {
        protocol::methods::INITIALIZE => {
            let current_state = *state.init_state.read();
            
            if current_state == InitializationState::Shutdown {
                return JsonRpcResponse::error(
                    req.id,
                    SHUTDOWN_IN_PROGRESS,
                    "Server is shutting down.".into()
                );
            }
            
            // If already initialized, just return the current capabilities successfully
            if current_state == InitializationState::Initialized {
                let capabilities = state.server_capabilities.read().clone().unwrap_or_default();
                let result = InitializeResult {
                    server_info: ServerInfo {
                        name: "mnemosyne".to_string(),
                        version: env!("CARGO_PKG_VERSION").to_string(),
                    },
                    capabilities,
                    protocol_version: PROTOCOL_VERSION.to_string(),
                };
                return JsonRpcResponse::success(req.id, serde_json::to_value(result).unwrap_or(json!({})));
            }
            
            {
                let mut init_lock = state.init_state.write();
                *init_lock = InitializationState::Initializing;
            }
            
            // Parse client capabilities
            let params: InitializeParams = match serde_json::from_value(req.params.clone()) {
                Ok(p) => p,
                Err(e) => {
                    let mut init_lock = state.init_state.write();
                    *init_lock = InitializationState::Uninitialized;
                    return JsonRpcResponse::error(
                        req.id,
                        INVALID_PARAMS,
                        format!("Invalid initialize params: {}", e)
                    );
                }
            };
            
            // Store client capabilities
            *state.client_capabilities.write() = Some(params.capabilities);
            
            // Build server capabilities
            let mut capabilities = ServerCapabilities::default();
            capabilities.supported_methods = vec![
                protocol::methods::PROJECT_WATCH.to_string(),
                protocol::methods::PROJECT_UNWATCH.to_string(),
                protocol::methods::PROJECT_LIST.to_string(),
                protocol::methods::SNAPSHOT_CREATE.to_string(),
                protocol::methods::SNAPSHOT_LIST.to_string(),
                protocol::methods::SNAPSHOT_GET.to_string(),
                protocol::methods::SYMBOL_GET_HISTORY.to_string(),
                protocol::methods::SYMBOL_SEARCH.to_string(),
                protocol::methods::CONTENT_SEARCH_V1.to_string(),
                protocol::methods::PROJECT_GET_STATISTICS.to_string(),
                protocol::methods::DAEMON_GET_STATUS.to_string(),
                protocol::methods::SYMBOL_GET_SEMANTIC_HISTORY.to_string(),
            ];
            *state.server_capabilities.write() = Some(capabilities.clone());
            
            {
                let mut init_lock = state.init_state.write();
                *init_lock = InitializationState::Initialized;
            }
            
            let result = InitializeResult {
                server_info: ServerInfo {
                    name: "mnemosyne".to_string(),
                    version: env!("CARGO_PKG_VERSION").to_string(),
                },
                capabilities,
                protocol_version: PROTOCOL_VERSION.to_string(),
            };
            
            info!("Client initialized: {:?}", params.client_info);
            JsonRpcResponse::success(req.id, serde_json::to_value(result).unwrap_or(json!({})))
        }

        protocol::methods::EXIT => {
            info!("Exit requested via RPC");
            tokio::spawn(async {
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                std::process::exit(0);
            });
            JsonRpcResponse::success(req.id, json!(null))
        }

        protocol::methods::STATUS | protocol::methods::DAEMON_GET_STATUS => {
            let total_reqs = state.total_requests.load(Ordering::Relaxed);
            let total_time = state.total_processing_time_us.load(Ordering::Relaxed);
            
            let avg_time = if total_reqs > 0 {
                (total_time as f64 / total_reqs as f64) / 1000.0
            } else {
                0.0
            };

            let total_saves = state.total_saves.load(Ordering::Relaxed);
            let total_save_time = state.total_save_time_us.load(Ordering::Relaxed);
            let avg_save_time = if total_saves > 0 {
                (total_save_time as f64 / total_saves as f64) / 1000.0
            } else {
                0.0
            };

            let mut total_snapshots = 0;
            let mut total_symbols = 0;
            for repo_entry in state.repos.iter() {
                let repo = repo_entry.value();
                total_snapshots += repo.db.get_snapshot_count().unwrap_or(0);
                total_symbols += repo.db.get_symbol_count().unwrap_or(0);
            }

            let total_size = state.calculate_total_size();

            let status = protocol::StatusResponse {
                version: env!("CARGO_PKG_VERSION").into(),
                uptime_secs: state.start_time.elapsed().as_secs(),
                watched_projects: state.monitors.iter().map(|m| m.key().clone()).collect(),
                active_sessions: 0,
                history_size_bytes: total_size,
                total_size_bytes: total_size,
                avg_response_time_ms: avg_time,
                avg_save_time_ms: avg_save_time,
                total_saves,
                total_snapshots: total_snapshots as u64,
                total_symbols: total_symbols as u64,
            };
            JsonRpcResponse::success(req.id, serde_json::to_value(status).unwrap_or(json!({})))
        }

        protocol::methods::PROJECT_RELOAD => {
            info!("Reloading projects from registry...");
            
            let base_dir = match get_base_dir() {
                Ok(d) => d,
                Err(e) => return JsonRpcResponse::error(req.id, -32603, format!("Failed to get base dir: {}", e)),
            };
            
            let registry = match mnem_core::storage::registry::ProjectRegistry::new(&base_dir) {
                Ok(r) => r,
                Err(e) => return JsonRpcResponse::error(req.id, -32603, format!("Failed to load registry: {}", e)),
            };
            
            let projects = registry.list_projects();
            let total = projects.len();
            let mut loaded = 0;
            
            for project in projects {
                let project_path = std::path::PathBuf::from(&project.path);
                if !project_path.exists() {
                    continue;
                }
                
                // Skip if already loaded
                if state.repos.contains_key(&project.path) {
                    loaded += 1;
                    continue;
                }
                
                match Repository::open(base_dir.clone(), project_path.clone()) {
                    Ok(repo) => {
                        let repo = Arc::new(repo);
                        let monitor = Arc::new(Monitor::with_state(project_path.clone(), repo.clone(), state.clone()));
                        
                        let scan_path = project.path.clone();
                        let monitor_scan = monitor.clone();
                        tokio::task::spawn_blocking(move || {
                            if let Err(e) = monitor_scan.initial_scan() {
                                error!("Initial scan failed for {}: {}", scan_path, e);
                            }
                        });
                        
                        let monitor_start = monitor.clone();
                        tokio::spawn(async move {
                            if let Err(e) = monitor_start.start().await {
                                error!("Monitor loop failed: {}", e);
                            }
                        });
                        
                        state.repos.insert(project.path.clone(), repo);
                        state.monitors.insert(project.path.clone(), monitor);
                        loaded += 1;
                        info!("Reloaded project: {}", project.path);
                    }
                    Err(e) => {
                        warn!("Failed to reload project {}: {}", project.path, e);
                    }
                }
            }
            
            JsonRpcResponse::success(req.id, json!({"loaded": loaded, "total": total}))
        }

        protocol::methods::WATCH | protocol::methods::PROJECT_WATCH => {
            let params: protocol::WatchParams = match serde_json::from_value(req.params.clone()) {
                Ok(p) => p,
                Err(e) => return JsonRpcResponse::error(req.id, -32602, format!("Invalid params: {}", e)),
            };

            let path = std::path::Path::new(&params.project_path);
            let blacklist = ["/", "/Users", "/Users/", "/var", "/tmp", "/etc", "/bin", "/sbin", "/usr"];
            if blacklist.contains(&params.project_path.as_str()) || path.parent().is_none() {
                return JsonRpcResponse::error(req.id, -32602, "Protected path cannot be watched".into());
            }

            if state.monitors.contains_key(&params.project_path) {
                return JsonRpcResponse::success(req.id, json!({"status": "already_watching"}));
            }

            let base_dir = match get_base_dir() {
                Ok(dir) => dir,
                Err(e) => return JsonRpcResponse::error(req.id, -32000, format!("Failed to get base directory: {}", e)),
            };
            let project_path = PathBuf::from(&params.project_path);

            match Repository::open(base_dir, project_path.clone()) {
                Ok(repo) => {
                    let repo = Arc::new(repo);
                    let monitor = Arc::new(Monitor::with_state(project_path, repo.clone(), state.clone()));

                    let monitor_scan = monitor.clone();
                    tokio::task::spawn_blocking(move || {
                        if let Err(e) = monitor_scan.initial_scan() {
                            error!("Initial scan failed: {}", e);
                        }
                    });

                    let monitor_start = monitor.clone();
                    tokio::spawn(async move {
                        if let Err(e) = monitor_start.start().await {
                            error!("Monitor loop failed: {}", e);
                        }
                    });

                    state.repos.insert(params.project_path.clone(), repo);
                    state.monitors.insert(params.project_path.clone(), monitor);

                    info!("Watching project: {}", params.project_path);
                    JsonRpcResponse::success(req.id, json!({"status": "watching"}))
                }
                Err(e) => JsonRpcResponse::error(req.id, -32000, format!("Failed to open repo: {}", e)),
            }
        }

        protocol::methods::UNWATCH | protocol::methods::PROJECT_UNWATCH => {
            let params: protocol::UnwatchParams = match serde_json::from_value(req.params.clone()) {
                Ok(p) => p,
                Err(e) => return JsonRpcResponse::error(req.id, -32602, format!("Invalid params: {}", e)),
            };

            state.monitors.remove(&params.project_path);
            state.repos.remove(&params.project_path);
            info!("Unwatched project: {}", params.project_path);
            JsonRpcResponse::success(req.id, json!({"status": "unwatched"}))
        }

        protocol::methods::FILE_LIST | protocol::methods::FILE_GET_LIST => {
            let params: protocol::FileListParams = match serde_json::from_value(req.params.clone()) {
                Ok(p) => p,
                Err(e) => return JsonRpcResponse::error(req.id, -32602, format!("Invalid params: {}", e)),
            };

            let mut files = Vec::new();
            for repo_entry in state.repos.iter() {
                let repo = repo_entry.value();
                match repo.list_files(params.filter.as_deref(), params.branch.as_deref()) {
                    Ok(f) => {
                        for entry in f {
                            files.push(json!({
                                "path": entry.path,
                                "snapshot_count": 0,
                                "last_modified": entry.last_update,
                            }));
                        }
                    }
                    Err(e) => error!("Failed to list files for {}: {}", repo.project.path, e),
                }
            }
            JsonRpcResponse::success(req.id, serde_json::to_value(files).unwrap_or(json!([])))
        }

        protocol::methods::SNAPSHOT_HISTORY | protocol::methods::SNAPSHOT_LIST => {
            let params: protocol::SnapshotHistoryParams = match serde_json::from_value(req.params.clone()) {
                Ok(p) => p,
                Err(e) => return JsonRpcResponse::error(req.id, -32602, format!("Invalid params: {}", e)),
            };

            let file_path = std::path::Path::new(&params.file_path);
            for repo_entry in state.repos.iter() {
                let repo = repo_entry.value();
                let project_path = std::path::Path::new(&repo.project.path);
                if file_path.starts_with(project_path) {
                    // Try cache first
                    if let Some(cached) = state.get_cached_history(&params.file_path) {
                        let infos: Vec<protocol::SnapshotInfo> = cached
                            .into_iter()
                            .filter(|sn| {
                                if let Some(ref b) = params.branch {
                                    sn.git_branch.as_ref() == Some(b)
                                } else {
                                    true
                                }
                            })
                            .map(|sn| {
                                let commit_message = sn.commit_hash.as_ref().and_then(|h| {
                                    repo.db.get_git_commit(h).ok().flatten().map(|(msg, _, _)| msg)
                                });
                                protocol::SnapshotInfo {
                                    id: sn.id,
                                    file_path: sn.file_path,
                                    timestamp: sn.timestamp,
                                    content_hash: sn.content_hash,
                                    git_branch: sn.git_branch,
                                    commit_hash: sn.commit_hash,
                                    commit_message,
                                }
                            })
                            .collect();
                        return JsonRpcResponse::success(req.id, serde_json::to_value(infos).unwrap_or(json!([])));
                    }

                    // Cache miss - fetch from database
                    match repo.get_history(&params.file_path) {
                        Ok(history) => {
                            // Cache the result
                            state.cache_history(params.file_path.clone(), history.clone());
                            
                            let infos: Vec<protocol::SnapshotInfo> = history
                                .into_iter()
                                .filter(|sn| {
                                    if let Some(ref b) = params.branch {
                                        sn.git_branch.as_ref() == Some(b)
                                    } else {
                                        true
                                    }
                                })
                                .map(|sn| {
                                    let commit_message = sn.commit_hash.as_ref().and_then(|h| {
                                        repo.db.get_git_commit(h).ok().flatten().map(|(msg, _, _)| msg)
                                    });
                                    protocol::SnapshotInfo {
                                        id: sn.id,
                                        file_path: sn.file_path,
                                        timestamp: sn.timestamp,
                                        content_hash: sn.content_hash,
                                        git_branch: sn.git_branch,
                                        commit_hash: sn.commit_hash,
                                        commit_message,
                                    }
                                })
                                .collect();
                            return JsonRpcResponse::success(req.id, serde_json::to_value(infos).unwrap_or(json!([])));
                        }
                        Err(e) => return JsonRpcResponse::error(req.id, -32000, e.to_string()),
                    }
                }
            }
            JsonRpcResponse::error(req.id, -32000, "No watched project owns this file".into())
        }

        protocol::methods::SNAPSHOT_CONTENT | protocol::methods::SNAPSHOT_GET => {
            let params: protocol::SnapshotContentParams = match serde_json::from_value(req.params.clone()) {
                Ok(p) => p,
                Err(e) => return JsonRpcResponse::error(req.id, -32602, format!("Invalid params: {}", e)),
            };

            for repo_entry in state.repos.iter() {
                let repo = repo_entry.value();
                match repo.get_content(&params.content_hash) {
                    Ok(content) => {
                        let text = String::from_utf8_lossy(&content).to_string();
                        return JsonRpcResponse::success(req.id, json!({"content": text}));
                    }
                    Err(_) => continue,
                }
            }
            JsonRpcResponse::error(req.id, -32000, "Content not found".into())
        }

        protocol::methods::CONTENT_SEARCH | protocol::methods::CONTENT_SEARCH_V1 => {
            let params: protocol::ContentSearchParams = match serde_json::from_value(req.params.clone()) {
                Ok(p) => p,
                Err(e) => return JsonRpcResponse::error(req.id, -32602, format!("Invalid params: {}", e)),
            };

            let mut all_results = Vec::new();

            for repo_entry in state.repos.iter() {
                let repo = repo_entry.value();
                match repo.grep_contents(&params.query, params.path_filter.as_deref()) {
                    Ok(results) => all_results.extend(results),
                    Err(e) => error!("Search failed for {}: {}", repo.project.path, e),
                }
            }

            all_results.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
            if let Some(limit) = params.limit {
                all_results.truncate(limit);
            }

            JsonRpcResponse::success(req.id, json!({ "results": all_results }))
        }

        protocol::methods::SYMBOL_FIND | protocol::methods::SYMBOL_SEARCH => {
            let params: protocol::SymbolFindParams = match serde_json::from_value(req.params.clone()) {
                Ok(p) => p,
                Err(e) => return JsonRpcResponse::error(req.id, -32602, format!("Invalid params: {}", e)),
            };

            let mut all_symbols = Vec::new();

            for repo_entry in state.repos.iter() {
                let repo = repo_entry.value();
                if let Some(ref filter_path) = params.project_path {
                    if repo.project.path != *filter_path {
                        continue;
                    }
                }

                match repo.find_symbols(&params.query) {
                    Ok(symbols) => all_symbols.extend(symbols),
                    Err(e) => error!("Symbol find failed for {}: {}", repo.project.path, e),
                }
            }

            JsonRpcResponse::success(req.id, serde_json::to_value(all_symbols).unwrap_or(json!([])))
        }

        protocol::methods::PROJECT_ACTIVITY | protocol::methods::PROJECT_GET_ACTIVITY => {
            let params: protocol::SnapshotActivityParams = match serde_json::from_value(req.params.clone()) {
                Ok(p) => p,
                Err(e) => return JsonRpcResponse::error(req.id, -32602, format!("Invalid params: {}", e)),
            };

            let mut all_activity = Vec::new();

            for repo_entry in state.repos.iter() {
                let repo = repo_entry.value();
                if let Some(ref filter_path) = params.project_path {
                    if repo.project.path != *filter_path {
                        continue;
                    }
                }

                if let Ok(history) = repo.db.get_global_history(params.limit) {
                    for sn in history {
                        let commit_message = sn.commit_hash.as_ref().and_then(|h| {
                            repo.db.get_git_commit(h).ok().flatten().map(|(msg, _, _)| msg)
                        });
                        all_activity.push(protocol::SnapshotInfo {
                            id: sn.id,
                            file_path: sn.file_path,
                            timestamp: sn.timestamp,
                            content_hash: sn.content_hash,
                            git_branch: sn.git_branch,
                            commit_hash: sn.commit_hash,
                            commit_message,
                        });
                    }
                }
            }

            all_activity.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
            all_activity.truncate(params.limit);

            JsonRpcResponse::success(req.id, serde_json::to_value(all_activity).unwrap_or(json!([])))
        }

        protocol::methods::PROJECT_MAP | protocol::methods::PROJECT_GET_MAP => {
            let params: protocol::ProjectMapParams = match serde_json::from_value(req.params.clone()) {
                Ok(p) => p,
                Err(e) => return JsonRpcResponse::error(req.id, -32602, format!("Invalid params: {}", e)),
            };

            if let Some(repo_entry) = state.repos.get(&params.project_path) {
                let repo = repo_entry.value();
                let files = match repo.list_files(None, None) {
                    Ok(f) => f,
                    Err(e) => return JsonRpcResponse::error(req.id, -32000, e.to_string()),
                };

                let mut files_list = Vec::new();
                let mut symbols_map = json!({});
                for f in files {
                    files_list.push(json!(f.path));
                    if let Ok(snaps) = repo.db.get_history(&f.path) {
                        if let Some(latest) = snaps.first() {
                            if let Ok(symbols) = repo.db.get_symbols_for_snapshot(latest.id) {
                                symbols_map[&f.path] = json!(symbols
                                    .into_iter()
                                    .map(|sym| {
                                        json!({
                                            "name": sym.name,
                                            "kind": sym.kind,
                                            "start_line": sym.start_line,
                                            "end_line": sym.end_line
                                        })
                                    })
                                    .collect::<Vec<_>>());
                            }
                        }
                    }
                }
                let map = json!({ "files": files_list, "symbols": symbols_map });
                return JsonRpcResponse::success(req.id, map);
            }
            JsonRpcResponse::error(req.id, -32000, "Project not watched".into())
        }

        protocol::methods::PROJECT_STATISTICS | protocol::methods::PROJECT_GET_STATISTICS => {
            let params: protocol::ProjectStatisticsParams = match serde_json::from_value(req.params.clone()) {
                Ok(p) => p,
                Err(e) => return JsonRpcResponse::error(req.id, -32602, format!("Invalid params: {}", e)),
            };

            let mut target_repo_arc = None;
            
            if let Some(ref path) = params.project_path {
                target_repo_arc = state.repos.get(path).map(|r| r.value().clone());
            } else if state.repos.len() == 1 {
                target_repo_arc = state.repos.iter().next().map(|r| r.value().clone());
            }

            if let Some(repo) = target_repo_arc {
                let total_snapshots = repo.db.get_snapshot_count().unwrap_or(0);
                let total_files = repo.db.get_recent_files(1000, None, None).unwrap_or_default().len();
                let last_activity = repo.db.get_global_history(1).ok()
                    .and_then(|h| h.first().map(|sn| sn.timestamp.clone()))
                    .unwrap_or_default();

                let resp = protocol::ProjectStatisticsResponse {
                    total_snapshots,
                    total_files,
                    total_branches: repo.list_branches().unwrap_or_default().len(),
                    total_commits: 0,
                    size_bytes: 0,
                    last_activity,
                    activity_by_day: Vec::new(),
                    activity_by_hour: Vec::new(),
                    top_files: Vec::new(),
                    top_branches: Vec::new(),
                    extensions: Vec::new(),
                };
                return JsonRpcResponse::success(req.id, serde_json::to_value(resp).unwrap_or(json!({})));
            }
            JsonRpcResponse::error(req.id, -32000, "No project selected or found".into())
        }

        protocol::methods::FILE_INFO | protocol::methods::FILE_GET_INFO => {
            let params: serde_json::Value = req.params.clone();
            let file_path = params["file_path"].as_str().unwrap_or("");
            
            for repo_entry in state.repos.iter() {
                let repo = repo_entry.value();
                if file_path.starts_with(&repo.project.path) {
                    if let Ok(history) = repo.db.get_history(file_path) {
                        if let Some(latest) = history.first() {
                            let info = json!({
                                "path": file_path,
                                "snapshot_count": history.len(),
                                "total_size_human": "Unknown",
                                "earliest": history.last().map(|s| s.timestamp.clone()).unwrap_or_default(),
                                "latest": latest.timestamp.clone(),
                            });
                            return JsonRpcResponse::success(req.id, info);
                        }
                    }
                }
            }
            JsonRpcResponse::error(req.id, -32000, "File not found in any project".into())
        }

        protocol::methods::FILE_DIFF | protocol::methods::FILE_GET_DIFF => {
            let params: protocol::FileDiffParams = match serde_json::from_value(req.params.clone()) {
                Ok(p) => p,
                Err(e) => return JsonRpcResponse::error(req.id, -32602, format!("Invalid params: {}", e)),
            };

            for repo_entry in state.repos.iter() {
                let repo = repo_entry.value();
                if params.file_path.starts_with(&repo.project.path) {
                    match repo.get_file_diff(&params.file_path, params.base_hash.as_deref(), &params.target_hash) {
                        Ok(diff) => return JsonRpcResponse::success(req.id, json!({ "diff": diff })),
                        Err(e) => return JsonRpcResponse::error(req.id, -32000, e.to_string()),
                    }
                }
            }
            JsonRpcResponse::error(req.id, -32000, "File not found".into())
        }

        protocol::methods::SYMBOL_HISTORY | protocol::methods::SYMBOL_GET_HISTORY => {
            let params: protocol::SymbolHistoryParams = match serde_json::from_value(req.params.clone()) {
                Ok(p) => p,
                Err(e) => return JsonRpcResponse::error(req.id, INVALID_PARAMS, format!("Invalid params: {}", e)),
            };

            let mut history = Vec::new();
            for repo_entry in state.repos.iter() {
                let repo = repo_entry.value();
                if let Ok(h) = repo.db.get_symbol_history(&params.symbol_name) {
                    if !h.is_empty() {
                        for (sn, sym) in h {
                            let commit_message = sn.commit_hash.as_ref().and_then(|h| {
                                repo.db.get_git_commit(h).ok().flatten().map(|(msg, _, _)| msg)
                            });
                            history.push(json!({
                                "snapshot": protocol::SnapshotInfo {
                                    id: sn.id,
                                    file_path: sn.file_path,
                                    timestamp: sn.timestamp,
                                    content_hash: sn.content_hash,
                                    git_branch: sn.git_branch,
                                    commit_hash: sn.commit_hash,
                                    commit_message,
                                },
                                "symbol_name": sym.name,
                                "symbol_kind": sym.kind,
                                "structural_hash": sym.structural_hash,
                                "start_line": sym.start_line,
                                "end_line": sym.end_line,
                            }));
                        }
                        break;
                    }
                }
            }
            JsonRpcResponse::success(req.id, json!({ "history": history }))
        }

        protocol::methods::SYMBOL_GET_SEMANTIC_HISTORY => {
            let params: protocol::SemanticHistoryParams = match serde_json::from_value(req.params.clone()) {
                Ok(p) => p,
                Err(e) => return JsonRpcResponse::error(req.id, INVALID_PARAMS, format!("Invalid params: {}", e)),
            };

            let mut all_deltas = Vec::new();
            for repo_entry in state.repos.iter() {
                let repo = repo_entry.value();
                if let Ok(deltas) = repo.db.get_symbol_deltas(&params.symbol_name) {
                    all_deltas.extend(deltas);
                }
            }
            
            JsonRpcResponse::success(req.id, json!({ "deltas": all_deltas }))
        }

        protocol::methods::GET_WATCHED_PROJECTS | protocol::methods::PROJECT_LIST => {
            let projects: Vec<protocol::WatchedProject> = state.monitors.iter().map(|m| {
                protocol::WatchedProject {
                    project_path: m.key().clone(),
                    watched_at: state.start_time.elapsed().as_secs().to_string(),
                    last_activity: state.start_time.elapsed().as_secs().to_string(),
                    file_count: 0,
                    snapshot_count: 0,
                }
            }).collect();
            
            let resp = protocol::GetWatchedProjectsResponse { projects };
            JsonRpcResponse::success(req.id, serde_json::to_value(resp).unwrap_or(json!({"projects": []})))
        }

        protocol::methods::SHUTDOWN => {
            info!("Shutdown requested via RPC");
            {
                let mut init_lock = state.init_state.write();
                *init_lock = InitializationState::Shutdown;
            }
            tokio::spawn(async {
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                std::process::exit(0);
            });
            JsonRpcResponse::success(req.id, json!({"status": "shutting_down"}))
        }

        _ => JsonRpcResponse::error(req.id, METHOD_NOT_FOUND, format!("Method not found: {}", req.method)),
    };
    
    state.record_request(start_instant.elapsed().as_micros() as u64);
    response
}

