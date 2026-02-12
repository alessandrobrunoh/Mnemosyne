use std::path::PathBuf;

pub const SOCKET_DIR: &str = ".mnemosyne";
pub const SOCKET_NAME: &str = "mnemd.sock";

pub fn get_socket_path(base_dir: &std::path::Path) -> PathBuf {
    base_dir.join(SOCKET_NAME)
}
