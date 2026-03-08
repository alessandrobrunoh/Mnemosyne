# Mnemosyne Skills for Claude Code

This directory contains **Skills** — structured workflow guides that teach Claude Code how
to use Mnemosyne's MCP tools effectively.

Each skill covers a specific use-case and describes the exact sequence of MCP tool calls
needed to accomplish it.

## Available Skills

| Skill | Description |
|---|---|
| [`recover_lost_code`](./recover_lost_code.md) | Find and restore accidentally deleted or overwritten code |
| [`inspect_file_history`](./inspect_file_history.md) | Browse a file's full snapshot timeline |
| [`compare_versions`](./compare_versions.md) | Unified diff between two snapshots or vs. current disk state |
| [`track_symbol_evolution`](./track_symbol_evolution.md) | Trace how a function, struct, or class changed over time |
| [`restore_symbol`](./restore_symbol.md) | Surgically restore a single symbol without touching the rest of the file |
| [`search_history`](./search_history.md) | Full-text or regex search across all historical snapshots |
| [`manage_project`](./manage_project.md) | List, inspect, and add projects to Mnemosyne's watch list |

## MCP Tools Reference

The skills in this directory use the following MCP tools exposed by `mnem-mcp`:

| Tool | Purpose |
|---|---|
| `mnem_list_projects` | List all projects tracked by the daemon |
| `mnem_get_file_versions` | Get the snapshot history for a file |
| `mnem_get_file_content` | Read a file's content at a specific snapshot |
| `mnem_restore_file_version` | Overwrite a file with a historical snapshot |
| `mnem_get_file_diff` | Produce a unified diff between two snapshots |
| `mnem_get_symbol_versions` | Get the version history of a specific symbol |
| `mnem_restore_symbol_version` | Surgically restore one symbol from a historical snapshot |
| `mnem_find_symbols` | Search for symbols by name across a project |
| `mnem_get_project_structure` | Get a semantic map of files and symbols in a project |
| `mnem_search_content` | Full-text or regex search across all snapshot content |

## How Claude Code Uses These Skills

When a user asks for help with Mnemosyne, Claude Code should:

1. Identify the relevant skill from the table above.
2. Follow the step-by-step instructions in the skill file.
3. Call the MCP tools in the prescribed order.
4. Confirm with the user before any destructive action (restore, overwrite).
