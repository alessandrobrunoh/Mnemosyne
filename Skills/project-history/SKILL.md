---
name: mnemosyne-project-history
description: A skill that enables Claude to browse the overall project timeline, review recent activity across many files, and compare file versions at any point in time using Mnemosyne's snapshot history. Trigger this whenever the user asks "what changed in this project today?", "show me all recent edits", "compare the current version to last night", "find where a string appeared in older code", or "give me an overview of recent activity".
---

# Project History

Mnemosyne maintains a continuous timeline of every save across all watched files in a project. Use this skill to give the user a broad overview of recent activity, search through the full history of the codebase for any content, and compare file states at different points in time — all without relying on Git.

Always call `mnem_list_projects` first whenever the user has not provided a full absolute path, so you can discover which project roots are currently being watched by the daemon.

## Available Tools

| Tool | Purpose |
|------|---------|
| `mnem_list_projects` | Discover all watched project paths |
| `mnem_get_project_structure` | Get the semantic file-and-symbol map of a project at the current moment |
| `mnem_get_file_versions` | List snapshots (hash, timestamp, branch) for a specific file, newest-first |
| `mnem_get_file_content` | Read the full source text of a snapshot by its content hash |
| `mnem_get_file_diff` | Show a unified diff between two snapshots, or a snapshot vs. the current file on disk |
| `mnem_search_content` | Full-text search (supports regex) across all snapshot history in a project |

---

## Workflow — Browsing the Project Timeline

### Step 1 — Get the project overview

Call `mnem_get_project_structure` with the `project_path`. The daemon resolves partial names using a priority order:
1. Exact match against the full absolute path.
2. Path-suffix match (e.g. `"MyApp"` matches `"/home/user/MyApp"`).
3. Case-insensitive directory-name substring match.
4. Single-project fallback (if only one project is watched).

If the name is ambiguous across multiple projects (e.g. `"App"` could match `"MyApp"` and `"OtherApp"`), use a more specific fragment or the full absolute path from `mnem_list_projects`.

The response gives you the current file tree and a map of top-level symbols in each file. Use this as an anchor to orient the rest of the exploration.

### Step 2 — Drill into a specific file's history

Call `mnem_get_file_versions` with any `file_path` of interest. The list is ordered newest-first by `timestamp` (ISO 8601). Browse the timestamps to identify when key changes occurred.

To see what changed between any two points, call `mnem_get_file_diff` with:
- `base_hash` — the `content_hash` of the older snapshot.
- `target_hash` — the `content_hash` of the newer snapshot, or `"__DISK__"` to compare against the current on-disk state.

Call `mnem_get_file_content` on any `content_hash` to read the full source at that snapshot.

### Step 3 — Search across all history

Call `mnem_search_content` with a `query` string or regular expression to find any text — a variable name, an error message, a TODO comment, or a deleted API endpoint — across all snapshots for the project.

Each result includes:
- The matching line and surrounding context.
- The `file_path` it was found in.
- The `content_hash` of the snapshot containing the match.

Pass any `content_hash` from the results to `mnem_get_file_content` to read the full source at that snapshot, or to `mnem_get_file_diff` to see how it compares to another version.

Present a summary of the search results to the user: which files had matches and how many hits each had. For excerpts, prioritise results by (1) recency — prefer matches from the most recent snapshots, (2) density — files with the most matches are likely more significant, and (3) criticality — matches in core source files (e.g. `main`, `server`, `handler`) are usually more relevant than matches in tests or generated files. Avoid dumping raw output unless the user asks.

---

## Edge Cases

- **Relative paths**: All file-targeting tools accept both absolute and relative paths. Relative paths are resolved against the watched project root. When ambiguous, call `mnem_list_projects` first.
- **No snapshots found for a file**: Mnemosyne only captures saves made while the daemon is running and the project is actively watched. If a file has no history, inform the user and suggest they run `mnem watch <path>` so future saves are recorded.
- **Large result sets**: Pass a `limit` parameter to `mnem_get_file_versions` and `mnem_search_content` to control how many results are returned when history is extensive. Fetch in pages and summarise rather than returning everything at once.
- **Comparing a snapshot to the current file**: Use `"__DISK__"` as the `target_hash` in `mnem_get_file_diff` to diff a historical snapshot against the file as it currently exists on disk.
- **Ambiguous project name**: If `mnem_get_project_structure` resolves to the wrong project, use the full absolute path returned by `mnem_list_projects` to disambiguate.

---

## Example Prompts

1. "What files changed in my project today and what did those changes look like?"
2. "Search all my code history for any snapshot that contained a `SECRET_KEY` variable and show me where it appeared."
3. "Compare `src/api/routes.py` as it was at 9 AM this morning to how it looks right now."
