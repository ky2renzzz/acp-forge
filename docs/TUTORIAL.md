# ACP Forge × Grok Build — Integration Tutorial

This guide walks you through installing ACP Forge and integrating it into your Grok Build workflow.

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Installation](#installation)
3. [Project Setup](#project-setup)
4. [Skills Integration](#skills-integration)
5. [Hooks Integration](#hooks-integration)
6. [Worktree Divergence Detection](#worktree-divergence-detection)
7. [Session Recording & Replay](#session-recording--replay)
8. [Headless / CI Mode](#headless--ci-mode)
9. [Configuration Reference](#configuration-reference)

---

## Prerequisites

- **Rust 1.85+** — `rustup update stable`
- **Git** — any recent version
- **Grok Build** — `npm i -g @xai/grok-build` or via SuperGrok subscription
- A git repository to work in

## Installation

### Option A: Build from source

```bash
git clone https://github.com/user/acp-forge.git
cd acp-forge
cargo build --release
```

The binary is at `target/release/acp-forge` (or `acp-forge.exe` on Windows).

Add to PATH:

```bash
# Linux/macOS
cp target/release/acp-forge ~/.local/bin/

# Windows (PowerShell)
Copy-Item target\release\acp-forge.exe $env:USERPROFILE\.local\bin\
```

### Option B: cargo install (when published)

```bash
cargo install acp-forge-cli
```

### Verify

```bash
acp-forge --version
# acp-forge-cli 0.1.0
```

## Project Setup

### 1. Initialize ACP Forge in your project

```bash
cd your-project
acp-forge init --base main
```

This creates:

```
.forge/
├── config.toml     # ACP Forge configuration
├── sessions/       # Recorded execution sessions
├── skills/         # Local skills directory
└── worktrees/      # Agent worktree storage
```

### 2. Set up Grok Build config

Create `.grok/config.toml` in your project root:

```toml
[skills]
# Tell Grok Build where to find ACP Forge skills
paths = [".forge/skills", ".grok/skills"]

[hooks]
# Run ACP Forge overlap check after sub-agents complete
on_merge = ["acp-forge overlaps"]
```

### 3. Create AGENTS.md (optional but recommended)

```markdown
# Project Agent Instructions

## Tools Available

This project uses ACP Forge for orchestration. Available commands:

- `acp-forge overlaps` — check file conflicts between branches
- `acp-forge session verify <file>` — verify session integrity
- `acp-forge skill list` — show available skills

## Conventions

- Run `acp-forge overlaps` before merging parallel changes.
- Record sessions for tasks that touch 3+ files.
```

Grok Build reads this file automatically.

---

## Skills Integration

ACP Forge supports both formats that Grok Build understands.

### Using TOML skills (native format)

Create `.forge/skills/my-skill.toml`:

```toml
name = "review-rust"
version = "0.1.0"
description = "Review Rust code for common issues"
roles = ["reviewer"]
tags = ["rust", "review"]

template = """
Review {{file}} focusing on {{focus}}.

Check for:
- Unnecessary allocations
- Missing error handling
- Unsafe blocks without justification
- Clippy warnings
"""

[params.file]
description = "File to review"
required = true

[params.focus]
description = "What to focus on"
default = "correctness and performance"
required = false
```

Use it:

```bash
acp-forge skill render review-rust -p file=src/main.rs
```

### Using SKILL.md skills (Grok Build format)

Create `.grok/skills/my-skill/SKILL.md`:

```markdown
---
name: my-skill
description: What this skill does and when to activate it
---
# My Skill

Instructions for the agent when this skill is active.
```

Grok Build discovers these automatically and shows them as `/my-skill` slash commands.

### Listing all skills

```bash
# Lists both TOML and SKILL.md skills
acp-forge skill list --dir .forge/skills
acp-forge skill list --dir .grok/skills
```

---

## Hooks Integration

Grok Build hooks let you run ACP Forge at key points in the multi-agent pipeline.

### Create `.grok/hooks.yaml`:

```yaml
hooks:
  on_subagent_complete:
    # Check that code still compiles after each sub-agent finishes
    - script: cargo check --quiet
      description: "Verify compilation"

  on_merge:
    # Detect file overlaps before applying merged changes
    - script: acp-forge overlaps
      description: "Check for file conflicts between worktrees"

    # Run tests
    - script: cargo test --quiet
      description: "Run test suite"

  on_conflict:
    # Alert when sub-agents conflict
    - script: echo "⚠ Conflict detected — review with acp-forge overlaps"
      description: "Conflict notification"
```

### How it works

1. Grok Build decomposes your task into sub-tasks
2. Each sub-agent runs in its own worktree
3. When a sub-agent finishes → `on_subagent_complete` fires → `cargo check`
4. When all sub-agents finish and merge → `on_merge` fires → `acp-forge overlaps` + `cargo test`
5. If hooks fail, Grok Build shows the error and pauses

### Verify hooks are discovered

```bash
grok inspect
# Should show your hooks under "Hooks:"
```

---

## Worktree Divergence Detection

This is ACP Forge's killer feature — detecting file conflicts *before* Grok Build merges sub-agent results.

### How Grok Build handles worktrees (natively)

```
main (base)
├── worktree-agent-1 (branch: agent/auth)
├── worktree-agent-2 (branch: agent/api)
└── worktree-agent-3 (branch: agent/tests)
```

Each sub-agent works in isolation. Grok Build merges results *after* all sub-agents finish. If two agents edited the same file, you get a conflict at merge time.

### How ACP Forge improves this

ACP Forge scans all worktree branches and finds overlapping file edits *before merge*:

```bash
acp-forge overlaps
```

Output:

```
⚠ File overlaps detected:
  agent/auth ↔ agent/api
    • src/middleware/auth.rs
    • src/routes/mod.rs
```

### Setting up automatic overlap detection

Add to `.grok/hooks.yaml`:

```yaml
hooks:
  on_merge:
    - script: acp-forge overlaps
      description: "Pre-merge overlap check"
```

Now Grok Build runs the check automatically before applying merged changes.

### Programmatic access (Rust)

```rust
use acp_forge_worktree::{WorktreePool, divergence};

let pool = WorktreePool::new(repo_path, "main", worktree_root);
let overlaps = pool.check_overlaps().await?;

for o in &overlaps {
    println!("{} ↔ {}: {:?}", o.branch_a, o.branch_b, o.files);
}
```

---

## Session Recording & Replay

Record everything agents do for audit, debugging, and reproducibility.

### Record a session (programmatic)

```rust
use acp_forge_graph::{Session, SessionMeta, Recorder, EventKind, EventPayload};
use acp_forge_core::Id;

let meta = SessionMeta {
    description: "Add auth feature".into(),
    base_ref: "main".into(),
    agent_count: 3,
};
let session = Session::new(meta);
let mut recorder = Recorder::new(session);

// Record events as they happen
let agent_id = Id::new();
let event_id = recorder.tool_call(
    agent_id,
    "read_file",
    serde_json::json!({"path": "src/auth.rs"}),
    vec![],
)?;

recorder.file_edit(
    agent_id,
    "src/auth.rs",
    "added JWT validation",
    vec![event_id],
)?;

// Seal and save
let session = recorder.finish()?;
session.save(Path::new(".forge/sessions/auth-session.json"))?;
```

### Verify a session

```bash
acp-forge session verify .forge/sessions/auth-session.json
# ✓ Session — all 42 events verified.
```

Every event has a SHA-256 content hash. The session has an integrity hash over all events. Tampering with any event breaks the chain.

### Diff two sessions

```bash
acp-forge session diff session-run1.json session-run2.json
# Sessions diverge at event index 7.
#   A[7]: ToolCall — call: read_file
#   B[7]: FileEdit — modified src/auth.rs
```

Useful for comparing two runs of the same task to see where behavior diverged.

---

## Headless / CI Mode

Use ACP Forge in CI/CD pipelines alongside Grok Build's headless mode.

### GitHub Actions example

```yaml
name: ACP Forge Checks

on: [push, pull_request]

jobs:
  forge-checks:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Build ACP Forge
        run: |
          git clone https://github.com/user/acp-forge.git /tmp/acp-forge
          cd /tmp/acp-forge
          cargo build --release
          cp target/release/acp-forge /usr/local/bin/

      - name: Initialize Forge
        run: acp-forge init --base main

      - name: Check overlaps
        run: acp-forge overlaps

      - name: Verify sessions
        run: |
          for f in .forge/sessions/*.json; do
            [ -f "$f" ] && acp-forge session verify "$f"
          done

      - name: Run tests
        run: cargo test
```

### Grok Build headless + ACP Forge

```bash
# Run Grok Build in headless mode, then verify with ACP Forge
grok build -p "Add user authentication" --max-subagents 4

# Check for overlaps in the worktrees Grok Build created
acp-forge overlaps

# Verify the session if recorded
acp-forge session verify .forge/sessions/latest.json
```

---

## Configuration Reference

### `.forge/config.toml` (ACP Forge)

```toml
[session]
base_ref = "main"        # Branch all agents fork from
max_agents = 8           # Max parallel agents

[worktree]
root = ".forge/worktrees" # Where to create worktrees

[cache]
ttl_seconds = 300        # Web cache TTL
```

### `.grok/config.toml` (Grok Build)

```toml
[skills]
paths = [".forge/skills", "examples/skills"]

[hooks]
on_edit = ["cargo check --quiet"]
on_merge = ["acp-forge overlaps", "cargo test --quiet"]

[subagents]
max = 8
default_model = "grok-3"

[subagents.overrides.tests]
model = "grok-3-mini"
```

### `.grok/hooks.yaml` (Grok Build)

```yaml
hooks:
  on_subagent_complete:
    - script: cargo check --quiet
  on_merge:
    - script: acp-forge overlaps
    - script: cargo test --quiet
```

---

## Next Steps

- Browse example skills in `examples/skills/`
- Read the [README](../README.md) for architecture details
- Check the [AGENTS.md](../AGENTS.md) for coding conventions
- Create your own SKILL.md files in `.grok/skills/`

## Troubleshooting

**`acp-forge overlaps` shows "not a forge project"**
→ Run `acp-forge init --base main` in your project root.

**Grok Build doesn't see my skills**
→ Run `grok inspect` to check skill discovery paths. Ensure `.grok/skills/<name>/SKILL.md` exists.

**Session verification fails**
→ The session file was modified after sealing. Re-record the session.

**Hooks don't fire**
→ Ensure `.grok/hooks.yaml` is in the project root. Run `grok inspect` to verify.
