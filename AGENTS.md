# ACP Forge — Agent Instructions

> This file is read by Grok Build and compatible agents (Claude Code, Codex CLI).
> It defines how agents should work within this project.

## Project Overview

ACP Forge is a Rust workspace that extends xAI Grok Build with:
- Reproducible execution graphs (DAG recording + replay)
- Pre-merge worktree divergence detection
- Composable skill pipelines (TOML + SKILL.md)
- Cross-reference web source verification

## Architecture

Monorepo with 6 crates under `crates/`. Each crate is independently compilable.

| Crate | Purpose | Key types |
|-------|---------|-----------|
| `acp-forge-core` | Orchestration primitives | `Agent`, `Task`, `Orchestrator`, `Id<T>` |
| `acp-forge-worktree` | Git worktree management | `WorktreePool`, `WorktreeHandle` |
| `acp-forge-graph` | Execution DAG | `Session`, `Event`, `Recorder`, `Replayer` |
| `acp-forge-skills` | Skill templates | `Skill`, `Registry`, `Pipeline` |
| `acp-forge-web` | Web verification | `Provider`, `Cache`, `Verification` |
| `acp-forge-cli` | CLI binary | clap subcommands |

## Conventions

- **Rust 2024 edition.** Use `edition = "2024"` patterns.
- **Zero warnings policy.** `cargo check` must produce no warnings.
- **Every public type gets a doc comment.**
- **Tests next to code.** Use `#[cfg(test)] mod tests` in each module.
- **Error types per crate.** Each crate has its own `error.rs` with `thiserror`.
- **Typed IDs.** Use `Id<T>` from `acp-forge-core` — never raw UUIDs.
- **No `unwrap()` in library code.** Only in tests.
- **Imports:** workspace dependencies via `Cargo.toml`, not path hacks.

## Build & Test

```bash
cargo check          # Must pass with 0 warnings
cargo test           # Must pass all tests
cargo build --release  # For release binary
```

## When Making Changes

1. Run `cargo check` before committing — zero warnings required.
2. Add tests for new public functions.
3. Update the relevant crate's `lib.rs` re-exports if adding new modules.
4. If adding a dependency, add it to workspace `Cargo.toml` first.
5. Skill files: use TOML for parameterized skills, SKILL.md for simple prompts.

## File Patterns

- `crates/*/src/*.rs` — Rust source files
- `examples/skills/*.toml` — Native skill templates
- `examples/skills/*/SKILL.md` — Grok Build compatible skills
- `.grok/` — Grok Build configuration (not tracked in git)
