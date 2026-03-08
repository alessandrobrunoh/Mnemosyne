# Skill: Recover Lost Code

**Goal:** Recover accidentally deleted or overwritten code from a file's Mnemosyne history.

## When to Use

Use this skill when the user says things like:
- "I accidentally deleted a function"
- "I overwrote my file and need it back"
- "I lost some code and can't find it"
- "Ctrl+Z didn't go back far enough"

## Steps

### Step 1 – List all snapshots for the file

Call `mnem_get_file_versions` with the target file path to retrieve the full snapshot history,
including timestamps, Git branch names, and content hashes.

```json
{
  "tool": "mnem_get_file_versions",
  "arguments": {
    "file_path": "<absolute-or-relative path to the file>"
  }
}
```

### Step 2 – Preview promising snapshots

For each snapshot that may contain the lost code, call `mnem_get_file_content` with its
`content_hash` to read the file's contents at that point in time. Compare timestamps to
narrow down the right snapshot.

```json
{
  "tool": "mnem_get_file_content",
  "arguments": {
    "content_hash": "<hash from Step 1>"
  }
}
```

### Step 3 – Restore the correct version

Once you have identified the snapshot containing the lost code, restore it with
`mnem_restore_file_version`. This overwrites the current file with the historical snapshot.

```json
{
  "tool": "mnem_restore_file_version",
  "arguments": {
    "file_path": "<path to the file>",
    "content_hash": "<hash of the chosen snapshot>"
  }
}
```

> **Tip:** If the user only wants to recover a single function or struct—not the entire
> file—use the [`restore_symbol`](./restore_symbol.md) skill instead.

## Notes

- Always confirm with the user before performing Step 3 (the restore is destructive).
- If `mnem_get_file_versions` returns no results, the file may not be tracked. Suggest
  running `mnem track` from the project root to start tracking.
