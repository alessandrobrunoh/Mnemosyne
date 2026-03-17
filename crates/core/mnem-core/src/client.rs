use crate::env::get_base_dir;
use crate::error::{AppError, AppResult};
use crate::process::is_process_running;
use crate::protocol::{JsonRpcRequest, JsonRpcResponse, PID_FILE};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

#[cfg(unix)]
use std::os::unix::net::UnixStream;

#[cfg(unix)]
type IpcStream = UnixStream;

#[cfg(windows)]
type IpcStream = std::fs::File;

/// JSON-RPC client that talks to the mnem-daemon daemon over IPC (Unix Socket or Named Pipe).
pub struct DaemonClient {
    stream: IpcStream,
    next_id: AtomicU64,
    auth_token: Option<String>,
}

impl DaemonClient {
    /// Connect to the running mnem-daemon daemon.
    pub fn connect() -> AppResult<Self> {
        let base_dir = get_base_dir()?;
        let socket_path = crate::protocol::get_socket_path(&base_dir);
        let mut client = Self::connect_to(socket_path)?;
        client.initialize()?;
        Ok(client)
    }

    /// Connect to a specific socket path (useful for testing).
    pub fn connect_to(socket_path: PathBuf) -> AppResult<Self> {
        #[cfg(unix)]
        {
            let stream = UnixStream::connect(&socket_path).map_err(|e| {
                AppError::Internal(format!(
                    "Cannot connect to mnem-daemon at {:?}: {}. Is the daemon running?",
                    socket_path, e
                ))
            })?;
            stream
                .set_read_timeout(Some(std::time::Duration::from_secs(30)))
                .map_err(AppError::IoGeneric)?;
            let auth_token = crate::utils::auth::AuthManager::get_token().ok();
            let client = Self {
                stream,
                next_id: AtomicU64::new(1),
                auth_token,
            };

            Ok(client)
        }
        #[cfg(windows)]
        {
            use std::fs::OpenOptions;
            let stream = OpenOptions::new()
                .read(true)
                .write(true)
                .open(&socket_path)
                .map_err(|e| {
                    AppError::Internal(format!(
                        "Cannot connect to mnem-daemon named pipe at {:?}: {}. Is the daemon running?",
                        socket_path, e
                    ))
                })?;
            let auth_token = crate::utils::auth::AuthManager::get_token().ok();
            let client = Self {
                stream,
                next_id: AtomicU64::new(1),
                auth_token,
            };

            Ok(client)
        }
    }

    /// Initialize the protocol connection
    pub fn initialize(&mut self) -> AppResult<crate::protocol::InitializeResult> {
        let params = crate::protocol::InitializeParams {
            client_info: Some(crate::protocol::ClientInfo {
                name: "mnemosyne-client".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
            capabilities: crate::protocol::ClientCapabilities {
                semantic_analysis: true,
                git_integration: true,
                progress_notifications: false,
            },
            workspace_folders: None,
        };

        let result_value = self.call(
            crate::protocol::methods::INITIALIZE,
            serde_json::to_value(params).unwrap(),
        )?;
        serde_json::from_value(result_value)
            .map_err(|e| AppError::Internal(format!("Parse initialize response: {}", e)))
    }

    /// Send a JSON-RPC request and wait for the response.
    pub fn call(
        &mut self,
        method: &str,
        params: serde_json::Value,
    ) -> AppResult<serde_json::Value> {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let mut request = JsonRpcRequest::new(id, method, params);
        request.auth_token = self.auth_token.clone();

        let mut line = serde_json::to_string(&request)
            .map_err(|e| AppError::Internal(format!("Serialize request: {}", e)))?;
        line.push('\n');

        self.stream
            .write_all(line.as_bytes())
            .map_err(|e| AppError::Internal(format!("Send to daemon: {}", e)))?;
        self.stream
            .flush()
            .map_err(|e| AppError::Internal(format!("Flush to daemon: {}", e)))?;

        let mut reader = BufReader::new(&self.stream);
        let mut response_line = String::new();
        reader
            .read_line(&mut response_line)
            .map_err(|e| AppError::Internal(format!("Read from daemon: {}", e)))?;

        let response: JsonRpcResponse = serde_json::from_str(&response_line)
            .map_err(|e| AppError::Internal(format!("Parse response: {}", e)))?;

        if let Some(err) = response.error {
            return Err(AppError::Internal(format!(
                "Daemon error ({}): {}",
                err.code, err.message
            )));
        }

        response
            .result
            .ok_or_else(|| AppError::Internal("Empty response from daemon".into()))
    }

    /// Send a notification (no response expected).
    pub fn notify(&mut self, method: &str, params: serde_json::Value) -> AppResult<()> {
        let mut request = JsonRpcRequest::notification(method, params);
        request.auth_token = self.auth_token.clone();

        let mut line = serde_json::to_string(&request)
            .map_err(|e| AppError::Internal(format!("Serialize notification: {}", e)))?;
        line.push('\n');

        self.stream
            .write_all(line.as_bytes())
            .map_err(|e| AppError::Internal(format!("Send notification: {}", e)))?;
        self.stream
            .flush()
            .map_err(|e| AppError::Internal(format!("Flush notification: {}", e)))?;

        Ok(())
    }

    /// Check if the daemon is reachable.
    pub fn is_alive(&mut self) -> bool {
        self.call(crate::protocol::methods::STATUS, serde_json::Value::Null)
            .is_ok()
    }
}

/// Check if the mnem-daemon daemon is currently running.
pub fn daemon_running() -> bool {
    let base_dir = match get_base_dir() {
        Ok(dir) => dir,
        Err(_) => return false,
    };

    let pid_path = base_dir.join(PID_FILE);
    if !pid_path.exists() {
        return false;
    }

    if let Ok(pid_str) = std::fs::read_to_string(&pid_path) {
        if let Ok(pid) = pid_str.trim().parse::<u32>() {
            // Use the safe cross-platform is_process_running function
            return is_process_running(pid).unwrap_or(false);
        }
    }

    false
}

/// Ensure the mnem-daemon daemon is running. If not, attempt to start it.
/// Returns Ok(true) if daemon was started, Ok(false) if already running.
pub fn ensure_daemon() -> AppResult<bool> {
    if daemon_running() {
        return Ok(false);
    }

    let daemon_bin = find_daemon_binary()?;

    // Spawn daemon as a detached background process
    #[cfg(unix)]
    {
        let child = std::process::Command::new(&daemon_bin)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .map_err(|e| {
                AppError::Internal(format!(
                    "Failed to start mnem-daemon at {:?}: {}",
                    daemon_bin, e
                ))
            })?;

        // Don't wait on child - let it run detached
        std::mem::forget(child);
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const DETACHED_PROCESS: u32 = 0x00000008;
        const CREATE_NO_WINDOW: u32 = 0x08000000;

        let _child = std::process::Command::new(&daemon_bin)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .creation_flags(DETACHED_PROCESS | CREATE_NO_WINDOW)
            .spawn()
            .map_err(|e| {
                AppError::Internal(format!(
                    "Failed to start mnem-daemon at {:?}: {}",
                    daemon_bin, e
                ))
            })?;
    }

    // Wait for daemon to be ready (up to 10 seconds)
    // We check both PID and socket readiness to avoid race conditions
    let base_dir = get_base_dir()?;
    let socket_path = crate::protocol::get_socket_path(&base_dir);

    for _i in 0..100 {
        std::thread::sleep(std::time::Duration::from_millis(100));

        // First check if daemon process is running
        if daemon_running() {
            // Then try to actually connect to verify socket is ready
            #[cfg(unix)]
            {
                if std::os::unix::net::UnixStream::connect(&socket_path).is_ok() {
                    return Ok(true);
                }
            }
            #[cfg(windows)]
            {
                if std::fs::OpenOptions::new()
                    .read(true)
                    .write(true)
                    .open(&socket_path)
                    .is_ok()
                {
                    return Ok(true);
                }
            }
        }
    }

    Err(AppError::Internal(
        "mnem-daemon daemon started but socket did not become ready in time".into(),
    ))
}

/// Find the mnem-daemon binary. Searches:
/// 1. Same directory as the current executable
/// 2. PATH
fn find_daemon_binary() -> AppResult<PathBuf> {
    let mut bin_name = "mnem-daemon".to_string();
    if cfg!(windows) {
        bin_name.push_str(".exe");
    }

    // 1. Check next to the current binary
    if let Ok(current_exe) = std::env::current_exe() {
        if let Some(parent) = current_exe.parent() {
            let sibling = parent.join(&bin_name);
            if sibling.exists() {
                return Ok(sibling);
            }
        }
    }

    // 2. Check PATH
    #[cfg(unix)]
    {
        if let Ok(output) = std::process::Command::new("which")
            .arg("mnem-daemon")
            .output()
        {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !path.is_empty() {
                    return Ok(PathBuf::from(path));
                }
            }
        }
    }
    #[cfg(windows)]
    {
        if let Ok(output) = std::process::Command::new("where")
            .arg("mnem-daemon")
            .output()
        {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                // where can return multiple paths, take the first one
                if let Some(first_path) = path.lines().next() {
                    return Ok(PathBuf::from(first_path));
                }
            }
        }
    }

    Err(AppError::Internal(format!(
        "Cannot find {} binary. Build with `cargo build` or add to PATH.",
        bin_name
    )))
}
