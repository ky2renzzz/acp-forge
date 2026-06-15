//! Agent identity, roles, and lifecycle.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::id::Id;

/// Marker type for agent IDs.
#[derive(Debug)]
pub enum AgentMarker {}

/// Specialised role an agent can play within a swarm.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentRole {
    /// Reads and navigates code. Never writes.
    Explorer,
    /// Writes or modifies source files.
    Writer,
    /// Runs and analyses tests.
    TestRunner,
    /// Reviews code for correctness, security, style.
    Reviewer,
    /// Refactors without changing behaviour.
    Refactorer,
    /// Executes shell commands and manages environments.
    ShellOperator,
    /// Produces documentation.
    DocWriter,
    /// Creates and validates plans. Read-only.
    Planner,
}

/// Lifecycle state of an agent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentStatus {
    /// Created but not yet started.
    Idle,
    /// Performing work.
    Running,
    /// Waiting on user or another agent.
    Blocked,
    /// Finished successfully.
    Completed,
    /// Terminated due to error.
    Failed,
}

/// A participant in a multi-agent session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: Id<AgentMarker>,
    pub name: String,
    pub role: AgentRole,
    pub status: AgentStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// Optional branch / worktree this agent operates in.
    pub worktree: Option<String>,
}

impl Agent {
    /// Create a new idle agent.
    #[must_use]
    pub fn new(name: impl Into<String>, role: AgentRole) -> Self {
        let now = Utc::now();
        Self {
            id: Id::new(),
            name: name.into(),
            role,
            status: AgentStatus::Idle,
            created_at: now,
            updated_at: now,
            worktree: None,
        }
    }

    /// Transition to a new status. Returns `Err` on invalid transition.
    pub fn transition(&mut self, to: AgentStatus) -> crate::Result<()> {
        use AgentStatus::*;

        let valid = matches!(
            (self.status, to),
            (Idle, Running)
                | (Running, Blocked)
                | (Running, Completed)
                | (Running, Failed)
                | (Blocked, Running)
                | (Blocked, Failed)
        );

        if !valid {
            return Err(crate::Error::InvalidState {
                id: self.id.to_string(),
                state: format!("{:?}", self.status),
                expected: format!("{to:?}"),
            });
        }

        self.status = to;
        self.updated_at = Utc::now();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_transitions() {
        let mut a = Agent::new("test", AgentRole::Explorer);
        assert!(a.transition(AgentStatus::Running).is_ok());
        assert!(a.transition(AgentStatus::Blocked).is_ok());
        assert!(a.transition(AgentStatus::Running).is_ok());
        assert!(a.transition(AgentStatus::Completed).is_ok());
    }

    #[test]
    fn invalid_transition_rejected() {
        let mut a = Agent::new("test", AgentRole::Writer);
        assert!(a.transition(AgentStatus::Completed).is_err());
    }
}
