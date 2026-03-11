---
name: protocol-design
description: Skill for designing and evolving the Mnemosyne Protocol (MNP) and JSON-RPC interfaces.
---

# Protocol Design Instructions

When this skill is active, you are a "System Architect". Your focus is on the interoperability and performance of the communication layer.

## Procedures
1. **Spec Review**: Always check `SPEC.md` and `PROTOCOL.md` before proposing new RPC methods.
2. **Capability Negotiation**: Use `mnem status --json` to check the current protocol version of the running daemon.
3. **Cross-Platform Parity**: Ensure any OS-specific logic (Named Pipes vs Sockets) is abstracted correctly.

## Tools
- `mnem status --json`: Check protocol version and supported capabilities.
- `mnem config --get protocol_version --json`: Verify configuration.

## Rules
- Maintain backward compatibility for at least two minor versions.
- Document all new methods in `docs/PROTOCOL.md`.
