//! Error types for the core crate.

use std::path::PathBuf;

/// Alias used throughout the crate.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// All errors that can arise from core orchestration.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("agent {id} not found")]
    AgentNotFound { id: String },

    #[error("task {id} not found")]
    TaskNotFound { id: String },

    #[error("agent {id} is in state {state} — expected {expected}")]
    InvalidState {
        id: String,
        state: String,
        expected: String,
    },

    #[error("capacity exceeded: {message}")]
    CapacityExceeded { message: String },

    #[error("worktree error at {path}: {source}")]
    Worktree {
        path: PathBuf,
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("serialization: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("io: {0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Other(String),
}
