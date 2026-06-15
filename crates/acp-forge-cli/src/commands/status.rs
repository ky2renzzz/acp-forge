//! `acp-forge status` — display agent and session status.

pub async fn run() -> anyhow::Result<()> {
    let forge_dir = std::path::Path::new(".forge");
    if !forge_dir.exists() {
        anyhow::bail!("not a forge project — run `acp-forge init` first");
    }

    println!("ACP Forge status");
    println!("─────────────────");

    // List worktree directories.
    let wt_dir = forge_dir.join("worktrees");
    if wt_dir.exists() {
        let entries: Vec<_> = std::fs::read_dir(&wt_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().map(|ft| ft.is_dir()).unwrap_or(false))
            .collect();

        if entries.is_empty() {
            println!("  No active worktrees.");
        } else {
            println!("  Active worktrees:");
            for entry in entries {
                println!("    • {}", entry.file_name().to_string_lossy());
            }
        }
    }

    // List sessions.
    let sessions_dir = forge_dir.join("sessions");
    if sessions_dir.exists() {
        let count = std::fs::read_dir(&sessions_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .is_some_and(|ext| ext == "json")
            })
            .count();
        println!("  Recorded sessions: {count}");
    }

    Ok(())
}
