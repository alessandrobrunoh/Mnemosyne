use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum DaemonRequest {
    Ping,
    Status,
    WatchProject { path: PathBuf },
    UnwatchProject { path: PathBuf },
    Stop,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum DaemonResponse {
    Pong,
    Status { projects: Vec<ProjectInfo> },
    Success,
    Error { message: String },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectInfo {
    pub path: PathBuf,
    pub name: String,
    pub file_count: usize,
    pub last_activity: String,
}

#[cfg(unix)]
pub use unix_impl::*;

#[cfg(windows)]
pub use windows_impl::*;

#[cfg(unix)]
mod unix_impl {
    use super::*;
    use std::os::unix::net::{UnixListener, UnixStream};

    pub struct IpcServer {
        listener: UnixListener,
    }

    impl IpcServer {
        pub fn new(socket_path: PathBuf) -> AppResult<Self> {
            let _ = std::fs::remove_file(&socket_path);
            let listener = UnixListener::bind(&socket_path).map_err(|e| AppError::IoGeneric(e))?;
            Ok(Self { listener })
        }

        pub fn accept(&self) -> AppResult<(UnixStream, DaemonRequest)> {
            let (mut stream, _) = self.listener.accept().map_err(|e| AppError::IoGeneric(e))?;

            let mut buffer = Vec::new();
            stream
                .read_to_end(&mut buffer)
                .map_err(|e| AppError::IoGeneric(e))?;

            let request: DaemonRequest = serde_json::from_slice(&buffer)
                .map_err(|e| AppError::Internal(format!("Failed to parse request: {}", e)))?;

            Ok((stream, request))
        }
    }

    pub fn respond_unix(stream: &mut UnixStream, response: DaemonResponse) -> AppResult<()> {
        let data = serde_json::to_vec(&response)
            .map_err(|e| AppError::Internal(format!("Failed to serialize response: {}", e)))?;
        stream
            .write_all(&data)
            .map_err(|e| AppError::IoGeneric(e))?;
        Ok(())
    }

    pub struct IpcClient;

    impl IpcClient {
        pub fn send(socket_path: &PathBuf, request: DaemonRequest) -> AppResult<DaemonResponse> {
            let mut stream = UnixStream::connect(socket_path)
                .map_err(|e| AppError::Internal(format!("Daemon not running: {}", e)))?;

            let data = serde_json::to_vec(&request)
                .map_err(|e| AppError::Internal(format!("Failed to serialize request: {}", e)))?;

            stream
                .write_all(&data)
                .map_err(|e| AppError::IoGeneric(e))?;
            stream
                .shutdown(std::net::Shutdown::Write)
                .map_err(|e| AppError::IoGeneric(e))?;

            let mut buffer = Vec::new();
            stream
                .read_to_end(&mut buffer)
                .map_err(|e| AppError::IoGeneric(e))?;

            let response: DaemonResponse = serde_json::from_slice(&buffer)
                .map_err(|e| AppError::Internal(format!("Failed to parse response: {}", e)))?;

            Ok(response)
        }

        pub fn is_running(socket_path: &PathBuf) -> bool {
            matches!(UnixStream::connect(socket_path), Ok(_))
        }
    }
}

#[cfg(windows)]
mod windows_impl {
    use super::*;
    use std::net::{TcpListener, TcpStream};

    fn get_port_file() -> PathBuf {
        dirs::home_dir()
            .expect("Home dir not found")
            .join(".mnemosyne")
            .join("daemon.port")
    }

    pub struct IpcServer {
        listener: TcpListener,
        port: u16,
    }

    impl IpcServer {
        pub fn new(_socket_path: PathBuf) -> AppResult<Self> {
            let listener = TcpListener::bind("127.0.0.1:0").map_err(|e| AppError::IoGeneric(e))?;
            let port = listener
                .local_addr()
                .map_err(|e| AppError::IoGeneric(e))?
                .port();

            std::fs::write(get_port_file(), port.to_string())
                .map_err(|e| AppError::IoGeneric(e))?;

            Ok(Self { listener, port })
        }

        pub fn accept(&self) -> AppResult<(TcpStream, DaemonRequest)> {
            let (stream, _) = self.listener.accept().map_err(|e| AppError::IoGeneric(e))?;

            let mut buffer = Vec::new();
            let mut stream_read = stream.try_clone().map_err(|e| AppError::IoGeneric(e))?;
            stream_read
                .read_to_end(&mut buffer)
                .map_err(|e| AppError::IoGeneric(e))?;

            let request: DaemonRequest = serde_json::from_slice(&buffer)
                .map_err(|e| AppError::Internal(format!("Failed to parse request: {}", e)))?;

            Ok((stream, request))
        }

        pub fn port(&self) -> u16 {
            self.port
        }
    }

    pub fn respond_tcp(stream: &mut TcpStream, response: DaemonResponse) -> AppResult<()> {
        let data = serde_json::to_vec(&response)
            .map_err(|e| AppError::Internal(format!("Failed to serialize response: {}", e)))?;
        stream
            .write_all(&data)
            .map_err(|e| AppError::IoGeneric(e))?;
        Ok(())
    }

    pub struct IpcClient;

    impl IpcClient {
        pub fn send(_socket_path: &PathBuf, request: DaemonRequest) -> AppResult<DaemonResponse> {
            let port_file = get_port_file();

            let port = std::fs::read_to_string(&port_file)
                .map_err(|e| AppError::Internal(format!("Daemon not running: {}", e)))?
                .trim()
                .parse::<u16>()
                .map_err(|e| AppError::Internal(format!("Invalid port: {}", e)))?;

            let mut stream = TcpStream::connect(format!("127.0.0.1:{}", port))
                .map_err(|e| AppError::Internal(format!("Daemon not running: {}", e)))?;

            let data = serde_json::to_vec(&request)
                .map_err(|e| AppError::Internal(format!("Failed to serialize request: {}", e)))?;

            stream
                .write_all(&data)
                .map_err(|e| AppError::IoGeneric(e))?;
            stream
                .shutdown(std::net::Shutdown::Write)
                .map_err(|e| AppError::IoGeneric(e))?;

            let mut buffer = Vec::new();
            stream
                .read_to_end(&mut buffer)
                .map_err(|e| AppError::IoGeneric(e))?;

            let response: DaemonResponse = serde_json::from_slice(&buffer)
                .map_err(|e| AppError::Internal(format!("Failed to parse response: {}", e)))?;

            Ok(response)
        }

        pub fn is_running(_socket_path: &PathBuf) -> bool {
            let port_file = get_port_file();

            if let Ok(port_str) = std::fs::read_to_string(&port_file) {
                if let Ok(port) = port_str.trim().parse::<u16>() {
                    return TcpStream::connect(format!("127.0.0.1:{}", port)).is_ok();
                }
            }
            false
        }
    }
}

pub fn get_socket_path() -> PathBuf {
    let home = dirs::home_dir().expect("Home directory not found");
    home.join(".mnemosyne").join("daemon.sock")
}
