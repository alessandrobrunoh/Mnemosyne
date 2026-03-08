# Skill: Manage Project

**Goal:** List watched projects, start tracking a new project directory, or get a semantic
overview of a project's files and symbols.

## When to Use

Use this skill when the user wants to:
- See which projects Mnemosyne is currently tracking
- Add a new project to Mnemosyne's watch list
- Get a map of the files and symbols in a project

## Actions

### List all watched projects

Call `mnem_list_projects` to see every project directory currently tracked by the daemon.

```json
{
  "tool": "mnem_list_projects",
  "arguments": {}
}
```

Each entry includes the project's `path` and the number of tracked files and snapshots.

### Inspect a project's structure

Call `mnem_get_project_structure` with a `project_path` (a partial name is accepted, e.g.
`"MyProject"`) to get a semantic map of the project's files and the symbols they contain.

```json
{
  "tool": "mnem_get_project_structure",
  "arguments": {
    "project_path": "<full or partial project path>"
  }
}
```

### Find symbols across a project

Call `mnem_find_symbols` with a `query` and an optional `project_path` to locate functions,
structs, or classes by name anywhere in the project's snapshot history.

```json
{
  "tool": "mnem_find_symbols",
  "arguments": {
    "query": "<symbol name or pattern>",
    "project_path": "<full or partial project path (optional)>"
  }
}
```

### Start tracking a new project

If a project is not yet watched, ask the user to run the following CLI command in the
project's root directory:

```bash
mnem track
```

After tracking is enabled, the daemon begins capturing snapshots on every file save
automatically.

## Notes

- `mnem_list_projects` requires the Mnemosyne daemon to be running (`mnem on`).
- Use `mnem status` to verify the daemon is active before calling any MCP tool.
- A partial `project_path` match (e.g. `"Mnemosyne"` instead of `/home/user/Mnemosyne`) is
  resolved by the MCP server using a substring search against all watched project paths.
  The **first** matching project is used. If the partial name is ambiguous (matches more
  than one project), ask the user to provide a more specific path.
