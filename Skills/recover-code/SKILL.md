---
name: mnemosyne-recover-code
description: A skill that enables Claude to find and restore deleted or overwritten code using Mnemosyne's local snapshot history. Trigger this whenever the user says "I accidentally deleted code", "I lost a function", "undo my last save", "restore the file", or "get back what I had before".
---

# Recover Code

Mnemosyne records every file save as an immutable snapshot identified by a content hash and a timestamp. Use this skill to walk the user back through those snapshots and recover code that was deleted, overwritten, or otherwise lost — even if the change was never committed to Git.

Always call `mnem_list_projects` first whenever the user has not provided a full absolute path, so you can discover which project roots are currently being watched by the daemon.

## Available Tools

| Tool | Purpose |
|------|---------|
| `mnem_list_projects` | Discover all watched project paths |
| `mnem_get_file_versions` | List snapshots (hash, timestamp, branch) for a file, newest-first |
| `mnem_get_file_content` | Read the full source text of a snapshot by its content hash |
| `mnem_restore_file_version` | Overwrite the current file on disk with a chosen snapshot |
| `mnem_get_file_diff` | Show a unified diff between two snapshots, or a snapshot vs. the current file on disk |

---

## Workflow — Recovering a Lost File or Code Block

### Step 1 — List snapshots for the file

Call `mnem_get_file_versions` with the target `file_path`. The list is ordered newest-first by `timestamp` (ISO 8601). Each entry contains:
- `content_hash` — unique identifier for that snapshot
- `timestamp` — when the save occurred
- `git_branch` / `git_commit` — optional Git context at the time of the save

Start from the most recent snapshot and work backwards until you find a candidate that predates the accidental change.

### Step 2 — Inspect the candidate snapshot

Call `mnem_get_file_content` with the chosen `content_hash` to read the full source at that point in time. Check whether it contains the lost code. If not, continue to the next snapshot in the list.

To make it easier for the user to see what will change, call `mnem_get_file_diff` with the chosen hash as `base_hash` and `"__DISK__"` as `target_hash`. This shows the diff from the historical snapshot to the current on-disk file: lines prefixed with `-` are in the snapshot (what will be restored), and lines prefixed with `+` are only in the current file (what will be lost). Present this to the user before restoring so they can confirm.

### Step 3 — Restore the correct version

Once you have confirmed the right snapshot, call `mnem_restore_file_version` with `file_path` and the chosen `target_hash`. This overwrites the current file on disk atomically.

After the call succeeds, tell the user:
- The timestamp of the snapshot that was restored.
- The content hash, so they can reference it again if needed.
- That they can call `mnem_get_file_diff` at any time to compare the restored version against any other snapshot.

> **Tip**: If the user only needs to recover a single function or struct rather than the entire file, use the `mnemosyne-symbol-evolution` skill instead, which covers surgical per-symbol restoration.

---

## Edge Cases

- **Relative paths**: All file-targeting tools accept both absolute and relative paths. Relative paths are resolved against the watched project root. When ambiguous, call `mnem_list_projects` first.
- **No snapshots found**: Mnemosyne only records saves made while the daemon is running and the project is actively watched. If `mnem_get_file_versions` returns an empty list, inform the user and suggest they run `mnem watch <path>` so future saves are captured.
- **Large histories**: Pass a `limit` parameter to `mnem_get_file_versions` to page through extensive histories without overwhelming the context window.
- **Comparing to current disk state**: Use `"__DISK__"` as the `target_hash` in `mnem_get_file_diff` to diff a historical snapshot against the file as it currently exists on disk.

---

## Example Prompts

1. "I accidentally deleted the `validate_input` function in `src/handlers/auth.rs` about 10 minutes ago. Can you get it back?"
2. "I saved the wrong version of `utils/config.py`. Please restore the version from earlier today, before my last three saves."
3. "Show me what `src/main.go` looked like two hours ago and then put it back if it looks right."
