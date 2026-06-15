//! Tasks — the unit of work within a session.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::agent::AgentMarker;
use crate::id::Id;

/// Marker type for task IDs.
#[derive(Debug)]
pub enum TaskMarker {}

/// Progress state of a task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Pending,
    InProgress,
    Done,
    Failed,
    Cancelled,
}

/// A discrete unit of work.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: Id<TaskMarker>,
    pub description: String,
    pub status: TaskStatus,
    pub assigned_to: Option<Id<AgentMarker>>,
    pub depends_on: Vec<Id<TaskMarker>>,
    pub created_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
}

impl Task {
    /// Create a new pending task.
    #[must_use]
    pub fn new(description: impl Into<String>) -> Self {
        Self {
            id: Id::new(),
            description: description.into(),
            status: TaskStatus::Pending,
            assigned_to: None,
            depends_on: Vec::new(),
            created_at: Utc::now(),
            finished_at: None,
        }
    }

    /// Assign this task to an agent.
    pub fn assign(&mut self, agent: Id<AgentMarker>) {
        self.assigned_to = Some(agent);
    }

    /// Add a dependency — this task waits for `other` to finish.
    pub fn depends_on(&mut self, other: Id<TaskMarker>) {
        if !self.depends_on.contains(&other) {
            self.depends_on.push(other);
        }
    }

    /// Mark in progress.
    pub fn start(&mut self) -> crate::Result<()> {
        if self.status != TaskStatus::Pending {
            return Err(crate::Error::InvalidState {
                id: self.id.to_string(),
                state: format!("{:?}", self.status),
                expected: "Pending".into(),
            });
        }
        self.status = TaskStatus::InProgress;
        Ok(())
    }

    /// Mark completed.
    pub fn complete(&mut self) -> crate::Result<()> {
        if self.status != TaskStatus::InProgress {
            return Err(crate::Error::InvalidState {
                id: self.id.to_string(),
                state: format!("{:?}", self.status),
                expected: "InProgress".into(),
            });
        }
        self.status = TaskStatus::Done;
        self.finished_at = Some(Utc::now());
        Ok(())
    }

    /// Mark failed.
    pub fn fail(&mut self) {
        self.status = TaskStatus::Failed;
        self.finished_at = Some(Utc::now());
    }

    /// Check whether all dependencies are satisfied.
    #[must_use]
    pub fn is_ready(&self, completed: &[Id<TaskMarker>]) -> bool {
        self.status == TaskStatus::Pending
            && self.depends_on.iter().all(|dep| completed.contains(dep))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_lifecycle() {
        let mut t = Task::new("implement feature");
        assert_eq!(t.status, TaskStatus::Pending);
        t.start().unwrap();
        assert_eq!(t.status, TaskStatus::InProgress);
        t.complete().unwrap();
        assert_eq!(t.status, TaskStatus::Done);
        assert!(t.finished_at.is_some());
    }

    #[test]
    fn dependency_readiness() {
        let dep = Task::new("prerequisite");
        let mut task = Task::new("depends on prereq");
        task.depends_on(dep.id);

        assert!(!task.is_ready(&[]));
        assert!(task.is_ready(&[dep.id]));
    }
}
