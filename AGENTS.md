# Mnemosyne Development Guidelines for Agents & Humans

This project aims to establish itself as a standard, similar to ACP (Agent Client Protocol) or LSP (Language Server Protocol), created by Zed Industries, the developers behind the Zed IDE.

The goal is to build a TUI (Text-based User Interface), CLI (Command Line Interface), and MCP (Model Context Platform e Model Context Server) on top of this foundation.

## Core Philosophy

1. **The Standard is Sovereign**: Every modification must adhere to the **Mnemosyne Protocol (MNP)**. If a feature requires a new RPC method, it must be documented in `SPEC.md` first.
2. **Semantic Identity over Textual Diffs**: We don't just track lines; we track **logic**. Always leverage Tree-sitter ASTs and `structural_hash` to maintain continuity across renames and refactors.
3. **High-Performance "Zed-style" Engineering**:
   - **Zero-Copy**: Use `bytes::Bytes` and `mmap` for data handling. Never copy a buffer if you can share a reference.
   - **Granular Concurrency**: Avoid global locks. Use concurrent collections (`DashMap`) and atomics to ensure the daemon never blocks the developer's flow.
   - **Non-blocking IO**: All long-running operations (parsing, storage) should happen in background tasks or threads.

## Implementation Rules

- **Strict Pathing**: Always use absolute paths for file system operations.
- **Error Handling**: No `unwrap()` or `expect()` in the daemon. Use `Result` and propagate errors using `AppError`.
- **Database**: SQLite is used as a relational index for metadata, while the filesystem is the source of truth for blobs (Content-Addressable Storage).
- **Tooling**: Ensure any new CLI command follows the `Layout` UI patterns for consistency.

## Vision for Future Contributors

- **Connection Pooling**: Transition from `Mutex<Connection>` to an authenticated connection pool (e.g., `r2d2-sqlite`) to allow massive parallel reads.
- **Incremental Parsing**: Implement Tree-sitter incremental re-parsing to handle massive files with microsecond latency.
- **String Interning**: Reduce memory footprint by interning common symbols and paths.
- **Semantic Patching**: Store actual data as semantic deltas (patches) instead of raw chunks when logic is similar.
- **Platform Parity**: Always maintain full feature parity between Windows (Named Pipes) and Unix-like systems.

