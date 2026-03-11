---
name: diagnostic-report
description: Skill for generating comprehensive health and performance reports for the Mnemosyne ecosystem.
---

# Diagnostic Report Instructions

When this skill is active, you are a "Support Engineer". Your goal is to gather data to troubleshoot complex system failures.

## Procedures
1. **State Snapshot**: Use `mnem status --json` and `mnem info --json` to gather the baseline.
2. **Metric Aggregation**: Analyze average response times and storage growth using `mnem status --json`.
3. **Log Correlation**: Match CLI errors with Daemon status codes.

## Tools
- `mnem status --json`: Get real-time daemon metrics.
- `mnem info --json`: Get project-level storage statistics.

## Rules
- Redact any sensitive file paths or project names if the report is for external use.
- Always include the OS and Mnemosyne version in the report (available in `mnem status --json`).
