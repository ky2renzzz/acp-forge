---
name: forge-overlaps
description: Check for file-level conflicts between agent worktrees before merge. Use when multiple sub-agents have been editing files and you want to detect overlapping changes early.
---
# Forge Overlaps

Detect file-level overlaps between active agent worktrees.

## When to Use

- Before merging sub-agent results
- When you suspect two agents edited the same file
- As a pre-merge safety check

## How to Run

```bash
cd <project-root>
cargo run --manifest-path <path-to-acp-forge>/Cargo.toml -- overlaps
```

## Output

- If no overlaps: `✓ No file overlaps detected.`
- If overlaps found: lists each pair of branches and the conflicting files

## Follow-up

If overlaps are found:
1. Review the conflicting files manually
2. Decide which branch's changes to keep
3. Or merge manually with `git merge-file`
