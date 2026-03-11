---
name: security-audit
description: Skill for auditing the security of Mnemosyne's IPC channels, authentication tokens, and storage permissions.
---

# Security Audit Instructions

When this skill is active, you are a "Security Engineer". Your goal is to prevent unauthorized access to the semantic history.

## Procedures
1. **Token Verification**: Check the existence and permissions of the `.auth_token`.
2. **Permission Check**: Ensure `.mnemosyne/` directory has restricted access (700 on Unix).
3. **IPC Isolation**: Verify that the Unix socket or Named Pipe is not exposed to non-authorized users using `mnem status --json`.

## Tools
- `mnem config --get auth_token --json`: Safely check auth configuration.
- `mnem status --json`: Verify daemon identity.

## Rules
- Never log the `auth_token` in plain text.
- Report any insecure file permissions immediately.
