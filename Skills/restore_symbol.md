# Skill: Restore Symbol

**Goal:** Surgically restore a single symbol (function, struct, or class) from a previous
snapshot without touching any other code in the file.

## When to Use

Use this skill when the user wants to:
- Undo a bad change to one specific function while keeping all other edits
- Recover a deleted method without reverting the whole file
- Roll back a struct definition to an earlier state

## Steps

### Step 1 – Find relevant snapshots for the symbol

Call `mnem_get_symbol_versions` with the symbol name to retrieve all recorded versions of
that symbol.

```json
{
  "tool": "mnem_get_symbol_versions",
  "arguments": {
    "symbol_name": "<name of the symbol to restore>"
  }
}
```

### Step 2 – Preview the target version

Once the user selects a snapshot to restore from, call `mnem_get_file_content` with the
snapshot's `content_hash` to confirm the symbol looks correct before restoring.

```json
{
  "tool": "mnem_get_file_content",
  "arguments": {
    "content_hash": "<hash of the chosen snapshot>"
  }
}
```

### Step 3 – Perform the surgical restore

Call `mnem_restore_symbol_version` with:
- `file_path` – the file containing the symbol
- `target_hash` – the content hash of the chosen snapshot
- `symbol_name` – the name of the symbol to restore

```json
{
  "tool": "mnem_restore_symbol_version",
  "arguments": {
    "file_path": "<path to the file>",
    "target_hash": "<hash of the chosen snapshot>",
    "symbol_name": "<name of the symbol>"
  }
}
```

This replaces **only** the specified symbol in the file; all surrounding code is left
untouched.

## Notes

- Always confirm with the user before performing Step 3 (the restore modifies the file).
- If the symbol no longer exists in the current file (it was deleted entirely), Mnemosyne
  will re-insert it using the surrounding AST context recorded in the snapshot (the
  sibling symbols immediately before and after it). If that context no longer exists, the
  symbol is appended at the end of the file.
- For restoring an entire file rather than a single symbol, use the
  [`recover_lost_code`](./recover_lost_code.md) skill instead.
