pub mod config;
pub mod gc;
pub mod uninstall;
pub mod update;

pub use config::handle_config;
pub use gc::handle_gc;
pub use uninstall::handle_uninstall;
pub use update::handle_update;
