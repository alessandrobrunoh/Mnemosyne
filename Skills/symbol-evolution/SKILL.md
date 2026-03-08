---
name: mnemosyne-symbol-evolution
description: A skill that enables Claude to trace how a specific function, struct, class, or other named symbol changed over time using Mnemosyne's snapshot history. Trigger this whenever the user asks "when did this function change?", "what did X look like before?", "show me the history of this class", "who changed this method?", or "restore the old version of this function".
---

# Symbol Evolution

Mnemosyne tracks named symbols — functions, structs, classes, enums, and other language constructs — across every file save. Use this skill to reconstruct the full evolution of a symbol, compare specific versions, or surgically restore a single symbol to a past state without touching the rest of the file.

Always call `mnem_list_projects` first whenever the user has not provided a full absolute path, so you can discover which project roots are currently being watched by the daemon.

## Available Tools

| Tool | Purpose |
|------|---------|
| `mnem_list_projects` | Discover all watched project paths |
| `mnem_get_symbol_versions` | Trace every snapshot containing a named symbol, with its full definition and structural-hash metadata at each point |
| `mnem_get_file_content` | Read the full source text of a snapshot by its content hash |
| `mnem_get_file_diff` | Show a unified diff between two snapshots, or a snapshot vs. the current file on disk |
| `mnem_restore_symbol_version` | Surgically replace one symbol in the current file using its definition from a past snapshot |
| `mnem_find_symbols` | Search for symbols by name or pattern across all watched projects |

---

## Workflow — Tracing a Symbol's History

### Step 1 — Find the symbol across history

Call `mnem_get_symbol_versions` with the `symbol_name`. This returns a chronological list of every snapshot that contains the symbol, including:
- The full source definition of the symbol at that point.
- A `structural_hash` that indicates whether the logic changed (a different hash means meaningful code changes, not just whitespace or comment edits).
- The `content_hash` of the containing snapshot and the `timestamp` of that save.

If you are unsure of the exact symbol name or want to search across the whole project, call `mnem_find_symbols` first with a partial name or pattern.

### Step 2 — Compare two versions

To show the user the exact lines that changed between two points in time, call `mnem_get_file_diff` with:
- `file_path` — the source file containing the symbol.
- `base_hash` — the `content_hash` of the older snapshot.
- `target_hash` — the `content_hash` of the newer snapshot (or `"__DISK__"` to compare against the current on-disk file).

Present the returned unified diff and explain the key differences in plain language. Focus on the section of the diff that covers the target symbol; ignore unrelated hunks unless the user asks.

### Step 3 — Restore a specific version (if requested)

If the user wants to revert a single symbol to a past implementation without touching the rest of the file:

Call `mnem_restore_symbol_version` with the `file_path`, the `target_hash` of the snapshot that contains the desired implementation, and the `symbol_name`. This performs an in-place replacement: only the named symbol is updated; all surrounding code is left untouched.

Before calling the restore, show the user the symbol definition from the target snapshot (via `mnem_get_file_content`) so they can confirm it is the correct version.

After the call succeeds, report:
- The timestamp of the restored snapshot.
- That only the named symbol was modified; the rest of the file is unchanged.

---

## Edge Cases

- **Relative paths**: All file-targeting tools accept both absolute and relative paths. When ambiguous, call `mnem_list_projects` to find the full absolute root.
- **Symbol not found in older snapshots**: If the symbol does not appear in early snapshots, it was added later. `mnem_get_symbol_versions` will only return snapshots from after the symbol's first appearance.
- **Symbol insertion point lost**: If `mnem_restore_symbol_version` cannot locate the original insertion point (because the surrounding code changed significantly), it appends the symbol at the end of the file and the response text will say so. When you see that message, notify the user so they can reposition it manually.
- **Structural hash unchanged**: If consecutive snapshots share the same `structural_hash`, the symbol's logic did not change between those saves (only whitespace, comments, or formatting changed). You can safely skip those snapshots when looking for meaningful changes.
- **Large histories**: Pass a `limit` parameter to `mnem_get_symbol_versions` to control how many results are returned when history is extensive.

---

## Example Prompts

1. "Show me every time the `calculate_discount` function changed in `src/pricing.rs` and explain what each change did."
2. "What did the `User` struct look like before the refactor I did this morning in `models/user.py`?"
3. "Restore the old version of the `parse_config` function in `lib/config.go` — the one from before my last two saves."
