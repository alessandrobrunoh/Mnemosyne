//! Git integration module for Mnemosyne
//!
//! Provides functionality for reading Git configuration and metadata
//! to attribute code changes to specific users.

pub mod user;

pub use user::GitUser;
pub use user::GitUserResolver;
