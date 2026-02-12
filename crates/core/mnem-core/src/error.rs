use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("IO error at {path}: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("Generic IO error: {0}")]
    IoGeneric(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Security violation: {0}")]
    Security(String),

    #[error("Path Traversal detected: {0}")]
    PathTraversal(PathBuf),

    #[error("Semantic analysis failed: {0}")]
    Semantic(String),

    #[error("SDP error: {0}")]
    Sdp(#[from] semantic_delta_protocol::SdpError),

    #[error("Protocol error: {0}")]
    Protocol(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Other error: {0}")]
    Other(#[from] anyhow::Error),
}

pub type AppResult<T> = Result<T, AppError>;
