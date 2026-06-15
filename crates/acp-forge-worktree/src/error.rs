//! Error types for worktree operations.

use std::path::PathBuf;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("git command failed: {0}")]
    Git(String),

    #[error("worktree {path} already exists")]
    AlreadyExists { path: PathBuf },

    #[error("worktree {path} not found")]
    NotFound { path: PathBuf },

    #[error("divergence detected between {a} and {b}: {detail}")]
    Divergence {
        a: String,
        b: String,
        detail: String,
    },

    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}
