//! Error types for the execution graph.

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("event {id} not found")]
    EventNotFound { id: String },

    #[error("cycle detected: {0}")]
    CycleDetected(String),

    #[error("hash mismatch for event {id}: expected {expected}, got {actual}")]
    HashMismatch {
        id: String,
        expected: String,
        actual: String,
    },

    #[error("replay diverged at event {index}: {detail}")]
    ReplayDiverged { index: usize, detail: String },

    #[error("serialization: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}
