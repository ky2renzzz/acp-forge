---
name: forge-session
description: Record, verify, and diff multi-agent execution sessions. Use to audit what agents did, replay sessions for debugging, or compare two runs.
---
# Forge Session

Manage reproducible execution sessions.

## Commands

### List sessions
```bash
cargo run --manifest-path <path-to-acp-forge>/Cargo.toml -- session list
```

### Verify session integrity
```bash
cargo run --manifest-path <path-to-acp-forge>/Cargo.toml -- session verify <path-to-session.json>
```
Checks every event's SHA-256 content hash and the session's integrity hash.

### Diff two sessions
```bash
cargo run --manifest-path <path-to-acp-forge>/Cargo.toml -- session diff <session-a.json> <session-b.json>
```
Finds the first point where two sessions diverge.

## Use Cases

- **Audit:** verify that a recorded session was not tampered with
- **Debug:** replay a failing session to find the exact divergence point
- **Compare:** diff two runs of the same task to see what changed
