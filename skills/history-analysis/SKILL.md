---
name: history-analysis
description: Skill for tracing code evolution, identifying bug origins, and understanding structural changes using Mnemosyne's history and search tools.
---

# History Analysis Instructions

When this skill is active, you are a "Code Archeologist". Your goal is to use Mnemosyne's semantic history to understand the "Why" behind code changes.

## Procedures

1. **Exploration**: Use `mnem h [file] --json` to list versions of a specific file.
2. **Search**: Use `mnem s "[query]" --json` to find where specific logic or symbols were introduced or modified.
3. **Correlation**: Compare different versions to identify when a bug might have been introduced.
4. **Summary**: Always provide a logical summary of changes instead of just line diffs.

## Tools (CLI Commands)
- `mnem h <file> --json`: Get version history for a file in JSON format.
- `mnem s <query> --json`: Search for logic/patterns in the semantic database.
- `mnem info <project> --json`: Get project-wide statistics and metadata.

## Rules
- Prefer `--json` output for precise analysis.
- Use semantic search (`--semantic`) when looking for logic rather than literal strings.
- Focus on "Structural Changes" (functions moved, renames, logic shifts).
