# Mnemosyne Protocol (MNP) v1.0 Standardization

## Overview
Mnemosyne is designed as a **Protocol-First** system, inspired by the success of the Language Server Protocol (LSP). By standardizing how code history is captured and queried, we decouple the "Semantic Memory" engine from specific IDEs or tools.

## Key Principles

### 1. JSON-RPC 2.0 Foundation
We chose JSON-RPC 2.0 over binary protocols (like gRPC) to prioritize **Interoperability** and **Debuggability**.
- **Low Barrier to Entry**: Clients can be implemented in any language (JS, Python, Lua, Go) in minutes.
- **Human Readable**: Developers can inspect the message flow through local sockets without specialized tools.

### 2. Lifecycle Management
MNP v1.0 introduces a formal lifecycle, ensuring robust state management between the daemon and clients:
- `initialize`: Negotiation of protocol version and server/client capabilities.
- `shutdown`: Graceful stop of the session.
- `exit`: Process termination.

### 3. Capability Negotiation
Instead of assuming features exist, clients and servers exchange "Capabilities" during initialization. This allows:
- Backward compatibility with older daemons.
- Graceful degradation if semantic analysis or git integration is unavailable.

### 4. Standardized Namespacing
All methods are prefixed with `mnem/` (e.g., `mnem/project/watch`) to prevent collisions with other protocols (LSP, MCP) that might share the same transport layer in a developer's environment.

## Backward Compatibility Layer
The reference implementation (mnem-daemon) includes a **Method Normalization Layer**. It transparently maps legacy method names (e.g., `project/watch`) to their v1.0 counterparts, ensuring existing extensions (like the Zed extension) continue to work while transitioning to the new standard.
