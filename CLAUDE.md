# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

Indlovu — privacy-first vector database with built-in POPIA/GDPR compliance. Rust workspace, edition 2024, MIT licensed.

**Current state:** Early prototype / functional scaffold. Core vector search, compliance wrapper, audit logging, erasure certificates, and HTTP API all work. In-memory only (no persistence). Python bindings are a stub. 11 tests passing.

## Build & Test Commands

```bash
cargo build                          # Build all crates
cargo test --workspace               # Run all tests (11 tests)
cargo test -p indlovu-core           # Test a single crate
cargo test -p indlovu-core test_name # Run a single test
cargo clippy --workspace --all-targets -- -D warnings  # Lint (CI enforced)
cargo fmt --all -- --check           # Format check (CI enforced)
cargo run -p indlovu-server          # Run HTTP server on :6333
cargo run --example quickstart       # Run quickstart example
```

CI runs: fmt check → clippy → test → build examples (GitHub Actions).

## Architecture

Layered design where each layer wraps the one below:

```
indlovu-server (Axum HTTP API on :6333)
    ↓ uses
indlovu-compliance (CompliantStore wrapper, audit log, erasure certificates)
    ↓ wraps
indlovu-core (Collection with HNSW via usearch, metadata filtering)

indlovu-python (PyO3 bindings — stub only, not connected to core yet)
```

**Core pattern:** `CompliantStore<S: VectorStore + ErasureSupport>` wraps any store implementation to add audit logging and compliance tracking. All server-managed collections use `CompliantStore<Collection>` (aliased as `ManagedCollection` in `state.rs`).

**Key traits** (defined in `indlovu-core/src/traits.rs`):
- `VectorStore` — insert, insert_batch, search (with metadata Filter), get, delete, count, dimensions, distance
- `ErasureSupport` — erase_by_source, find_by_source (cascading deletion by source_document_id)
- `AuditLog` — log and query audit entries (AuditEntry with AuditAction enum)

**Server conventions** (`indlovu-server`):
- Routes in `routes.rs`, handlers in `handlers.rs`, shared state in `state.rs`
- All responses use `ApiResponse<T>` wrapper (`{ success, data, error }`)
- State is `AppState` with `Arc<RwLock<HashMap<String, Arc<RwLock<ManagedCollection>>>>>` for collections
- Also has conversion tracking (`ConversionEvent` / `ConversionSummary`) for landing page A/B experiment

**Metadata filtering** (`indlovu-core/src/metadata.rs`):
- `Filter` enum with variants: `Eq`, `Ne`, `Gt`, `Lt`, `In`, `And`, `Or`
- `Filter::matches(&self, metadata: &serde_json::Value) -> bool`
- Search over-fetches candidates (4x) when filtering to compensate for filtered-out results

**Erasure flow:**
1. `erase_by_source(source_document_id)` finds all records with matching source
2. Deletes each from both the HashMap and usearch index
3. `CompliantStore` wraps this to add audit logging
4. `execute_erasure()` function handles multi-collection erasure and produces `ErasureCertificate`
5. Certificate includes SHA-256 hash for tamper-evidence, with `verify()` method

**What works:**
- Vector insert/search/delete with usearch HNSW (cosine, euclidean, inner product)
- Metadata filtering on search results
- Compliance wrapper with full audit logging
- Right-to-erasure with cascading deletion and certificates
- HTTP API for all core operations
- Landing page with CTA A/B testing and conversion analytics

**What doesn't work yet:**
- No disk persistence (all in-memory, lost on restart)
- Python bindings are a stub (PyCollection has name/dimensions only, no vector ops)
- tantivy is a workspace dep but unused (no full-text or hybrid search)
- Retention policies are defined as structs but TTL enforcement is not implemented
- Audit log is in-memory only (no persistence, no chain-hashing)
- No DSAR endpoint, no list/get/delete collection endpoints

## Key Files

| File | Purpose |
|------|---------|
| `crates/indlovu-core/src/collection.rs` | Main `Collection` struct — HNSW index + record storage |
| `crates/indlovu-core/src/traits.rs` | Core traits: `VectorStore`, `ErasureSupport`, `AuditLog`, `AuditAction`, `AuditEntry` |
| `crates/indlovu-core/src/metadata.rs` | `Filter` enum for metadata-based search filtering |
| `crates/indlovu-core/src/types.rs` | `Vector`, `VectorRecord`, `SearchResult`, `Distance` |
| `crates/indlovu-compliance/src/compliant_store.rs` | `CompliantStore<S>` wrapper adding audit to any store |
| `crates/indlovu-compliance/src/erasure.rs` | `ErasureCertificate` and `execute_erasure()` function |
| `crates/indlovu-compliance/src/audit.rs` | `InMemoryAuditLog` implementation |
| `crates/indlovu-compliance/src/retention.rs` | `RetentionPolicy` and `ComplianceFramework` structs |
| `crates/indlovu-server/src/handlers.rs` | HTTP request handlers |
| `crates/indlovu-server/src/routes.rs` | Route definitions |
| `crates/indlovu-server/src/state.rs` | `AppState` with collections map + conversion tracking |
| `crates/indlovu-python/src/lib.rs` | PyO3 stub (Collection class only) |
| `examples/quickstart.rs` | Working demo of insert → search → erasure with audit trail |
| `site/index.html` | Landing page with A/B CTA experiment |
| `IMPLEMENTATION_PLAN.md` | Detailed phased plan for completing the project |

## Adding Features

New functionality typically touches multiple crates in order:
1. Core types/traits in `indlovu-core`
2. Compliance wrapper in `indlovu-compliance`
3. Server handler + route in `indlovu-server`
4. Python bindings in `indlovu-python`

Server handlers go in `handlers.rs` with serde request/response types; routes go in `routes.rs`.

## Dependencies

Key workspace dependencies:
- `usearch = "2"` — HNSW vector index (C++ with Rust bindings)
- `tantivy = "0.22"` — Full-text search (workspace dep, not yet used)
- `axum = "0.8"` — HTTP framework
- `pyo3 = "0.23"` — Python bindings
- `sha2 = "0.10"` — SHA-256 for erasure certificates (compliance crate only)
- `tokio`, `serde`, `uuid`, `chrono`, `thiserror`, `tracing` — standard Rust ecosystem

## Obsidian Vault Context

This project is tracked in the Obsidian vault at `/home/ameen/projects/`:
- **Operator card:** `_notes/indlovu.md` — project status and notes
- **Dashboard:** `_index.md` — overview of all projects
