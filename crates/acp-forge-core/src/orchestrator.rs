//! The orchestrator coordinates agents and tasks.

use std::collections::HashMap;

use crate::agent::{Agent, AgentMarker, AgentRole, AgentStatus};
use crate::error::{Error, Result};
use crate::id::Id;
use crate::task::{Task, TaskMarker, TaskStatus};

/// Upper bound on concurrent agents.
const MAX_AGENTS: usize = 8;

/// Central coordinator for a multi-agent session.
#[derive(Debug)]
pub struct Orchestrator {
    agents: HashMap<Id<AgentMarker>, Agent>,
    tasks: HashMap<Id<TaskMarker>, Task>,
}

impl Orchestrator {
    /// Create an empty orchestrator.
    #[must_use]
    pub fn new() -> Self {
        Self {
            agents: HashMap::new(),
            tasks: HashMap::new(),
        }
    }

    // ── Agents ──────────────────────────────────────────────────────

    /// Spawn a new agent with the given role.
    pub fn add_agent(&mut self, name: impl Into<String>, role: AgentRole) -> Result<Id<AgentMarker>> {
        if self.agents.len() >= MAX_AGENTS {
            return Err(Error::CapacityExceeded {
                message: format!("maximum {MAX_AGENTS} agents"),
            });
        }
        let agent = Agent::new(name, role);
        let id = agent.id;
        self.agents.insert(id, agent);
        Ok(id)
    }

    /// Get a reference to an agent.
    pub fn agent(&self, id: Id<AgentMarker>) -> Result<&Agent> {
        self.agents
            .get(&id)
            .ok_or_else(|| Error::AgentNotFound { id: id.to_string() })
    }

    /// Get a mutable reference to an agent.
    pub fn agent_mut(&mut self, id: Id<AgentMarker>) -> Result<&mut Agent> {
        self.agents
            .get_mut(&id)
            .ok_or_else(|| Error::AgentNotFound { id: id.to_string() })
    }

    /// All agents, ordered by creation time.
    #[must_use]
    pub fn agents(&self) -> Vec<&Agent> {
        let mut v: Vec<_> = self.agents.values().collect();
        v.sort_by_key(|a| a.id);
        v
    }

    /// Currently running agents.
    #[must_use]
    pub fn active_agents(&self) -> Vec<&Agent> {
        self.agents
            .values()
            .filter(|a| a.status == AgentStatus::Running)
            .collect()
    }

    // ── Tasks ───────────────────────────────────────────────────────

    /// Register a task.
    pub fn add_task(&mut self, task: Task) -> Id<TaskMarker> {
        let id = task.id;
        self.tasks.insert(id, task);
        id
    }

    /// Get a reference to a task.
    pub fn task(&self, id: Id<TaskMarker>) -> Result<&Task> {
        self.tasks
            .get(&id)
            .ok_or_else(|| Error::TaskNotFound { id: id.to_string() })
    }

    /// Get a mutable reference to a task.
    pub fn task_mut(&mut self, id: Id<TaskMarker>) -> Result<&mut Task> {
        self.tasks
            .get_mut(&id)
            .ok_or_else(|| Error::TaskNotFound { id: id.to_string() })
    }

    /// Tasks ready to be started (all dependencies met).
    #[must_use]
    pub fn ready_tasks(&self) -> Vec<&Task> {
        let completed: Vec<_> = self
            .tasks
            .values()
            .filter(|t| t.status == TaskStatus::Done)
            .map(|t| t.id)
            .collect();

        self.tasks
            .values()
            .filter(|t| t.is_ready(&completed))
            .collect()
    }

    /// Assign a ready task to an idle agent and start both.
    pub fn dispatch(&mut self, task_id: Id<TaskMarker>, agent_id: Id<AgentMarker>) -> Result<()> {
        // Validate agent is idle or blocked.
        {
            let agent = self.agent(agent_id)?;
            if agent.status != AgentStatus::Idle && agent.status != AgentStatus::Running {
                return Err(Error::InvalidState {
                    id: agent_id.to_string(),
                    state: format!("{:?}", agent.status),
                    expected: "Idle or Running".into(),
                });
            }
        }

        // Start the task.
        let task = self.task_mut(task_id)?;
        task.start()?;
        task.assign(agent_id);

        // Transition agent to Running if idle.
        let agent = self.agent_mut(agent_id)?;
        if agent.status == AgentStatus::Idle {
            agent.transition(AgentStatus::Running)?;
        }

        Ok(())
    }
}

impl Default for Orchestrator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capacity_limit() {
        let mut o = Orchestrator::new();
        for i in 0..MAX_AGENTS {
            o.add_agent(format!("agent-{i}"), AgentRole::Explorer).unwrap();
        }
        assert!(o.add_agent("one-too-many", AgentRole::Writer).is_err());
    }

    #[test]
    fn dispatch_lifecycle() {
        let mut o = Orchestrator::new();
        let agent_id = o.add_agent("writer", AgentRole::Writer).unwrap();
        let task = Task::new("write module");
        let task_id = o.add_task(task);

        o.dispatch(task_id, agent_id).unwrap();

        assert_eq!(o.agent(agent_id).unwrap().status, AgentStatus::Running);
        assert_eq!(o.task(task_id).unwrap().status, TaskStatus::InProgress);
    }

    #[test]
    fn ready_tasks_respect_dependencies() {
        let mut o = Orchestrator::new();

        let t1 = Task::new("first");
        let mut t2 = Task::new("second");
        t2.depends_on(t1.id);

        let t1_id = t1.id;
        o.add_task(t1);
        o.add_task(t2);

        // Only t1 should be ready initially.
        assert_eq!(o.ready_tasks().len(), 1);

        // Complete t1.
        let t = o.task_mut(t1_id).unwrap();
        t.start().unwrap();
        t.complete().unwrap();

        // Now t2 should also be ready.
        assert_eq!(o.ready_tasks().len(), 1); // t2 is now ready
    }
}
