pub mod history;
pub mod info;
pub mod restore;
pub mod search;

// Migrated commands - using Strategy pattern
pub use history::HistoryCommand;
pub use info::InfoCommand;
pub use restore::RestoreCommand;
pub use search::SearchCommand;
