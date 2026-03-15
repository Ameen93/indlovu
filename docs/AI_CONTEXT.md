---
project: indlovu
type: ai-context
last_updated: 2026-03-15
---

# AI Context — Indlovu

> Quick-reference for AI assistants working on this codebase.

## What Is This?

A privacy-first vector database in Rust. Wraps usearch HNSW indexing with a compliance layer that adds audit logging, right-to-erasure, and erasure certificates. HTTP API via Axum. Python bindings via PyO3 (stub only).

## One-Line Summary

Rust vector DB with POPIA/GDPR compliance baked in — audit trails, cascading erasure by source document, SHA-256 erasure certificates.

## Status: Early Prototype

Working: vector search, metadata filtering, compliance wrapper, audit log, erasure certificates, HTTP API, CI.
Not working: persistence (all in-memory), Python bindings (stub), full-text search, hybrid search, retention enforcement.

## How to Build/Test

```bash
cargo build                    # Build
cargo test --workspace         # 11 tests, all passing
cargo run -p indlovu-server    # HTTP API on :6333
cargo run --example quickstart # Demo
```

## Architecture in 30 Seconds

```
Server (Axum :6333) → CompliantStore<Collection> → Collection (usearch HNSW)
```

- `Collection` = HashMap<Uuid, VectorRecord> + usearch HNSW index
- `CompliantStore<S>` = generic wrapper adding audit logging to any VectorStore
- `AppState` = Arc<RwLock<HashMap<String, Arc<RwLock<ManagedCollection>>>>>

## Core Traits

```rust
trait VectorStore: Send + Sync {
    fn insert(&mut self, record: VectorRecord) -> Result<Uuid>;
    fn search(&self, query: &Vector, top_k: usize, filter: Option<&Filter>) -> Result<Vec<SearchResult>>;
    fn get(&self, id: &Uuid) -> Result<Option<VectorRecord>>;
    fn delete(&mut self, id: &Uuid) -> Result<bool>;
    // + insert_batch, count, dimensions, distance
}

trait ErasureSupport: VectorStore {
    fn erase_by_source(&mut self, source_document_id: &str) -> Result<Vec<Uuid>>;
    fn find_by_source(&self, source_document_id: &str) -> Result<Vec<Uuid>>;
}
```

## Key Types

- `Vector` = `Vec<f32>`
- `VectorRecord` = id (Uuid) + vector + metadata (serde_json::Value) + source_document_id (Option) + created_at + contains_pii
- `Distance` = Cosine | Euclidean | InnerProduct
- `Filter` = Eq | Ne | Gt | Lt | In | And | Or
- `ErasureCertificate` = certificate_id + timestamp + subject_id + vectors_deleted + sha256_hash

## Where Things Live

| What | Where |
|------|-------|
| Core storage | `crates/indlovu-core/src/collection.rs` |
| Traits | `crates/indlovu-core/src/traits.rs` |
| Compliance wrapper | `crates/indlovu-compliance/src/compliant_store.rs` |
| Erasure certs | `crates/indlovu-compliance/src/erasure.rs` |
| HTTP handlers | `crates/indlovu-server/src/handlers.rs` |
| Routes | `crates/indlovu-server/src/routes.rs` |
| State | `crates/indlovu-server/src/state.rs` |
| Python stub | `crates/indlovu-python/src/lib.rs` |
| Implementation plan | `IMPLEMENTATION_PLAN.md` |

## Adding a Feature

1. Types/traits in `indlovu-core`
2. Compliance wrapper in `indlovu-compliance`
3. Handler + route in `indlovu-server`
4. Python binding in `indlovu-python`

## Known Gaps

- No persistence — data lost on restart
- Python bindings don't do anything yet
- tantivy dep unused (planned for BM25)
- Retention TTL not enforced
- Only 11 tests — needs much more coverage
- No list/get/delete collection API endpoints
- Audit log is in-memory only
