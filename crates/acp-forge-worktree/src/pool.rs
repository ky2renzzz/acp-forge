//! Worktree pool — allocates and tracks worktrees for the agent swarm.

use std::collections::HashMap;
use std::path::PathBuf;

use acp_forge_core::agent::AgentMarker;
use acp_forge_core::id::Id;

use crate::divergence;
use crate::error::Result;
use crate::git::WorktreeHandle;

/// Manages a set of worktrees, one per agent.
#[derive(Debug)]
pub struct WorktreePool {
    repo: PathBuf,
    base_ref: String,
    worktree_root: PathBuf,
    handles: HashMap<Id<AgentMarker>, WorktreeHandle>,
}

impl WorktreePool {
    /// Create a pool rooted at `worktree_root` for the given repository.
    ///
    /// `base_ref` is the branch all agent branches fork from (e.g. `main`).
    #[must_use]
    pub fn new(repo: PathBuf, base_ref: impl Into<String>, worktree_root: PathBuf) -> Self {
        Self {
            repo,
            base_ref: base_ref.into(),
            worktree_root,
            handles: HashMap::new(),
        }
    }

    /// Allocate a worktree for an agent.
    pub async fn allocate(
        &mut self,
        agent_id: Id<AgentMarker>,
        branch_name: &str,
    ) -> Result<&WorktreeHandle> {
        let dir = self.worktree_root.join(branch_name);

        let handle = WorktreeHandle::create(
            &self.repo,
            &dir,
            branch_name,
            &self.base_ref,
        )
        .await?;

        self.handles.insert(agent_id, handle);
        Ok(self.handles.get(&agent_id).unwrap())
    }

    /// Release and remove a worktree.
    pub async fn release(&mut self, agent_id: Id<AgentMarker>) -> Result<()> {
        if let Some(handle) = self.handles.remove(&agent_id) {
            handle.remove().await?;
        }
        Ok(())
    }

    /// Release all worktrees.
    pub async fn release_all(&mut self) -> Result<()> {
        let ids: Vec<_> = self.handles.keys().copied().collect();
        for id in ids {
            self.release(id).await?;
        }
        Ok(())
    }

    /// Get the worktree for a specific agent.
    #[must_use]
    pub fn get(&self, agent_id: Id<AgentMarker>) -> Option<&WorktreeHandle> {
        self.handles.get(&agent_id)
    }

    /// All active worktrees.
    #[must_use]
    pub fn all(&self) -> Vec<&WorktreeHandle> {
        self.handles.values().collect()
    }

    /// Check for file-level overlaps across all active worktrees.
    pub async fn check_overlaps(&self) -> Result<Vec<divergence::Overlap>> {
        let handles: Vec<_> = self.handles.values().cloned().collect();
        divergence::detect_overlaps(&handles, &self.base_ref).await
    }

    /// Build a divergence report (ahead/behind counts).
    pub async fn divergence_report(&self) -> Result<HashMap<String, (usize, usize)>> {
        let handles: Vec<_> = self.handles.values().cloned().collect();
        divergence::divergence_report(&handles, &self.base_ref).await
    }

    /// The base ref all branches fork from.
    #[must_use]
    pub fn base_ref(&self) -> &str {
        &self.base_ref
    }
}
