use std::path::PathBuf;

pub const SOCKET_DIR: &str = "mnemosyne";
// Named pipe path on Windows: \\.\pipe\mnemd
pub const SOCKET_NAME: &str = r"\\.\pipe\mnemd";

pub fn get_socket_path(_base_dir: &std::path::Path) -> PathBuf {
    PathBuf::from(SOCKET_NAME)
}
