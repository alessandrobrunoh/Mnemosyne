# Skill: Search History

**Goal:** Perform a full-text or regex search across all historical snapshots of every
watched project.

## When to Use

Use this skill when the user wants to:
- Find where a specific string appeared in the past
- Locate a snippet of code that was deleted from the current codebase
- Search for a pattern across all saves, not just the current files

## Steps

### Step 1 – Run the content search

Call `mnem_search_content` with the search query. Optionally pass a `limit` to cap the
number of results returned.

```json
{
  "tool": "mnem_search_content",
  "arguments": {
    "query": "<text or regex pattern to search for>",
    "limit": 20
  }
}
```

Each result includes:
- `file_path` – the file that contained the match
- `line` – the matching line of text
- `content_hash` – identifier for the snapshot containing the match
- `timestamp` – when the snapshot was captured

### Step 2 – Inspect matching snapshots

For any result the user wants to explore, call `mnem_get_file_content` with the
`content_hash` to view the full file at that point in time.

```json
{
  "tool": "mnem_get_file_content",
  "arguments": {
    "content_hash": "<hash from Step 1>"
  }
}
```

### Step 3 – Restore if needed

If the user wants to bring back content found in a historical snapshot, use the
[`recover_lost_code`](./recover_lost_code.md) skill (for a whole file) or the
[`restore_symbol`](./restore_symbol.md) skill (for a single function or struct).

## Notes

- The search covers **all** snapshots across **all** watched projects, not only the
  currently tracked files.
- Queries support both plain text and regular expressions. The regex engine uses
  [Rust `regex` crate](https://docs.rs/regex) syntax (RE2-compatible; no backtracking,
  no look-ahead/look-behind). Special characters such as `.`, `*`, `(`, `)` must be
  escaped with `\` for literal matches.
- Results are not limited to the latest snapshot of each file; every saved version is
  indexed independently.
