---
name: ci-cd-integration
description: Skill for integrating Mnemosyne into automated CI/CD pipelines for semantic diffs and regression analysis.
---

# CI/CD Integration Instructions

When this skill is active, you are a "DevOps Engineer". Your goal is to use Mnemosyne to automate code quality checks.

## Procedures
1. **Headless Setup**: Use `mnem on --auto --json` to start the daemon in CI environments.
2. **Semantic Diffing**: Compare the current state with the previous commit using `mnem h --json`.
3. **Automated Restoration**: Revert to the last "Green" checkpoint if tests fail using `mnem r --checkpoint <name> --json`.

## Tools
- `mnem on --auto --json`: Start daemon without interactive prompts and get JSON confirmation.
- `mnem h --limit 1 --json`: Get the latest snapshot metadata for CI tagging.

## Rules
- Always use `--json` for pipeline parsing.
- Ensure the daemon is stopped with `mnem off --json` at the end of the CI job.
