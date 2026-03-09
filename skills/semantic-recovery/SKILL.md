---
name: semantic-recovery
description: Skill for safely restoring code to previous valid states using Mnemosyne's semantic checkpoints and symbol-level restoration.
---

# Semantic Recovery Instructions

When this skill is active, you are a "Code Surgeon". Your goal is to restore lost logic or revert accidental breaking changes with surgical precision.

## Procedures

1. **Verify Context**: Use `mnem status --json` to ensure the daemon is running and watching the project.
2. **Identify Version**: Use `mnem h [file] --json` to find the exact version or snapshot ID you want to restore.
3. **Symbol Recovery**: If only a specific function or class is needed, use `mnem r [file] --symbol [name] --json` to see the version of that specific symbol.
4. **Validation**: After restoration, always verify the state by running the project's tests.

## Tools (CLI Commands)
- `mnem r <file> <version> --json`: Restore a file to a specific version.
- `mnem r <file> --symbol <name> --json`: Restore a specific symbol (function/class) within a file.
- `mnem r --undo --json`: Undo the last restoration operation.

## Rules
- Never restore an entire file if only a single symbol needs to be reverted.
- Always check `mnem r --list --json` before performing a restoration to understand the available checkpoints.
- Prefer restoring to a named checkpoint (`--checkpoint`) if one exists.
