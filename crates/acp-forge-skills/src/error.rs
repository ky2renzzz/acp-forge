//! Error types for the skills system.

use std::path::PathBuf;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("skill {name} not found")]
    NotFound { name: String },

    #[error("skill {name} version conflict: have {have}, need {need}")]
    VersionConflict {
        name: String,
        have: String,
        need: String,
    },

    #[error("template render: missing parameter {param}")]
    MissingParam { param: String },

    #[error("invalid skill file at {path}: {reason}")]
    InvalidFile { path: PathBuf, reason: String },

    #[error("cycle in skill composition: {chain}")]
    CompositionCycle { chain: String },

    #[error("toml: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}
