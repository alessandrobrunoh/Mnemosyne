# Mnemosyne Protocol Specification (MNP) v1.0

## Status
This document describes version 1.0.0 of the Mnemosyne Protocol.

## 1. Overview
Mnemosyne is a protocol for managed local code history with semantic awareness. It allows clients (IDEs, TUIs, CLIs) to interact with a daemon that snapshots every file save, performs AST-based analysis, and manages a deduplicated, content-addressed storage of code evolution.

## 2. Base Protocol
The protocol is based on **JSON-RPC 2.0**.

### 2.1 Transport
- **Unix**: Unix Domain Sockets.
- **Windows**: Named Pipes.

Messages are newline-delimited (`\n`).

### 2.2 Envelope
Every request and response follows the JSON-RPC 2.0 structure. Additionally, restricted requests must include an `auth_token`.

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "mnem/project/list",
  "params": {},
  "auth_token": "..."
}
```

## 3. Lifecycle Management

### 3.1 `initialize`
Must be the first request sent by the client.

**Params:**
- `client_info`: `{ name: string, version?: string }`
- `capabilities`: `ClientCapabilities`
  - `semantic_analysis`: bool (client supports symbol tracking)
  - `git_integration`: bool (client supports git branch/commit context)
  - `progress_notifications`: bool (client handles async progress)

**Response:**
- `server_info`: `{ name: string, version: string }`
- `capabilities`: `ServerCapabilities`
  - `protocol_version`: string (e.g., "1.0.0")
  - `supported_methods`: string[]
  - `semantic_analysis`: bool
  - `git_integration`: bool
  - `supported_languages`: string[]
  - `max_batch_size`: number
- `protocol_version`: `string`

### 3.2 `shutdown`
Requests the server to shut down gracefully. The server stops accepting new requests.

### 3.3 `exit`
Requests the server process to terminate.

## 4. Capabilities

### Server Capabilities
- `semantic_analysis`: bool
- `git_integration`: bool
- `supported_languages`: string[]
- `supported_methods`: string[]

## 5. Standardized Methods

### Project Management
- `mnem/project/watch`: Monitor a project directory.
- `mnem/project/unwatch`: Stop monitoring a directory.
- `mnem/project/list`: List all watched projects.
- `mnem/project/activity`: Get recent activity across projects.
- `mnem/project/statistics`: Get storage and activity metrics.

### Snapshots
- `mnem/snapshot/create`: Manually trigger a snapshot.
- `mnem/snapshot/list`: List snapshot history for a file.
- `mnem/snapshot/get`: Retrieve content of a snapshot by hash.
- `mnem/snapshot/restore`: Restore a file to a specific version.

### Semantic Analysis
- `mnem/symbol/history`: Get a flat list of snapshots containing a symbol.
- `mnem/symbol/semantic_history`: Get the evolutionary timeline of a symbol using **Semantic Deltas**.
- `mnem/symbol/diff`: Get semantic diff between symbol versions.
- `mnem/symbol/search`: Find symbols by name or pattern.

## 6. Semantic Deltas & Structural Identity
Mnemosyne tracks logic, not just text.

### 6.1 `structural_hash`
Every symbol is assigned a hash derived from its AST subtree (excluding comments and whitespace). If `structural_hash` remains identical across versions, the logic is considered unchanged.

### 6.2 Delta Kinds
- `Added`: New symbol introduced.
- `Modified`: Symbol exists with same name but different `structural_hash`.
- `Deleted`: Symbol removed from file.
- `Renamed`: `structural_hash` matches a deleted symbol, but the name is different.

## 7. Error Codes

| Code | Message | Description |
|------|---------|-------------|
| -32700 | Parse error | Invalid JSON received. |
| -32601 | Method not found | The method does not exist. |
| -32100 | Server not initialized | `initialize` was not called. |
| -32101 | Already initialized | `initialize` called more than once. |
| -32102 | Unauthorized | Invalid or missing `auth_token`. |
| -32103 | Project not found | The specified project is not watched. |
| -32109 | Shutdown in progress | Request sent after `shutdown` call. |

## 8. Implementation Details

### 8.1 Zero-Copy Architecture
Implementations should aim for zero-copy data handling:
- Use **Memory Mapping (mmap)** for file reads.
- Use reference-counted buffers (e.g., Rust `bytes::Bytes`) to share content between parsing and storage layers.

### 8.2 Hybrid Storage
Metadata should be indexed in a relational manner (e.g., SQLite) while actual file contents should be stored in a **Content-Addressable Storage (CAS)** system using **BLAKE3** hashing for global deduplication.

## 9. Backward Compatibility
Servers should implement a normalization layer to support legacy methods (e.g., `project/watch` -> `mnem/project/watch`) to support older clients during the transition to v1.0.
