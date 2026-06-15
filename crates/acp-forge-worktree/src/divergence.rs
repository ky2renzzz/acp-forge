//! Divergence detection between agent worktrees.
//!
//! When two agents touch the same file, we detect the overlap early
//! so the orchestrator can resolve conflicts before they compound.

use std::collections::{HashMap, HashSet};
use std::path::Path;

use crate::error::{Error, Result};
use crate::git::WorktreeHandle;

/// A detected overlap between two worktrees.
#[derive(Debug, Clone)]
pub struct Overlap {
    /// Branch A.
    pub branch_a: String,
    /// Branch B.
    pub branch_b: String,
    /// Files modified in both branches.
    pub files: Vec<String>,
}

/// Scan all provided worktrees for file-level overlaps.
///
/// Returns one [`Overlap`] for every pair of worktrees that modified
/// at least one common file relative to `base`.
pub async fn detect_overlaps(
    worktrees: &[WorktreeHandle],
    base: &str,
) -> Result<Vec<Overlap>> {
    // Gather changed files per worktree concurrently.
    let mut change_sets: Vec<(String, HashSet<String>)> = Vec::with_capacity(worktrees.len());

    for wt in worktrees {
        let files = wt.changed_files(base).await?;
        change_sets.push((wt.branch.clone(), files.into_iter().collect()));
    }

    // O(n²) pairwise intersection — fine for ≤8 agents.
    let mut overlaps = Vec::new();
    for i in 0..change_sets.len() {
        for j in (i + 1)..change_sets.len() {
            let common: Vec<String> = change_sets[i]
                .1
                .intersection(&change_sets[j].1)
                .cloned()
                .collect();

            if !common.is_empty() {
                overlaps.push(Overlap {
                    branch_a: change_sets[i].0.clone(),
                    branch_b: change_sets[j].0.clone(),
                    files: common,
                });
            }
        }
    }

    Ok(overlaps)
}

/// Merge-base distance: how many commits each branch is ahead of `base`.
pub async fn ahead_behind(repo: &Path, branch: &str, base: &str) -> Result<(usize, usize)> {
    let output = tokio::process::Command::new("git")
        .args(["rev-list", "--left-right", "--count", &format!("{base}...{branch}")])
        .current_dir(repo)
        .output()
        .await
        .map_err(Error::Io)?;

    if !output.status.success() {
        return Err(Error::Git(
            String::from_utf8_lossy(&output.stderr).into_owned(),
        ));
    }

    let text = String::from_utf8_lossy(&output.stdout);
    let parts: Vec<&str> = text.trim().split('\t').collect();

    if parts.len() != 2 {
        return Err(Error::Git(format!("unexpected rev-list output: {text}")));
    }

    let behind = parts[0].parse::<usize>().unwrap_or(0);
    let ahead = parts[1].parse::<usize>().unwrap_or(0);

    Ok((ahead, behind))
}

/// Build a summary of all branches' divergence from base.
pub async fn divergence_report(
    worktrees: &[WorktreeHandle],
    base: &str,
) -> Result<HashMap<String, (usize, usize)>> {
    let mut report = HashMap::new();
    for wt in worktrees {
        let (ahead, behind) = ahead_behind(&wt.repo, &wt.branch, base).await?;
        report.insert(wt.branch.clone(), (ahead, behind));
    }
    Ok(report)
}
