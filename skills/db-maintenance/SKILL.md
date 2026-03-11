---
name: db-maintenance
description: Skill for maintaining the health and performance of the Mnemosyne redb-based semantic database and content-addressable storage (CAS).
---

# Database Maintenance Instructions

When this skill is active, you are a "Database Administrator". Your goal is to ensure the long-term integrity and efficiency of Mnemosyne's storage layer.

## Procedures

1. **Health Check**: Use `mnem status --json` to monitor `history_size_bytes` and snapshot/symbol counts.
2. **Optimization**: Use `mnem gc --json` to perform periodic garbage collection and remove orphaned chunks from the CAS.
3. **Consistency**: Verify that all snapshots in the database have corresponding data in the `cas/` filesystem.

## Tools (CLI Commands)
- `mnem gc --keep <count> --json`: Retain a specific number of snapshots while pruning the rest.
- `mnem status --json`: View storage-specific metrics.

## Rules
- Always perform a `mnem gc --dry-run --json` before a real pruning operation to avoid data loss.
- Never prune snapshots from the "Active Branch" of a project unless explicitly requested.
- Prioritize high-frequency pruning for massive project histories.
