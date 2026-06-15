//! # acp-forge-web
//!
//! Real-time web source integration for agent orchestration.
//! Provides a trait-based provider system, response caching with TTL,
//! and cross-reference verification to reduce hallucination.

pub mod cache;
pub mod error;
pub mod provider;
pub mod verify;

pub use cache::Cache;
pub use error::{Error, Result};
pub use provider::{Provider, SearchResult};
