//! `acp-forge overlaps` — check for file-level overlaps between worktrees.

use std::path::PathBuf;

use acp_forge_worktree::git;

pub async fn run() -> anyhow::Result<()> {
    let forge_dir = PathBuf::from(".forge");
    if !forge_dir.exists() {
        anyhow::bail!("not a forge project — run `acp-forge init` first");
    }

    let config_text = std::fs::read_to_string(forge_dir.join("config.toml"))?;
    let config: toml::Value = config_text.parse()?;
    let base = config
        .get("session")
        .and_then(|s| s.get("base_ref"))
        .and_then(|v| v.as_str())
        .unwrap_or("main");

    let repo = std::env::current_dir()?;
    let worktrees = git::list_worktrees(&repo).await?;

    if worktrees.len() <= 1 {
        println!("No agent worktrees to compare.");
        return Ok(());
    }

    // Build WorktreeHandles for each non-main worktree.
    let handles: Vec<git::WorktreeHandle> = worktrees
        .into_iter()
        .filter(|(_, branch)| branch != base)
        .map(|(path, branch)| git::WorktreeHandle {
            path,
            branch,
            repo: repo.clone(),
        })
        .collect();

    let overlaps =
        acp_forge_worktree::divergence::detect_overlaps(&handles, base).await?;

    if overlaps.is_empty() {
        println!("✓ No file overlaps detected.");
    } else {
        println!("⚠ File overlaps detected:");
        for o in &overlaps {
            println!("  {} ↔ {}", o.branch_a, o.branch_b);
            for f in &o.files {
                println!("    • {f}");
            }
        }
    }

    Ok(())
}
