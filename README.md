# ACP Forge

**Multi-agent orchestration framework for [xAI Grok Build](https://x.ai/cli) and the [Agent Client Protocol](https://agentclientprotocol.com).**

ACP Forge extends Grok Build's native multi-agent capabilities with reproducible execution graphs, composable skill templates, advanced worktree divergence detection, and verified web source integration. It plugs directly into Grok Build's ACP agent pipeline as a set of external orchestration tools.

## Why

Grok Build ships with up to 8 parallel sub-agents working in git worktrees — but its orchestration is a black box. You can't replay a session, detect file conflicts before merge, compose reusable skill pipelines, or verify web-sourced claims independently.

ACP Forge fills those gaps:

| Gap in Grok Build | ACP Forge solution |
|---|---|
| No session replay / audit trail | Execution DAG with content-addressable hashing |
| Conflict detection is post-merge only | Pre-merge divergence detection across worktrees |
| Skills are flat `.md` files | Composable pipelines with dependency resolution |
| No claim verification | Cross-reference verification across multiple sources |
| Orchestration not programmable | Typed Rust API for agent/task lifecycle management |

## Grok Build Integration

ACP Forge works as an external ACP agent that Grok Build can invoke:

```yaml
# .grok/config.yaml
acp_agents:
  - name: forge-orchestrator
    endpoint: http://localhost:9090/acp
    trigger: on_decompose
```

It also reads Grok Build's native SKILL.md format and the `.grok/skills/` directory convention.

## Architecture

```
acp-forge/
├── crates/
│   ├── acp-forge-core       # Agent, Task, Orchestrator, typed IDs
│   ├── acp-forge-worktree   # Git worktree pool + divergence detection
│   ├── acp-forge-graph      # Execution DAG recording + replay
│   ├── acp-forge-skills     # Skill templates (TOML + SKILL.md), registry, composition
│   ├── acp-forge-web        # Web source providers, cache, verification
│   └── acp-forge-cli        # CLI binary
└── examples/
    └── skills/              # Example skills (TOML + SKILL.md formats)
```

**Built on:** ACP SDK 0.14 · Rust 2024 edition · tokio · serde · clap

## Core Concepts

### Orchestrator
Manages agents and tasks. Enforces capacity limits (max 8 agents — matching Grok Build's limit). Schedules work based on task dependency graphs.

### Worktree Isolation
Each agent gets its own git worktree branched from a base ref. Divergence detection surfaces overlapping file edits *before merge* — the main gap in Grok Build's current conflict resolution flow.

### Execution Graph
Every action (tool call, file edit, search, LLM response) is recorded as a node in a DAG. Nodes carry SHA-256 content hashes. Sessions serialize to canonical JSON and replay deterministically. Two sessions can be diffed to find the exact point of divergence.

### Skills (Dual Format)
- **TOML** — native format with typed parameters, defaults, version tracking, composition
- **SKILL.md** — xAI Grok Build / Anthropic compatible format (YAML frontmatter + markdown)

Skills declare dependencies and compose into ordered pipelines via topological sort.

### Web Verification
Query multiple source providers and cross-reference results. Claims pass when ≥N independent sources corroborate them. Responses cached with configurable TTL.

## Quick Start

```bash
# Build
cargo build

# Run all tests
cargo test

# Initialize a forge session
cargo run -- init --base main

# List skills (loads both TOML and SKILL.md)
cargo run -- skill list --dir examples/skills

# Render a skill template
cargo run -- skill render code-review -p file=src/main.rs

# Check worktree overlaps across agent branches
cargo run -- overlaps

# Verify a recorded session's integrity
cargo run -- session verify .forge/sessions/session-001.json

# Diff two sessions to find divergence
cargo run -- session diff session-a.json session-b.json
```

## CLI Commands

| Command | Description |
|---------|-------------|
| `init --base <ref>` | Initialize `.forge/` directory structure |
| `spawn <name> --role <role>` | Create an agent with a role |
| `status` | Show agents, worktrees, and session info |
| `overlaps` | Detect file conflicts between agent worktrees |
| `session list` | List recorded sessions |
| `session verify <path>` | Verify session integrity (hash chain) |
| `session diff <a> <b>` | Find divergence point between sessions |
| `skill list --dir <path>` | List available skills (TOML + SKILL.md) |
| `skill render <name> -p key=val` | Render a skill template |

## Skill Format (TOML)

```toml
name = "code-review"
version = "0.1.0"
description = "Review code for bugs and security issues"
roles = ["reviewer"]
tags = ["review", "security"]

template = """
Review {{file}} for {{focus}}.
"""

[params.file]
description = "Path to review"
required = true

[params.focus]
description = "Review focus"
default = "correctness"
required = false
```

## Skill Format (SKILL.md — Grok Build compatible)

```markdown
---
name: security-audit
description: Perform a security audit on code
---
# Security Audit

Review the provided code for injection, auth bypass, and secrets leakage.
```

Place in `.grok/skills/<name>/SKILL.md` to auto-discover.

## Design Principles

- **Minimal.** No unnecessary abstractions. Every type earns its place.
- **Correct.** Typed IDs prevent mixing domains. State machines enforce valid transitions.
- **Reproducible.** Content-addressable hashing. Canonical JSON. Deterministic replay.
- **Compatible.** ACP 0.14, Grok Build SKILL.md, `.grok/` directory conventions.
- **Composable.** Each crate is independently useful. The CLI is just one consumer.

## Requirements

- Rust 1.85+ (edition 2024)
- Git (for worktree operations)
- Optional: xAI Grok Build (for ACP integration)

## Roadmap

- [ ] ACP server endpoint (run as a Grok Build external agent)
- [ ] `grok inspect` integration for execution graph visualization
- [ ] AGENTS.md auto-generation from skill registry
- [ ] Headless mode (`-p` flag) for CI/CD pipelines
- [ ] Skill marketplace index (searchable registry)

## License

MIT
