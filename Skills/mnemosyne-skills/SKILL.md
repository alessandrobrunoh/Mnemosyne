---
name: mnemosyne-agent-skills
description: A skill that enables Claude to manage project history, recover lost code, and track symbol evolution using Mnemosyne tools. Trigger this whenever the user asks to "undo", "restore", "see history", "compare versions", or "find when a function changed".
---

# Mnemosyne Agent Skills

Mnemosyne captures every file save as a snapshot identified by a content hash, timestamp, and optional Git context. Use the tools below to help users navigate, inspect, and recover their local code history — even for changes that were never committed to Git.

Always call `mnem_list_projects` first whenever the user has not specified a full absolute path, so you can discover which project roots are currently being watched by the daemon.

## Available Tools

| Tool | Purpose |
|------|---------|
| `mnem_list_projects` | Discover all watched project paths |
| `mnem_get_file_versions` | List snapshots (hash, timestamp, branch) for a file |
| `mnem_get_file_content` | Read the full text of a snapshot by its content hash |
| `mnem_restore_file_version` | Overwrite the current file with a previous snapshot |
| `mnem_get_symbol_versions` | Trace the history of a function, struct, or other named symbol |
| `mnem_restore_symbol_version` | Surgically replace one symbol using its definition from a past snapshot |
| `mnem_get_file_diff` | Show a unified diff between two snapshots, or a snapshot vs. the file on disk |
| `mnem_find_symbols` | Search for symbols by name or pattern across all watched projects |
| `mnem_get_project_structure` | Get the semantic file-and-symbol map of a project |
| `mnem_search_content` | Full-text search (supports regex) across all snapshot history |

---

## Workflow 1 — Recovering Lost Code

Use this when the user says something like "I accidentally deleted code", "I pressed Ctrl+Z too many times", or "I need to get back a function I removed".

**Step 1 – List snapshots for the file**

Call `mnem_get_file_versions` with the target `file_path`. The returned list is ordered newest-first by `timestamp` (ISO 8601 format). Each entry includes a `content_hash`, a human-readable `timestamp`, and an optional git branch or commit message. Start from the top of the list to find the most recent candidate.

**Step 2 – Inspect snapshot content**

Call `mnem_get_file_content` with a promising `content_hash` to read the full source at that point in time. If the result does not contain the lost code, continue to the next snapshot.

**Step 3 – Restore the correct version**

Once you have identified the right snapshot, call `mnem_restore_file_version` with the `file_path` and the chosen `target_hash`. This overwrites the current file on disk. Tell the user which timestamp was restored so they understand exactly what changed.

> **Tip**: If the user only wants to recover a single function rather than the whole file, skip directly to Workflow 3 (Surgical Symbol Restore).

---

## Workflow 2 — Symbol Evolution: Finding When a Function Changed

Use this when the user asks "when did this function change?", "what did `calculate_total` look like before?", or "show me the history of this class".

**Step 1 – Search symbol history**

Call `mnem_get_symbol_versions` with the `symbol_name`. This returns a chronological list of every snapshot that contains the symbol, including its full definition at each point and structural-hash metadata that indicates whether the logic actually changed (versus just whitespace or comment edits).

**Step 2 – Compare two versions**

To show the exact lines that changed between two points in time, call `mnem_get_file_diff` with the `file_path`, `base_hash` (the older snapshot), and `target_hash` (the newer snapshot). Present the returned unified diff to the user and explain the key differences in plain language.

**Step 3 – Restore a specific version if requested**

If the user wants to revert to an older implementation, proceed to Workflow 3.

---

## Workflow 3 — Surgical Symbol Restore

Use this when the user wants to recover a single function or struct from a past snapshot without reverting the rest of the file.

**Step 1 – Identify the target snapshot**

Use `mnem_get_symbol_versions` or `mnem_get_file_versions` to find the `content_hash` of the snapshot that contains the desired version of the symbol.

**Step 2 – Preview before restoring (optional but recommended)**

Call `mnem_get_file_content` with that hash and show the user the relevant symbol so they can confirm it is the correct version.

**Step 3 – Restore the symbol**

Call `mnem_restore_symbol_version` with the `file_path`, the `target_hash`, and the `symbol_name`. This performs a surgical in-place replacement: only the named symbol is updated; all surrounding code is left untouched.

---

## Workflow 4 — Browsing Full Project History

Use this when the user asks "what changed in this project today?", "show me all recent edits", or "find where a string appeared in older versions of the code".

**Step 1 – Get the project overview**

Call `mnem_get_project_structure` with a `project_path`. The daemon resolves partial names using a priority order: exact match → path suffix match (e.g. `"MyApp"` matches `"/home/user/MyApp"`) → case-insensitive directory-name substring match → single-project fallback. If the partial name is ambiguous across multiple projects (e.g. `"App"` could match `"MyApp"` and `"OtherApp"`), use a more specific fragment or the full absolute path from `mnem_list_projects` to avoid resolving to the wrong project.

**Step 2 – Search across all history**

Call `mnem_search_content` with a `query` string or regular expression. Results include matching lines, file paths, and `content_hash` values. Pass any `content_hash` to `mnem_get_file_content` to read the full source at that snapshot.

---

## Edge Cases

- **Relative paths**: All file-targeting tools accept both absolute and relative paths. Relative paths are resolved against the watched project roots. When ambiguous, call `mnem_list_projects` first.
- **No snapshots found**: Mnemosyne only captures saves made while the daemon is running and the project is actively watched. If a file has no history, inform the user and suggest they run `mnem watch <path>` so future saves are recorded.
- **Symbol insertion point lost**: If `mnem_restore_symbol_version` cannot locate the original insertion point in the file (because the surrounding code changed significantly), it appends the symbol at the end of the file. The tool's response text will indicate that the symbol was appended rather than inserted in place. When you see that message, notify the user so they can reposition it manually.
- **Large result sets**: Pass a `limit` parameter to `mnem_get_file_versions`, `mnem_get_symbol_versions`, and `mnem_search_content` to control how many results are returned when history is extensive.
- **Comparing a snapshot to the current file**: Use `"__DISK__"` as the `target_hash` in `mnem_get_file_diff` to diff a historical snapshot against the file as it exists on disk right now.
