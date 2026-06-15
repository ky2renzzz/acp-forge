//! # acp-forge-graph
//!
//! Records multi-agent sessions as a directed acyclic graph of events.
//! Each node is an action (tool call, file edit, search, LLM response)
//! with a content hash for integrity verification. Sessions can be
//! serialized to canonical JSON and replayed deterministically.

pub mod error;
pub mod event;
pub mod hasher;
pub mod recorder;
pub mod replay;
pub mod session;

pub use error::{Error, Result};
pub use event::{Event, EventKind, EventPayload};
pub use recorder::Recorder;
pub use session::Session;
