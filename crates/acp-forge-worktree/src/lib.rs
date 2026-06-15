//! # acp-forge-worktree
//!
//! Manages git worktrees so each agent operates on an isolated branch
//! without stepping on another agent's changes. Provides divergence
//! detection to surface merge conflicts early.

pub mod divergence;
pub mod error;
pub mod git;
pub mod pool;

pub use error::{Error, Result};
pub use git::WorktreeHandle;
pub use pool::WorktreePool;
