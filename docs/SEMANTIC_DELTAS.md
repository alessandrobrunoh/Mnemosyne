# Semantic Deltas & AST-Based Tracking

## The Problem with File-Based History
Traditional local history tools (VSCode) and Version Control Systems (Git) treat code as lines of text. If you rename a function or move it, they see a deletion and an insertion. The logical link between the old and new version is lost.

## Mnemosyne's Solution: Semantic Deltas
Mnemosyne operates at the **AST (Abstract Syntax Tree)** level using Tree-sitter. It understands the logical structure of the code, not just the bytes.

### 1. Structural Identity (`structural_hash`)
Every logical entity (function, class, struct) is assigned a `structural_hash`. This hash is based on the logic of the node's body.
- **Refactor Resilience**: If a function name changes but the logic remains the same, the `structural_hash` remains stable.

### 2. The `SemanticDiffer` Engine
When a file is saved, the engine compares the previous snapshot's symbols with the current ones:
- **Modified**: Same name, different `structural_hash`.
- **Renamed**: Different name, same `structural_hash`.
- **Added/Deleted**: Entities appearing or disappearing from the AST.

### 3. Entity-Level History
Because we track symbols independently of the file snapshot, Mnemosyne can reconstruct the timeline of a *specific function* across renames and file moves. This is "Time Travel" for logic.

## Storage Optimization
Semantic Deltas allow for extreme deduplication. If a file of 10,000 lines has only one function modified, Mnemosyne only stores the new delta for that specific function. The rest of the file structure is reused via Content-Addressable Storage (CAS), achieving efficiency comparable to advanced binary delta systems (like IntelliJ) while providing semantic insights they lack.
