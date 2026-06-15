//! Live event recorder — appends events to a session as they happen.

use std::sync::atomic::{AtomicU64, Ordering};

use acp_forge_core::agent::AgentMarker;
use acp_forge_core::id::Id;
use chrono::Utc;

use crate::event::{Event, EventKind, EventMarker, EventPayload};
use crate::hasher;
use crate::session::Session;

/// Records events into a session in append-only fashion.
#[derive(Debug)]
pub struct Recorder {
    session: Session,
    seq: AtomicU64,
}

impl Recorder {
    /// Start recording into the given session.
    #[must_use]
    pub fn new(session: Session) -> Self {
        let seq = session.events.len() as u64;
        Self {
            session,
            seq: AtomicU64::new(seq),
        }
    }

    /// Record a new event and return its ID.
    pub fn record(
        &mut self,
        agent_id: Id<AgentMarker>,
        kind: EventKind,
        payload: EventPayload,
        parents: Vec<Id<EventMarker>>,
    ) -> crate::Result<Id<EventMarker>> {
        let seq = self.seq.fetch_add(1, Ordering::Relaxed);
        let id = Id::<EventMarker>::new();

        let hashable = serde_json::json!({
            "kind": kind,
            "payload": payload,
            "parents": parents,
        });
        let content_hash = hasher::hash_json(&hashable)?;

        let event = Event {
            id,
            agent_id,
            seq,
            kind,
            payload,
            parents,
            content_hash,
            timestamp: Utc::now(),
        };

        self.session.push(event);
        Ok(id)
    }

    /// Convenience: record a tool call.
    pub fn tool_call(
        &mut self,
        agent_id: Id<AgentMarker>,
        tool_name: &str,
        args: serde_json::Value,
        parents: Vec<Id<EventMarker>>,
    ) -> crate::Result<Id<EventMarker>> {
        self.record(
            agent_id,
            EventKind::ToolCall,
            EventPayload {
                summary: format!("call: {tool_name}"),
                data: args,
            },
            parents,
        )
    }

    /// Convenience: record a file edit.
    pub fn file_edit(
        &mut self,
        agent_id: Id<AgentMarker>,
        path: &str,
        description: &str,
        parents: Vec<Id<EventMarker>>,
    ) -> crate::Result<Id<EventMarker>> {
        self.record(
            agent_id,
            EventKind::FileEdit,
            EventPayload {
                summary: description.to_owned(),
                data: serde_json::json!({ "path": path }),
            },
            parents,
        )
    }

    /// Seal and return the finished session.
    pub fn finish(mut self) -> crate::Result<Session> {
        self.session.seal()?;
        Ok(self.session)
    }

    /// Borrow the session-in-progress (read-only).
    #[must_use]
    pub fn session(&self) -> &Session {
        &self.session
    }
}
