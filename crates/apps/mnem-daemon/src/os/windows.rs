use anyhow::Result;
use log::info;
use std::path::Path;
use tokio::net::windows::named_pipe::{ServerOptions, NamedPipeServer};

pub struct WindowsListener {
    pipe_name: String,
}

impl WindowsListener {
    pub fn new(pipe_name: &str) -> Self {
        Self {
            pipe_name: pipe_name.to_string(),
        }
    }

    pub async fn accept(&self) -> Result<NamedPipeServer> {
        let server = ServerOptions::new()
            .first_pipe_instance(true)
            .create(&self.pipe_name)?;
        
        server.connect().await?;
        Ok(server)
    }
}

pub async fn bind_socket(socket_path: &Path) -> Result<WindowsListener> {
    let pipe_name = socket_path.to_string_lossy().to_string();
    info!("mnemd listening on named pipe: {}", pipe_name);
    Ok(WindowsListener::new(&pipe_name))
}

pub fn check_running_pid(pid: u32) -> bool {
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if handle != 0 {
            CloseHandle(handle);
            true
        } else {
            false
        }
    }
}
