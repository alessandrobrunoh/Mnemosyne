# High-Performance System Architecture

Mnemosyne is built in Rust to provide a "zero-cost" background service that never interferes with the developer's typing flow. Our architecture is inspired by high-performance systems like Zed and Kafka.

## 1. Granular Concurrency (Lock-Free Philosophy)
We moved away from a "Global State Mutex" to a decentralized locking strategy:
- **DashMap**: We use concurrent hash maps for project and repository management, allowing multiple threads to read and write simultaneously.
- **Atomics**: Metrics and performance counters use `AtomicU64`, ensuring updates cost only a few CPU cycles with zero lock contention.
- **Fine-grained RwLocks**: Critical state like protocol initialization is protected by readers-writer locks from `parking_lot`, maximizing throughput for parallel RPC requests.

## 2. Zero-Copy I/O Stack
To minimize CPU and memory overhead during file saves, we implement a pure zero-copy pipeline:

### Memory-Mapped Files (`mmap`)
We use `memmap2` to map source files directly into the daemon's address space. 
- **Kernel-Level Efficiency**: The OS manages data loading via the Page Cache.
- **Zero Kernel-to-User Copy**: Data is accessed directly by the semantic engine without being copied into application buffers first.

### Shared Buffers (`bytes::Bytes`)
Once mapped, file contents are handled as `Bytes` objects.
- **Reference Counted**: Content is shared across the chunker, parser, and storage layers.
- **Allocation-Free Slicing**: Creating sub-chunks or extracting symbol text only creates a new "view" on the original buffer. No new allocations on the heap occur.

## 3. Hybrid Storage (SQLite + CAS)
We separate **Metadata** from **Content** to get the best of both worlds:
- **SQLite (Relational Index)**: Used for complex queries (time ranges, symbol relationships, delta chains). Optimized with Write-Ahead Logging (WAL).
- **Object Store (CAS)**: Deduplicated code chunks are stored on the filesystem, indexed by BLAKE3 hashes. This ensures that identical code blocks across the entire system are only stored once.
