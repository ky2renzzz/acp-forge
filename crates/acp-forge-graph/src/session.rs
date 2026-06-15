//! A session is the complete execution graph of one orchestration run.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use acp_forge_core::id::Id;

use crate::event::Event;
use crate::hasher;

/// Marker for session IDs.
#[derive(Debug)]
pub enum SessionMarker {}

/// Metadata about the session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMeta {
    pub description: String,
    pub base_ref: String,
    pub agent_count: usize,
}

/// A complete recorded session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: Id<SessionMarker>,
    pub meta: SessionMeta,
    pub events: Vec<Event>,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    /// Hash of the entire event sequence for tamper detection.
    pub integrity_hash: Option<String>,
}

impl Session {
    /// Create a new empty session.
    #[must_use]
    pub fn new(meta: SessionMeta) -> Self {
        Self {
            id: Id::new(),
            meta,
            events: Vec::new(),
            started_at: Utc::now(),
            finished_at: None,
            integrity_hash: None,
        }
    }

    /// Append an event.
    pub fn push(&mut self, event: Event) {
        self.events.push(event);
    }

    /// Seal the session: record finish time and compute integrity hash.
    pub fn seal(&mut self) -> crate::Result<()> {
        self.finished_at = Some(Utc::now());
        self.integrity_hash = Some(hasher::hash_json(&self.events)?);
        Ok(())
    }

    /// Verify integrity of a sealed session.
    pub fn verify_integrity(&self) -> crate::Result<bool> {
        let Some(ref expected) = self.integrity_hash else {
            return Ok(false);
        };
        let actual = hasher::hash_json(&self.events)?;
        Ok(&actual == expected)
    }

    /// Serialize to canonical JSON bytes.
    pub fn to_canonical_json(&self) -> crate::Result<Vec<u8>> {
        let json = hasher::canonical_json(self)?;
        Ok(json.into_bytes())
    }

    /// Deserialize from JSON.
    pub fn from_json(data: &[u8]) -> crate::Result<Self> {
        Ok(serde_json::from_slice(data)?)
    }

    /// Save to a file.
    pub fn save(&self, path: &std::path::Path) -> crate::Result<()> {
        let bytes = self.to_canonical_json()?;
        std::fs::write(path, bytes)?;
        Ok(())
    }

    /// Load from a file.
    pub fn load(path: &std::path::Path) -> crate::Result<Self> {
        let bytes = std::fs::read(path)?;
        Self::from_json(&bytes)
    }
}
