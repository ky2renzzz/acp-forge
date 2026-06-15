//! `acp-forge session` subcommands.

use std::path::Path;

use acp_forge_graph::replay::{self, Replayer};
use acp_forge_graph::Session;

/// List recorded sessions.
pub async fn list() -> anyhow::Result<()> {
    let dir = Path::new(".forge/sessions");
    if !dir.exists() {
        println!("No sessions directory.");
        return Ok(());
    }

    let mut entries: Vec<_> = std::fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .is_some_and(|ext| ext == "json")
        })
        .collect();

    entries.sort_by_key(|e| e.file_name());

    if entries.is_empty() {
        println!("No recorded sessions.");
    } else {
        println!("Recorded sessions:");
        for e in entries {
            let path = e.path();
            match Session::load(&path) {
                Ok(session) => {
                    let events = session.events.len();
                    let sealed = if session.integrity_hash.is_some() {
                        "sealed"
                    } else {
                        "open"
                    };
                    println!(
                        "  {} — {} events, {sealed}, {}",
                        path.display(),
                        events,
                        session.meta.description,
                    );
                }
                Err(e) => {
                    println!("  {} — error: {e}", path.display());
                }
            }
        }
    }

    Ok(())
}

/// Verify integrity of a session file.
pub async fn verify(path: &str) -> anyhow::Result<()> {
    let session = Session::load(Path::new(path))?;

    // Verify event hashes.
    let mut replayer = Replayer::new(&session);
    replayer.verify_all()?;

    // Verify session integrity hash.
    if session.verify_integrity()? {
        println!("✓ Session {path} — all {} events verified.", session.events.len());
    } else if session.integrity_hash.is_some() {
        anyhow::bail!("session integrity hash mismatch");
    } else {
        println!("⚠ Session {path} is not sealed (no integrity hash).");
    }

    Ok(())
}

/// Diff two session files.
pub async fn diff(a: &str, b: &str) -> anyhow::Result<()> {
    let sa = Session::load(Path::new(a))?;
    let sb = Session::load(Path::new(b))?;

    match replay::diff_sessions(&sa, &sb) {
        Some(index) => {
            println!("Sessions diverge at event index {index}.");
            if index < sa.events.len() {
                println!("  A[{index}]: {:?} — {}", sa.events[index].kind, sa.events[index].payload.summary);
            }
            if index < sb.events.len() {
                println!("  B[{index}]: {:?} — {}", sb.events[index].kind, sb.events[index].payload.summary);
            }
        }
        None => {
            println!("✓ Sessions are identical ({} events).", sa.events.len());
        }
    }

    Ok(())
}
