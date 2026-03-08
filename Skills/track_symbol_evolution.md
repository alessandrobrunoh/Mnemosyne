# Skill: Track Symbol Evolution

**Goal:** Trace how a specific function, struct, or class changed across snapshots over time.

## When to Use

Use this skill when the user wants to:
- Understand how a particular function evolved
- See when a struct's fields were added or removed
- Find out when a class method was last modified

## Steps

### Step 1 – Retrieve symbol history

Call `mnem_get_symbol_versions` with the symbol's name to get every recorded version, along
with the file location, timestamp, and content of the symbol at each point in time.

```json
{
  "tool": "mnem_get_symbol_versions",
  "arguments": {
    "symbol_name": "<name of the function, struct, or class>"
  }
}
```

Each entry includes:
- `timestamp` – when the snapshot was captured
- `file_path` – the file containing the symbol at that time
- `content_hash` – snapshot identifier
- `structural_hash` – hash of the symbol's AST subtree (identical means logic unchanged)

### Step 2 – Highlight key changes

Compare consecutive entries using their `structural_hash`:
- **Same `structural_hash`** → only comments, whitespace, or formatting changed; the
  AST subtree is identical, so the compiled behaviour is unchanged.
- **Different `structural_hash`** → the AST subtree changed. This includes genuine logic
  changes as well as refactors that preserve semantics (e.g. extracting a helper,
  reordering independent statements). Show a diff so the user can judge the significance.

To show the code at a specific version, call `mnem_get_file_content`:

```json
{
  "tool": "mnem_get_file_content",
  "arguments": {
    "content_hash": "<hash of the snapshot>"
  }
}
```

### Step 3 – Restore if needed

If the user wants to roll back to a specific version of the symbol, use the
[`restore_symbol`](./restore_symbol.md) skill.

## Notes

- The `structural_hash` is derived from the symbol's AST, so it is stable across renames
  of local variables or changes in formatting.
- If the symbol was renamed, Mnemosyne may track it under the old name in earlier snapshots.
  Use [`search_history`](./search_history.md) to locate it by content if needed.
