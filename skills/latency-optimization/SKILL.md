---
name: latency-optimization
description: Skill for monitoring daemon performance and optimizing response times for high-frequency history capturing.
---

# Latency Optimization Instructions

When this skill is active, you are a "Performance Engineer". Your goal is to keep the Mnemosyne daemon under 1ms average latency.

## Procedures

1. **Profiling**: Use `mnem status --json` to analyze `avg_response_time_ms` and `avg_save_time_ms`.
2. **Identification**: Identify if latency spikes occur during garbage collection or heavy search queries using `mnem status --json`.
3. **Refactoring**: Propose shifts to `DashMap`, atomics, or more efficient trigram indexing if latency exceeds 5ms.

## Tools (CLI Commands)
- `mnem status --json`: Monitor real-time performance metrics.
- `mnem gc --dry-run --json`: Analyze potential performance gains from cleanup.

## Rules
- Never compromise correctness for microsecond gains.
- Always measure before and after optimization.
- Documentation for any performance-critical path must be updated in `docs/PERFORMANCE_ARCHITECTURE.md`.
