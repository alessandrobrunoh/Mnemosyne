# Skill: Compare Versions

**Goal:** Produce a unified diff between two historical snapshots of a file, or between a
snapshot and the current file on disk.

## When to Use

Use this skill when the user wants to:
- See what changed between two saves
- Compare the current file to an older version
- Understand what was different at a specific point in time

## Steps

### Step 1 – Ensure snapshot hashes are available

If the user hasn't provided explicit content hashes, first call `mnem_get_file_versions` to
list the available snapshots and let the user pick the versions to compare.

```json
{
  "tool": "mnem_get_file_versions",
  "arguments": {
    "file_path": "<path to the file>"
  }
}
```

### Step 2 – Generate the diff

Call `mnem_get_file_diff` with:
- `file_path` – the file to compare
- `base_hash` – content hash of the **older** snapshot (diff base)
- `target_hash` – content hash of the **newer** snapshot, **or** the special value
  `"__DISK__"` to compare the historical snapshot against the current file on disk

```json
{
  "tool": "mnem_get_file_diff",
  "arguments": {
    "file_path": "<path to the file>",
    "base_hash": "<hash of the older snapshot>",
    "target_hash": "<hash of the newer snapshot or '__DISK__'>"
  }
}
```

The response contains a unified diff that can be displayed directly to the user.

### Step 3 – Offer further actions

After presenting the diff, offer the user:
- **Restore the base version** → use the [`recover_lost_code`](./recover_lost_code.md) skill
- **Restore a single symbol** → use the [`restore_symbol`](./restore_symbol.md) skill

## Notes

- Use `target_hash: "__DISK__"` as a quick way to answer "what did I change since this
  snapshot?" without needing a second hash.
- If the diff is empty, the two versions are identical in content.
