//! Events — the nodes of the execution DAG.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use acp_forge_core::agent::AgentMarker;
use acp_forge_core::id::Id;

/// Marker for event IDs.
#[derive(Debug)]
pub enum EventMarker {}

/// What kind of action this event represents.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    ToolCall,
    ToolResult,
    FileEdit,
    FileRead,
    Search,
    LlmRequest,
    LlmResponse,
    UserInput,
    Checkpoint,
}

/// The data attached to an event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventPayload {
    /// Human-readable summary.
    pub summary: String,
    /// Structured data (tool arguments, file paths, etc.).
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub data: serde_json::Value,
}

/// A single node in the execution graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// Unique identifier.
    pub id: Id<EventMarker>,
    /// Which agent produced this event.
    pub agent_id: Id<AgentMarker>,
    /// Monotonic sequence number within the session.
    pub seq: u64,
    /// Classification.
    pub kind: EventKind,
    /// Payload.
    pub payload: EventPayload,
    /// IDs of events this one causally depends on.
    pub parents: Vec<Id<EventMarker>>,
    /// SHA-256 of the canonical JSON of (kind, payload, parents).
    pub content_hash: String,
    /// When this event was recorded.
    pub timestamp: DateTime<Utc>,
}

impl Event {
    /// Compute the content hash for verification.
    pub fn compute_hash(&self) -> crate::Result<String> {
        use crate::hasher::hash_json;
        let hashable = serde_json::json!({
            "kind": self.kind,
            "payload": self.payload,
            "parents": self.parents,
        });
        hash_json(&hashable)
    }

    /// Verify the stored hash matches the computed one.
    pub fn verify(&self) -> crate::Result<bool> {
        let computed = self.compute_hash()?;
        Ok(computed == self.content_hash)
    }
}
