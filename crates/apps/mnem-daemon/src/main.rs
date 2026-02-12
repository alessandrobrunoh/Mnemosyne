use anyhow::Result;
use log::{error, info, warn};
use mnem_core::env::get_base_dir;
use mnem_core::protocol::{self, JsonRpcRequest, JsonRpcResponse, PID_FILE};

use mnem_core::storage::registry::ProjectRegistry;
use mnem_core::Repository;
use mnem_daemon::{DaemonState, Monitor};
use mnem_daemon::rpc_handler::handle_request;
use mnem_daemon::maintenance::run_background_maintenance;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

async fn restore_watched_projects(base_dir: &PathBuf, state: &Arc<DaemonState>) {

    let registry = match ProjectRegistry::new(base_dir) {
        Ok(r) => r,
        Err(e) => {
            warn!("Failed to load project registry: {}", e);
            return;
        }
    };

    let projects = registry.list_projects();
    if projects.is_empty() {
        info!("No projects in registry to auto-watch.");
        return;
    }

    info!("Restoring {} project(s) from registry...", projects.len());

    for project in projects {
        let project_path = PathBuf::from(&project.path);
        if !project_path.exists() {
            warn!("Skipping missing project path: {}", project.path);
            continue;
        }

        let path_key = project.path.clone();

        match Repository::open(base_dir.clone(), project_path.clone()) {
            Ok(repo) => {
                let repo = Arc::new(repo);
                let monitor = Arc::new(Monitor::new(project_path, repo.clone()));

                let scan_path = path_key.clone();
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

                state.repos.insert(path_key.clone(), repo);
                state.monitors.insert(path_key.clone(), monitor);


                info!("Auto-watching restored project: {}", path_key);
            }
            Err(e) => {
                warn!("Failed to open repo for {}: {}", path_key, e);
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let base_dir = get_base_dir()?;
    std::fs::create_dir_all(&base_dir)?;


    let log_dir = base_dir.join("logs");
    std::fs::create_dir_all(&log_dir)?;

    flexi_logger::Logger::try_with_env_or_str("info")?
        .log_to_file(
            flexi_logger::FileSpec::default()
                .directory(&log_dir)
                .basename("mnem-daemon")
                .suffix("log"),
        )
        .duplicate_to_stdout(flexi_logger::Duplicate::All)
        .format(flexi_logger::opt_format)
        .start()?;

    info!("mnem-daemon v{} starting up...", env!("CARGO_PKG_VERSION"));

    let socket_path = protocol::get_socket_path(&base_dir);
    let pid_path = base_dir.join(PID_FILE);

    if pid_path.exists() {
        if let Ok(pid_str) = std::fs::read_to_string(&pid_path) {
            if let Ok(pid) = pid_str.trim().parse::<u32>() {
                if mnem_daemon::os::check_running_pid(pid) {
                    eprintln!("mnem-daemon is already running (PID {})", pid);
                    std::process::exit(1);
                }
            }
        }
    }

    std::fs::write(&pid_path, std::process::id().to_string())?;

    // Generate Auth Token
    let auth_token = mnem_core::utils::auth::AuthManager::generate_token()?;
    info!("Secure auth token generated.");

    let state = Arc::new(DaemonState::new(auth_token));


    let restore_state = state.clone();
    let restore_base_dir = base_dir.clone();
    tokio::spawn(async move {
        restore_watched_projects(&restore_base_dir, &restore_state).await;
    });

    let maintenance_state = state.clone();
    tokio::spawn(async move {
        run_background_maintenance(maintenance_state).await;
    });

    let pid_path_clone = pid_path.clone();
    #[cfg(unix)]
    let socket_path_clone = socket_path.clone();

    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        info!("Shutting down mnem-daemon...");
        let _ = std::fs::remove_file(&pid_path_clone);
        #[cfg(unix)]
        let _ = std::fs::remove_file(&socket_path_clone);
        std::process::exit(0);
    });

    #[cfg(unix)]
    {
        use tokio::net::UnixListener;
        if socket_path.exists() {
            std::fs::remove_file(&socket_path)?;
        }
        let listener = UnixListener::bind(&socket_path)?;
        info!("mnem-daemon listening on unix socket: {:?}", socket_path);

        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    let state = state.clone();
                    tokio::spawn(async move {
                        if let Err(e) = handle_connection(stream, state).await {
                            error!("Connection error: {}", e);
                        }
                    });
                }
                Err(e) => {
                    error!("Accept error: {}", e);
                }
            }
        }
    }

    #[cfg(windows)]
    {
        use tokio::net::windows::named_pipe::ServerOptions;
        let pipe_name = socket_path.to_string_lossy().to_string();
        info!("mnem-daemon listening on named pipe: {}", pipe_name);

        let mut first = true;
        loop {
            let server = ServerOptions::new()
                .first_pipe_instance(first)
                .create(&pipe_name)?;

            first = false;

            server.connect().await?;
            let state = state.clone();
            tokio::spawn(async move {
                if let Err(e) = handle_connection(server, state).await {
                    error!("Connection error: {}", e);
                }
            });
        }
    }
}

async fn handle_connection<S>(stream: S, state: Arc<DaemonState>) -> Result<()>
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    let (reader, mut writer) = tokio::io::split(stream);
    let mut lines = BufReader::new(reader).lines();

    while let Some(line) = lines.next_line().await? {
        let request: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(req) => req,
            Err(e) => {
                let err_resp = JsonRpcResponse::error(None, -32700, format!("Parse error: {}", e));
                let resp_json = serde_json::to_string(&err_resp)? + "\n";
                writer.write_all(resp_json.as_bytes()).await?;
                continue;
            }
        };

        let start = std::time::Instant::now();
        
        // Token Validation
        let is_authorized = request.auth_token.as_ref() == Some(&state.auth_token);

        if !is_authorized && request.method != protocol::methods::STATUS {
            let err_resp = JsonRpcResponse::error(request.id, -32001, "Unauthorized: Invalid or missing auth token".into());
            let resp_json = serde_json::to_string(&err_resp)? + "\n";
            writer.write_all(resp_json.as_bytes()).await?;
            continue;
        }

        let response = handle_request(&request, &state).await;
        let duration = start.elapsed();

        state.record_request(duration.as_micros() as u64);

        let resp_json = serde_json::to_string(&response)? + "\n";
        writer.write_all(resp_json.as_bytes()).await?;
    }

    Ok(())
}

