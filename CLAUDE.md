# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

Indlovu — privacy-first vector database with built-in POPIA/GDPR compliance. Rust workspace, edition 2024.

## Build & Test Commands

```bash
cargo build                          # Build all crates
cargo test --workspace               # Run all tests
cargo test -p indlovu-core           # Test a single crate
cargo test -p indlovu-core test_name # Run a single test
cargo clippy --workspace --all-targets -- -D warnings  # Lint (CI enforced)
cargo fmt --all -- --check           # Format check (CI enforced)
cargo run -p indlovu-server          # Run HTTP server on :6333
cargo run --example quickstart       # Run quickstart example
```

CI runs: fmt check → clippy → test → build examples.

## Architecture

Layered design where each layer wraps the one below:

```
indlovu-server (Axum HTTP API on :6333)
    ↓ uses
indlovu-compliance (CompliantStore wrapper, audit log, erasure certificates)
    ↓ wraps
indlovu-core (Collection with HNSW via usearch, metadata filtering)

indlovu-python (PyO3 bindings — separate, not layered)
```

**Core pattern:** `CompliantStore<S: VectorStore + ErasureSupport>` wraps any store implementation to add audit logging and compliance tracking. All server-managed collections use `CompliantStore<Collection>` (aliased as `ManagedCollection` in `state.rs`).

**Key traits** (defined in `indlovu-core/src/traits.rs`):
- `VectorStore` — insert, search, get, delete, count
- `ErasureSupport` — erase_by_source, find_by_source (cascading deletion by source_document_id)
- `AuditLog` — log and query audit events

**Server conventions** (`indlovu-server`):
- Routes in `routes.rs`, handlers in `handlers.rs`, shared state in `state.rs`
- All responses use `ApiResponse<T>` wrapper (success, data, error fields)
- State is `AppState` with `Arc<RwLock>` around collections map and audit log

**Erasure flow:** Right-to-erasure deletes all vectors linked to a `source_document_id` across collections and produces an `ErasureCertificate` with SHA-256 hash for tamper-evidence.

## Adding Features

New functionality typically touches multiple crates in order: core types/traits → compliance wrapper → server handler + route → python bindings. Server handlers go in `handlers.rs` with serde request/response types; routes go in `routes.rs`.

## Obsidian Vault Context

This project is tracked in the Obsidian vault at `/home/ameen/projects/`:
- **Project note:** `_notes/indlovu.md` — high-level status and notes
- **Dashboard:** `_index.md` — overview of all projects
