//! Replay engine — step through a recorded session and verify integrity.

use crate::error::{Error, Result};
use crate::event::Event;
use crate::session::Session;

/// Result of replaying one event.
#[derive(Debug)]
pub enum StepResult {
    /// Event verified successfully.
    Ok { seq: u64 },
    /// Hash mismatch detected.
    HashMismatch {
        seq: u64,
        expected: String,
        actual: String,
    },
}

/// Walks through a session's events in sequence order.
pub struct Replayer<'a> {
    events: &'a [Event],
    cursor: usize,
}

impl<'a> Replayer<'a> {
    /// Create a replayer for the given session.
    #[must_use]
    pub fn new(session: &'a Session) -> Self {
        Self {
            events: &session.events,
            cursor: 0,
        }
    }

    /// Step to the next event. Returns `None` when exhausted.
    pub fn step(&mut self) -> Option<Result<StepResult>> {
        if self.cursor >= self.events.len() {
            return None;
        }

        let event = &self.events[self.cursor];
        self.cursor += 1;

        let computed = match event.compute_hash() {
            Ok(h) => h,
            Err(e) => return Some(Err(e)),
        };

        if computed != event.content_hash {
            Some(Ok(StepResult::HashMismatch {
                seq: event.seq,
                expected: event.content_hash.clone(),
                actual: computed,
            }))
        } else {
            Some(Ok(StepResult::Ok { seq: event.seq }))
        }
    }

    /// How many events remain.
    #[must_use]
    pub fn remaining(&self) -> usize {
        self.events.len() - self.cursor
    }

    /// Run all remaining steps, returning the first error or mismatch.
    pub fn verify_all(&mut self) -> Result<()> {
        while let Some(result) = self.step() {
            match result? {
                StepResult::Ok { .. } => {}
                StepResult::HashMismatch {
                    seq,
                    expected,
                    actual,
                } => {
                    return Err(Error::HashMismatch {
                        id: format!("seq:{seq}"),
                        expected,
                        actual,
                    });
                }
            }
        }
        Ok(())
    }
}

/// Diff two sessions: find the first point of divergence.
pub fn diff_sessions(a: &Session, b: &Session) -> Option<usize> {
    let len = a.events.len().min(b.events.len());
    for i in 0..len {
        if a.events[i].content_hash != b.events[i].content_hash {
            return Some(i);
        }
    }
    if a.events.len() != b.events.len() {
        Some(len)
    } else {
        None
    }
}
