use anyhow::Result;
use log::info;
use tokio::net::UnixListener;
use std::path::Path;

pub async fn bind_socket(socket_path: &Path) -> Result<UnixListener> {
    if socket_path.exists() {
        std::fs::remove_file(socket_path)?;
    }
    let listener = UnixListener::bind(socket_path)?;
    info!("mnemd listening on {:?}", socket_path);
    Ok(listener)
}

pub fn check_running_pid(pid: u32) -> bool {
    unsafe { libc::kill(pid as i32, 0) == 0 }
}
