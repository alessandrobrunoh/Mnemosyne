# Skill: Inspect File History

**Goal:** Browse the full snapshot timeline of a file, including timestamps, branches, and
content previews.

## When to Use

Use this skill when the user wants to:
- See how a file changed over time
- Find out when a specific line or feature was added
- Understand the edit history between Git commits

## Steps

### Step 1 – Fetch the snapshot list

Call `mnem_get_file_versions` to retrieve the snapshot history for the file. Use the optional
`limit` argument to control how many entries to return (default: 10).

```json
{
  "tool": "mnem_get_file_versions",
  "arguments": {
    "file_path": "<absolute-or-relative path to the file>",
    "limit": 20
  }
}
```

Each entry in the response includes:
- `timestamp` – when the snapshot was captured
- `branch` – the active Git branch at the time
- `content_hash` – unique identifier for the snapshot content

### Step 2 – Preview versions on demand

For any snapshot the user wants to inspect, call `mnem_get_file_content` with the
snapshot's `content_hash` to show the exact file contents at that moment.

```json
{
  "tool": "mnem_get_file_content",
  "arguments": {
    "content_hash": "<hash from Step 1>"
  }
}
```

### Step 3 – Offer further actions

After presenting the history, offer the user the following next steps:
- **Compare two versions** → use the [`compare_versions`](./compare_versions.md) skill
- **Restore a version** → use the [`recover_lost_code`](./recover_lost_code.md) skill
- **Track a symbol** → use the [`track_symbol_evolution`](./track_symbol_evolution.md) skill

## Notes

- If the snapshot list is empty, the file is not yet tracked. Suggest running `mnem track`
  from the project directory.
- Timestamps are stored in UTC; convert to the user's local time zone when displaying them.
