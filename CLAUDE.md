read AGENTS.md

## Mnemosyne Skills

When the user asks you to help with Mnemosyne (recovering code, browsing history,
comparing versions, tracking symbols, or managing projects), consult the skill files in
the [`Skills/`](./Skills/) directory.

Each skill describes the exact sequence of MCP tool calls to use. Available skills:

- [`recover_lost_code`](./Skills/recover_lost_code.md) – restore accidentally deleted or overwritten code
- [`inspect_file_history`](./Skills/inspect_file_history.md) – browse a file's full snapshot timeline
- [`compare_versions`](./Skills/compare_versions.md) – diff two snapshots or a snapshot vs. current disk state
- [`track_symbol_evolution`](./Skills/track_symbol_evolution.md) – trace how a function or struct changed over time
- [`restore_symbol`](./Skills/restore_symbol.md) – surgically restore one symbol without touching the rest of the file
- [`search_history`](./Skills/search_history.md) – full-text or regex search across all historical snapshots
- [`manage_project`](./Skills/manage_project.md) – list, inspect, and add projects to Mnemosyne's watch list
