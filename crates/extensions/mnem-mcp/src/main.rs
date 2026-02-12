use anyhow::{Context, Result};
use mnem_core::{client::DaemonClient, protocol::methods};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::Path;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: Value,
    method: String,
    #[serde(default)]
    params: Value,
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcError {
    code: i32,
    message: String,
}

// ---------------------------------------------------------------------------
// Path Resolution
// ---------------------------------------------------------------------------

/// Fetches the list of watched project paths from the daemon.
fn get_watched_paths(client: &mut DaemonClient) -> Result<Vec<String>> {
    let res = client.call(methods::PROJECT_LIST, json!({}))?;
    let projects = res["projects"].as_array().cloned().unwrap_or_default();


    Ok(projects
        .iter()
        .filter_map(|p| p["project_path"].as_str().map(String::from))
        .collect())
}

/// Resolves a potentially partial/relative project path to the actual
/// absolute path registered in the daemon.
///
/// Matching strategy (first match wins):
/// 1. Exact match against watched paths
/// 2. The query is a suffix of a watched path (e.g. "Gemini" matches ".../Gemini")
/// 3. The directory name of a watched path contains the query (case-insensitive)
/// 4. If only one project is watched, return it as default
fn resolve_project_path(client: &mut DaemonClient, raw: &str) -> Result<String> {
    let watched = get_watched_paths(client)?;

    if watched.is_empty() {
        anyhow::bail!(
            "No projects are currently watched by the daemon. \
             Use 'mnem watch <path>' to start watching a project."
        );
    }

    // 1. Exact match
    if watched.contains(&raw.to_string()) {
        return Ok(raw.to_string());
    }

    // 2. Suffix match (e.g. "Gemini" or "ChronosLocalHistory/Gemini")
    for path in &watched {
        if path.ends_with(raw) || path.ends_with(&format!("/{}", raw)) {
            return Ok(path.clone());
        }
    }

    // 3. Directory name contains query (case-insensitive fuzzy)
    let query_lower = raw.to_lowercase();
    for path in &watched {
        let dir_name = Path::new(path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        if dir_name.to_lowercase().contains(&query_lower) {
            return Ok(path.clone());
        }
    }

    // 4. Single project fallback
    if watched.len() == 1 {
        return Ok(watched[0].clone());
    }

    anyhow::bail!(
        "Cannot resolve '{}' to a watched project. Available projects: {}",
        raw,
        watched.join(", ")
    )
}

/// Resolves a file_path that might be relative to a watched project.
/// If the path is already absolute, returns it as-is.
/// If relative, prepends the matching project root.
fn resolve_file_path(client: &mut DaemonClient, raw: &str) -> Result<String> {
    if Path::new(raw).is_absolute() {
        return Ok(raw.to_string());
    }

    let watched = get_watched_paths(client)?;

    // Single project: prepend its root
    if watched.len() == 1 {
        let full = format!("{}/{}", watched[0], raw);
        return Ok(full);
    }

    // Try to find which project contains this relative path on disk
    for project in &watched {
        let candidate = format!("{}/{}", project, raw);
        if Path::new(&candidate).exists() {
            return Ok(candidate);
        }
    }

    // Fallback: use first project
    if let Some(first) = watched.first() {
        return Ok(format!("{}/{}", first, raw));
    }

    anyhow::bail!(
        "Cannot resolve relative file path '{}': no watched projects found.",
        raw
    )
}

// ---------------------------------------------------------------------------
// Main loop
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() -> Result<()> {
    let stdin = BufReader::new(tokio::io::stdin());
    let mut lines = stdin.lines();
    let mut stdout = tokio::io::stdout();

    while let Some(line) = lines.next_line().await? {
        let req: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(_) => continue,
        };

        let response = handle_request(req).await;
        let res_line = serde_json::to_string(&response)? + "\n";
        stdout.write_all(res_line.as_bytes()).await?;
        stdout.flush().await?;
    }

    Ok(())
}

async fn handle_request(req: JsonRpcRequest) -> JsonRpcResponse {
    let result = match req.method.as_str() {
        "initialize" => Ok(json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {}
            },
            "serverInfo": { "name": "mnem-mcp", "version": "0.2.0" }
        })),
        "notifications/initialized" => {
            return JsonRpcResponse {
                jsonrpc: "2.0".into(),
                id: Value::Null,
                result: None,
                error: None,
            }
        }
        "tools/list" => Ok(tools_list()),
        "tools/call" => handle_tool_call(req.params).await,
        _ => Err(anyhow::anyhow!("Method not found")),
    };

    match result {
        Ok(res) => JsonRpcResponse {
            jsonrpc: "2.0".into(),
            id: req.id,
            result: Some(res),
            error: None,
        },
        Err(e) => JsonRpcResponse {
            jsonrpc: "2.0".into(),
            id: req.id,
            result: None,
            error: Some(JsonRpcError {
                code: -32000,
                message: e.to_string(),
            }),
        },
    }
}

// ---------------------------------------------------------------------------
// Tool definitions
// ---------------------------------------------------------------------------

fn tools_list() -> Value {
    json!({
        "tools": [
            {
                "name": "mnem_list_projects",
                "description": "List all projects currently watched by Mnemosyne. Call this FIRST to discover available project paths before using other tools. Returns the absolute path of each watched project.",
                "inputSchema": {
                    "type": "object",
                    "properties": {}
                }
            },
            {
                "name": "mnem_get_file_versions",
                "description": "Get snapshot history (hash, timestamp, branch) for a file. The file_path can be relative to the project root or an absolute path. Returns an array of snapshots with metadata including content hash, timestamp, branch, and commit message.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "file_path": { "type": "string", "description": "Path to the file. Can be relative (e.g. 'src/main.rs') or absolute." },
                        "limit": { "type": "integer", "default": 10, "description": "Maximum number of snapshots to return" }
                    },
                    "required": ["file_path"]
                }
            },
            {
                "name": "mnem_get_file_content",
                "description": "Get the text content of a specific snapshot by its content hash. Returns the full file content as plain text.",
                "inputSchema": {
                    "type": "object",
                    "properties": { "content_hash": { "type": "string", "description": "The hash of the snapshot to retrieve" } },
                    "required": ["content_hash"]
                }
            },
            {
                "name": "mnem_restore_file_version",
                "description": "Revert a file to a previous snapshot version. The file_path can be relative or absolute. This operation overwrites the current file with the historical version.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "file_path": { "type": "string", "description": "Path to the file. Can be relative or absolute." },
                        "target_hash": { "type": "string", "description": "The content hash of the snapshot to restore" }
                    },
                    "required": ["file_path", "target_hash"]
                }
            },
            {
                "name": "mnem_get_symbol_versions",
                "description": "Get the semantic evolution of a symbol (function, struct, etc.) over time across all watched projects. Returns all versions of the symbol with their definitions, locations, and timestamps.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "symbol_name": { "type": "string", "description": "The name of the symbol to search for" },
                        "limit": { "type": "integer", "default": 10, "description": "Maximum number of versions to return" }
                    },
                    "required": ["symbol_name"]
                }
            },
            {
                "name": "mnem_restore_symbol_version",
                "description": "Surgically restore a single symbol from a previous snapshot. The file_path can be relative or absolute. This operation modifies only the specified symbol in the current file, preserving other code.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "file_path": { "type": "string", "description": "Path to the file. Can be relative or absolute." },
                        "target_hash": { "type": "string", "description": "The content hash of the snapshot containing the symbol" },
                        "symbol_name": { "type": "string", "description": "The name of the symbol to restore" }
                    },
                    "required": ["file_path", "target_hash", "symbol_name"]
                }
            },
            {
                "name": "mnem_get_file_diff",
                "description": "Get a text diff between two snapshots, or between a snapshot and the current file on disk. Use '__DISK__' as target_hash to compare against the current file. Returns a unified diff format showing changes.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "file_path": { "type": "string", "description": "Path to the file. Can be relative or absolute." },
                        "base_hash": { "type": "string", "description": "Hash of the base snapshot, or null for auto-detect" },
                        "target_hash": { "type": "string", "description": "Hash of the target snapshot, or '__DISK__' for current file" }
                    },
                    "required": ["file_path", "target_hash"]
                }
            },
            {
                "name": "mnem_find_symbols",
                "description": "Search for symbols (functions, structs, etc.) by name or pattern across watched projects. The project_path is optional and can be a partial name (e.g. 'Gemini'). Returns all matching symbols with their locations and definitions.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "query": { "type": "string", "description": "The search query for symbol names" },
                        "project_path": { "type": "string", "description": "Optional. Partial project name or full path to filter. Resolved automatically." }
                    },
                    "required": ["query"]
                }
            },
            {
                "name": "mnem_get_project_structure",
                "description": "Get the semantic map of a project (files and symbols). The project_path can be a partial name (e.g. 'Gemini') and will be resolved to the full watched path. Returns a hierarchical structure of files and their symbols.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "project_path": { "type": "string", "description": "Partial project name or full path. Resolved automatically." }
                    },
                    "required": ["project_path"]
                }
            },
            {
                "name": "mnem_search_content",
                "description": "Powerful grep across all project history. Searches all snapshots for a text query. Returns matching lines, file paths, and content hashes for further exploration. Supports regular expressions.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "query": { "type": "string", "description": "The text to search for in all file versions." },
                        "limit": { "type": "integer", "description": "Optional maximum number of results to return.", "default": 50 }
                    },
                    "required": ["query"]
                }
            }
        ]
    })
}

// ---------------------------------------------------------------------------
// Tool dispatch
// ---------------------------------------------------------------------------

async fn handle_tool_call(params: Value) -> Result<Value> {
    let name = params["name"].as_str().context("Missing tool name")?;
    let args = &params["arguments"];
    let mut client = DaemonClient::connect()?;

    match name {
        "mnem_list_projects" => {
            let res = client.call(methods::PROJECT_LIST, json!({}))?;
            let text = serde_json::to_string_pretty(&res)?;
            Ok(mcp_text(&text))
        }

        "mnem_get_file_versions" => {
            let raw_path = args["file_path"].as_str().context("file_path required")?;
            let path = resolve_file_path(&mut client, raw_path)?;
            let res = client.call(methods::SNAPSHOT_LIST, json!({ "file_path": path }))?;
            Ok(mcp_text(&serde_json::to_string_pretty(&res)?))
        }

        "mnem_get_file_content" => {
            let hash = args["content_hash"]
                .as_str()
                .context("content_hash required")?;
            let res = client.call(methods::SNAPSHOT_GET, json!({ "content_hash": hash }))?;
            let content = res["content"].as_str().unwrap_or("");
            Ok(mcp_text(content))
        }

        "mnem_restore_file_version" => {
            let raw_path = args["file_path"].as_str().context("file_path required")?;
            let path = resolve_file_path(&mut client, raw_path)?;
            let hash = args["target_hash"]
                .as_str()
                .context("target_hash required")?;
            client.call(
                methods::SNAPSHOT_RESTORE_V1,
                json!({ "target_path": path, "content_hash": hash }),
            )?;
            Ok(mcp_text(&format!("File {} restored to {}", path, hash)))
        }

        "mnem_get_symbol_versions" => {
            let symbol = args["symbol_name"]
                .as_str()
                .context("symbol_name required")?;
            let res = client.call(methods::SYMBOL_GET_HISTORY, json!({ "symbol_name": symbol }))?;
            Ok(mcp_text(&serde_json::to_string_pretty(&res)?))
        }

        "mnem_restore_symbol_version" => {
            let raw_path = args["file_path"].as_str().context("file_path required")?;
            let path = resolve_file_path(&mut client, raw_path)?;
            let hash = args["target_hash"]
                .as_str()
                .context("target_hash required")?;
            let symbol = args["symbol_name"]
                .as_str()
                .context("symbol_name required")?;
            client.call(
                methods::SNAPSHOT_RESTORE_SYMBOL_V1,
                json!({ "target_path": path, "content_hash": hash, "symbol_name": symbol }),
            )?;
            Ok(mcp_text(&format!(
                "Symbol '{}' restored successfully.",
                symbol
            )))
        }

        "mnem_get_file_diff" => {
            let raw_path = args["file_path"].as_str().context("file_path required")?;
            let path = resolve_file_path(&mut client, raw_path)?;
            let base = args["base_hash"].as_str();
            let target = args["target_hash"]
                .as_str()
                .context("target_hash required")?;
            let res = client.call(
                methods::FILE_GET_DIFF,
                json!({ "file_path": path, "base_hash": base, "target_hash": target }),
            )?;
            Ok(mcp_text(res["diff"].as_str().unwrap_or("")))
        }

        "mnem_find_symbols" => {
            let query = args["query"].as_str().context("query required")?;
            let resolved_path = match args["project_path"].as_str() {
                Some(raw) => Some(resolve_project_path(&mut client, raw)?),
                None => None,
            };
            let res = client.call(
                methods::SYMBOL_SEARCH,
                json!({ "query": query, "project_path": resolved_path }),
            )?;
            Ok(mcp_text(&serde_json::to_string_pretty(&res)?))
        }

        "mnem_get_project_structure" => {
            let raw_path = args["project_path"]
                .as_str()
                .context("project_path required")?;
            let path = resolve_project_path(&mut client, raw_path)?;
            let res = client.call(methods::PROJECT_GET_MAP, json!({ "project_path": path }))?;
            Ok(mcp_text(&serde_json::to_string_pretty(&res)?))
        }

        "mnem_search_content" => {
            let query = args["query"].as_str().context("query required")?;
            let limit = args["limit"].as_u64();
            let res = client.call(
                methods::CONTENT_SEARCH_V1,
                json!({ "query": query, "limit": limit }),
            )?;
            Ok(mcp_text(&serde_json::to_string_pretty(&res)?))
        }


        _ => Err(anyhow::anyhow!("Unknown tool: {}", name)),
    }
}

/// Wraps text in the MCP content response format.
fn mcp_text(text: &str) -> Value {
    json!({ "content": [{ "type": "text", "text": text }] })
}
