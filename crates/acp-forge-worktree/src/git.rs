//! Low-level git worktree operations via subprocess.
//!
//! We shell out to `git` rather than linking libgit2 because worktree
//! management involves index locking semantics that libgit2 does not
//! fully support.

use std::path::{Path, PathBuf};
use tokio::process::Command;
use tracing::instrument;

use crate::error::{Error, Result};

/// A handle to a single git worktree on disk.
#[derive(Debug, Clone)]
pub struct WorktreeHandle {
    /// Absolute path to the worktree directory.
    pub path: PathBuf,
    /// Branch name checked out in this worktree.
    pub branch: String,
    /// The repo root that owns this worktree.
    pub repo: PathBuf,
}

impl WorktreeHandle {
    /// Create a new worktree with a fresh branch forked from `base`.
    #[instrument(skip_all, fields(branch = %branch, base = %base))]
    pub async fn create(
        repo: &Path,
        worktree_dir: &Path,
        branch: &str,
        base: &str,
    ) -> Result<Self> {
        if worktree_dir.exists() {
            return Err(Error::AlreadyExists {
                path: worktree_dir.to_path_buf(),
            });
        }

        let output = Command::new("git")
            .args(["worktree", "add", "-b", branch])
            .arg(worktree_dir)
            .arg(base)
            .current_dir(repo)
            .output()
            .await?;

        if !output.status.success() {
            return Err(Error::Git(
                String::from_utf8_lossy(&output.stderr).into_owned(),
            ));
        }

        tracing::info!(path = %worktree_dir.display(), "worktree created");

        Ok(Self {
            path: worktree_dir.to_path_buf(),
            branch: branch.to_owned(),
            repo: repo.to_path_buf(),
        })
    }

    /// Remove this worktree and its branch.
    #[instrument(skip_all, fields(path = %self.path.display()))]
    pub async fn remove(&self) -> Result<()> {
        let output = Command::new("git")
            .args(["worktree", "remove", "--force"])
            .arg(&self.path)
            .current_dir(&self.repo)
            .output()
            .await?;

        if !output.status.success() {
            return Err(Error::Git(
                String::from_utf8_lossy(&output.stderr).into_owned(),
            ));
        }

        // Delete the branch after removing the worktree.
        let _ = Command::new("git")
            .args(["branch", "-D", &self.branch])
            .current_dir(&self.repo)
            .output()
            .await;

        tracing::info!("worktree removed");
        Ok(())
    }

    /// Get the HEAD commit SHA of this worktree's branch.
    pub async fn head_sha(&self) -> Result<String> {
        let output = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(&self.path)
            .output()
            .await?;

        if !output.status.success() {
            return Err(Error::Git(
                String::from_utf8_lossy(&output.stderr).into_owned(),
            ));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
    }

    /// Get the diff summary between this worktree and its base.
    pub async fn diff_stat(&self, base: &str) -> Result<String> {
        let output = Command::new("git")
            .args(["diff", "--stat", base])
            .current_dir(&self.path)
            .output()
            .await?;

        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }

    /// List files changed in this worktree relative to `base`.
    pub async fn changed_files(&self, base: &str) -> Result<Vec<String>> {
        let output = Command::new("git")
            .args(["diff", "--name-only", base])
            .current_dir(&self.path)
            .output()
            .await?;

        if !output.status.success() {
            return Err(Error::Git(
                String::from_utf8_lossy(&output.stderr).into_owned(),
            ));
        }

        Ok(String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter(|l| !l.is_empty())
            .map(String::from)
            .collect())
    }
}

/// List all worktrees for a repository.
pub async fn list_worktrees(repo: &Path) -> Result<Vec<(PathBuf, String)>> {
    let output = Command::new("git")
        .args(["worktree", "list", "--porcelain"])
        .current_dir(repo)
        .output()
        .await?;

    if !output.status.success() {
        return Err(Error::Git(
            String::from_utf8_lossy(&output.stderr).into_owned(),
        ));
    }

    let text = String::from_utf8_lossy(&output.stdout);
    let mut result = Vec::new();
    let mut current_path: Option<PathBuf> = None;

    for line in text.lines() {
        if let Some(path) = line.strip_prefix("worktree ") {
            current_path = Some(PathBuf::from(path));
        } else if let Some(branch) = line.strip_prefix("branch refs/heads/") {
            if let Some(path) = current_path.take() {
                result.push((path, branch.to_owned()));
            }
        } else if line.is_empty() {
            current_path = None;
        }
    }

    Ok(result)
}
