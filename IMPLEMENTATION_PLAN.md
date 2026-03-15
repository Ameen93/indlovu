# Indlovu — Implementation Plan

> Privacy-first vector database with built-in POPIA/GDPR compliance.
> Rust workspace. Scaffold is done, builds clean, zero tests. Time to make it real.

## Current State

The scaffold is in place and compiles:
- **4 crates:** `indlovu-core`, `indlovu-compliance`, `indlovu-server`, `indlovu-python`
- **Architecture:** `CompliantStore<Collection>` wraps core to add audit/erasure
- **Core:** In-memory `Collection` backed by `usearch` HNSW index, metadata filtering via `Filter` enum
- **Traits:** `VectorStore`, `ErasureSupport`, `AuditLog` — well-defined boundaries
- **Server:** Axum HTTP API on `:6333` with `ApiResponse<T>` wrapper
- **Python:** PyO3 bindings stub
- **CI:** GitHub Actions (fmt → clippy → test → build examples)
- **Tests:** Zero. Everything needs tests.
- **Persistence:** None. Everything is in-memory only.

---

## Phase 1: Make the Core Bulletproof (Priority: HIGH)

### 1.1 — Unit Tests for Core (Day 1-2)

Add comprehensive tests to `indlovu-core`. Cover:

```
tests to write in crates/indlovu-core/src/collection.rs (or tests/ dir):
- test_create_collection
- test_insert_and_get
- test_insert_dimension_mismatch_fails
- test_insert_batch
- test_search_top_k (insert 100 vectors, verify nearest neighbor correctness)
- test_search_with_cosine_distance
- test_search_with_euclidean_distance
- test_search_with_inner_product
- test_delete_record
- test_delete_nonexistent_returns_false
- test_count
- test_metadata_filter_eq
- test_metadata_filter_gt_lt
- test_metadata_filter_in
- test_metadata_filter_and_or_combination
- test_erase_by_source (insert 5 records with same source_document_id, erase, verify all gone)
- test_erase_by_source_nonexistent
- test_find_by_source
```

Also add tests for `metadata.rs` Filter evaluation logic separately.

### 1.2 — Disk Persistence (Day 2-4)

Currently everything lives in memory and is lost on restart. Implement:

1. **WAL (Write-Ahead Log)** for durability:
   - Append-only binary log of all mutations (insert, delete, erase)
   - On startup, replay WAL to reconstruct state
   - File location: `<data_dir>/<collection_name>/wal.bin`

2. **Snapshot/compaction:**
   - Periodically write full state to disk (records + usearch index via `index.save()`)
   - After snapshot, truncate WAL
   - File: `<data_dir>/<collection_name>/snapshot.bin` + `index.usearch`

3. **Collection metadata:**
   - Store collection config (name, dimensions, distance metric) in `<data_dir>/<collection_name>/meta.json`
   - Auto-discover collections on startup by scanning data_dir

4. **API changes:**
   - `Collection::new()` → `Collection::create()` (new) and `Collection::open()` (existing from disk)
   - Add `Collection::flush()` to force WAL → snapshot

Target: survive restarts, handle 1M+ vectors without OOM (memory-map the index).

### 1.3 — Hybrid Search with BM25 (Day 4-6)

Use `tantivy` for full-text/sparse search alongside HNSW dense vectors:

1. **Add a tantivy index** inside `Collection` for text fields in metadata
2. **New method:** `hybrid_search(query_vector, query_text, top_k, alpha, filter)` where `alpha` blends dense vs sparse scores
3. **Reciprocal Rank Fusion (RRF):** Combine HNSW and BM25 result lists: `score = Σ 1/(k + rank_i)`
4. **Store text content** as an optional field in `VectorRecord` (field: `text: Option<String>`)
5. **Tests:**
   - `test_hybrid_search_combines_results`
   - `test_hybrid_search_text_only` (no vector query)
   - `test_hybrid_search_vector_only` (no text query)
   - `test_rrf_scoring`

---

## Phase 2: Compliance Layer (Priority: HIGH — this is the differentiator)

### 2.1 — Audit Log Persistence (Day 6-7)

Current `AuditLog` trait exists but the implementation needs to be persistent:

1. **Append-only audit log file:** `<data_dir>/audit/audit.jsonl`
   - One JSON line per `AuditEntry`
   - Never deleted, never truncated (compliance requirement)
   - Include SHA-256 chain hash (each entry hashes previous entry's hash) for tamper evidence

2. **Query support:** Filter by collection, time range, action type, actor
3. **Retention policy config:** Auto-archive entries older than N days (move to `.gz`, don't delete)
4. **Tests:**
   - `test_audit_log_persists_across_restart`
   - `test_audit_chain_hash_integrity`
   - `test_audit_query_by_time_range`
   - `test_audit_query_by_collection`

### 2.2 — PII Tagging & Retention (Day 7-8)

1. **Collection-level PII flag:** `Collection::set_pii(true)` marks a collection as containing personal data
2. **Retention TTL:** `Collection::set_retention(Duration)` — auto-expire records after TTL
   - Background task (or on-access check) deletes expired records
   - Deletion goes through audit log
3. **PII report endpoint:** List all PII-flagged collections with record counts and retention status
4. **Data Subject Access Request (DSAR):** `GET /collections/{name}/dsar?source_document_id=X` returns all data linked to a person
5. **Tests:**
   - `test_retention_ttl_expires_records`
   - `test_pii_flagging`
   - `test_dsar_returns_all_records_for_source`

### 2.3 — Erasure Certificates (Day 8-9)

The `ErasureCertificate` struct exists. Make it robust:

1. **Certificate storage:** Write to `<data_dir>/erasure/certificates.jsonl`
2. **Include:** timestamp, source_document_id, list of deleted record IDs, SHA-256 hash of deleted data, operator/actor
3. **Verification endpoint:** `GET /compliance/erasure/{certificate_id}/verify` — proves the data was deleted
4. **POPIA mode flag:** `--popia` startup flag that:
   - Requires `source_document_id` on all inserts (reject without it)
   - Enables mandatory audit logging (can't disable)
   - Forces PII tagging on all collections by default
   - Enables retention policy enforcement
5. **Tests:**
   - `test_erasure_produces_certificate`
   - `test_erasure_certificate_verifiable`
   - `test_popia_mode_rejects_insert_without_source`

---

## Phase 3: Server & Python DX (Priority: MEDIUM)

### 3.1 — Complete HTTP API (Day 9-11)

Fill out the Axum server with all endpoints:

```
Collections:
  POST   /collections                          — create collection
  GET    /collections                          — list collections
  GET    /collections/{name}                   — collection info (count, dimensions, pii, retention)
  DELETE /collections/{name}                   — drop collection

Records:
  POST   /collections/{name}/records           — insert (single or batch)
  GET    /collections/{name}/records/{id}      — get by ID
  DELETE /collections/{name}/records/{id}      — delete by ID

Search:
  POST   /collections/{name}/search            — vector search (top_k, filter)
  POST   /collections/{name}/hybrid-search     — hybrid search (vector + text, alpha, filter)

Compliance:
  POST   /compliance/erase                     — right-to-erasure by source_document_id
  GET    /compliance/erasure-certificates       — list certificates
  GET    /compliance/erasure/{id}/verify        — verify certificate
  GET    /compliance/audit                     — query audit log (collection, from, to, action)
  GET    /compliance/dsar/{source_document_id}  — data subject access request
  GET    /compliance/pii-report                — list PII-flagged collections

Health:
  GET    /health                               — basic health check
  GET    /info                                 — version, uptime, collection count
```

All endpoints return `ApiResponse<T>`. Use proper HTTP status codes.

### 3.2 — Python Bindings (Day 11-13)

Make the PyO3 bindings actually useful. Target API:

```python
from indlovu import Indlovu, Collection

# Embedded mode (no server needed)
db = Indlovu(path="./my_data", popia_mode=True)

# Create collection
col = db.create_collection("documents", dimensions=384, distance="cosine")
col.set_pii(True)
col.set_retention(days=365)

# Insert
col.insert(
    vector=[0.1, 0.2, ...],
    metadata={"category": "legal", "author": "Ameen"},
    source_document_id="doc-123",  # required in POPIA mode
    text="The contract states that..."  # optional, enables hybrid search
)

# Batch insert
col.insert_batch(records=[...])

# Search
results = col.search(vector=[0.1, ...], top_k=10, filter={"category": {"$eq": "legal"}})
results = col.hybrid_search(vector=[0.1, ...], text="contract terms", top_k=10, alpha=0.7)

# Compliance
erased = db.erase(source_document_id="doc-123")  # returns ErasureCertificate
report = db.pii_report()
audit = db.audit_log(collection="documents", last_hours=24)
dsar = db.dsar(source_document_id="doc-123")

# Cleanup
col.flush()
db.close()
```

Publish-ready: `maturin develop` for local testing, `maturin build --release` for wheels.

### 3.3 — Built-in Embedding (Day 13-14) [STRETCH]

Optional: add ONNX Runtime embedding so users don't need to bring their own:

```python
col = db.create_collection("docs", model="all-MiniLM-L6-v2")  # auto-sets dimensions=384
col.insert(text="Hello world")  # auto-embeds, no vector needed
results = col.search(text="greeting")  # auto-embeds query
```

Use `ort` crate (ONNX Runtime Rust bindings). Ship `all-MiniLM-L6-v2` as default model (~80MB).
This is a stretch goal — skip if time is tight.

---

## Phase 4: Polish & Launch (Priority: HIGH for portfolio impact)

### 4.1 — Benchmarks (Day 14-15)

Create `benches/` directory with criterion benchmarks:

1. **Insert throughput:** 1K, 10K, 100K, 1M vectors (128d, 384d, 768d)
2. **Search latency:** p50, p95, p99 at various dataset sizes
3. **Hybrid search overhead:** vs pure vector search
4. **Memory footprint:** RSS at 100K, 500K, 1M vectors
5. **Compare vs Chroma** (Python, same hardware): insert + search + memory
6. **Target hardware:** 1-core, 2GB RAM (the SA/Africa use case)

Publish results in README with a table and chart.

### 4.2 — Documentation (Day 15-16)

1. **README.md** — already good, enhance with:
   - Architecture diagram (Mermaid)
   - Benchmark results table
   - "Why Indlovu?" section (compliance angle)
   - Quickstart for Python AND Rust
   - POPIA compliance section explaining what it does and doesn't guarantee

2. **docs/** site (MkDocs Material):
   - Getting Started
   - Python API Reference
   - Rust API Reference
   - HTTP API Reference
   - Compliance Guide (POPIA, GDPR, how to use erasure/audit/DSAR)
   - Architecture & Design Decisions
   - Benchmarks
   - Contributing Guide

### 4.3 — Launch Checklist (Day 16-17)

- [ ] All tests pass, clippy clean, fmt clean
- [ ] GitHub repo public with proper description, topics, social preview image
- [ ] PyPI package published (`pip install indlovu`)
- [ ] crates.io published (`cargo add indlovu-core`)
- [ ] Docker image on GHCR (`docker run ghcr.io/ameen/indlovu`)
- [ ] Blog post: "Building a POPIA-Compliant Vector Database in Rust"
- [ ] Post to: Hacker News, r/rust, r/MachineLearning, r/southafrica, SA tech Slack/Discord communities, Twitter/X
- [ ] LangChain integration PR (stretch)

---

## Architecture Reference

```
┌─────────────────────────────────────────────────┐
│                  HTTP API (Axum)                 │
│              indlovu-server :6333                │
├─────────────────────────────────────────────────┤
│           Compliance Layer                       │
│    CompliantStore<Collection>                     │
│    ┌──────────┬──────────┬───────────┐          │
│    │ AuditLog │ Erasure  │ Retention │          │
│    │ (chain-  │ Certifi- │ TTL +     │          │
│    │  hashed) │ cates    │ PII tags  │          │
│    └──────────┴──────────┴───────────┘          │
├─────────────────────────────────────────────────┤
│              Core Engine                         │
│    Collection                                    │
│    ┌──────────────┬──────────────┐              │
│    │ Dense Search  │ Sparse Search│              │
│    │ usearch HNSW  │ tantivy BM25 │              │
│    └──────────────┴──────────────┘              │
│    ┌──────────────┬──────────────┐              │
│    │ Metadata     │ Persistence  │              │
│    │ Filtering    │ WAL+Snapshot │              │
│    └──────────────┴──────────────┘              │
├─────────────────────────────────────────────────┤
│           Python Bindings (PyO3)                 │
│         pip install indlovu                      │
└─────────────────────────────────────────────────┘
```

## Key Design Principles

1. **Compliance is not optional.** Audit logging is always on. Erasure always works. POPIA mode makes it stricter, not different.
2. **Embedded-first.** No server needed for basic use. Import the library, open a path, go.
3. **SQLite-level simplicity.** Single data directory, no config files, no Docker required.
4. **Honest about scope.** "Designed to support POPIA compliance" — not "POPIA certified." Legal review is the user's responsibility.
5. **Portfolio quality.** Every commit, every doc, every API name should look like it came from a well-run open-source project.

## File Structure Target

```
indlovu/
├── Cargo.toml                    # Workspace root
├── CLAUDE.md                     # AI coding context
├── README.md                     # The showpiece
├── LICENSE                       # MIT
├── IMPLEMENTATION_PLAN.md        # This file
├── crates/
│   ├── indlovu-core/
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── collection.rs     # Main Collection impl
│   │   │   ├── persistence.rs    # WAL + snapshot (NEW)
│   │   │   ├── hybrid.rs         # BM25 + RRF (NEW)
│   │   │   ├── metadata.rs       # Filter evaluation
│   │   │   ├── traits.rs         # VectorStore, ErasureSupport, AuditLog
│   │   │   ├── types.rs          # Vector, VectorRecord, SearchResult
│   │   │   └── error.rs
│   │   └── tests/                # Integration tests (NEW)
│   ├── indlovu-compliance/
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── compliant_store.rs
│   │   │   ├── audit.rs          # Persistent, chain-hashed audit log
│   │   │   ├── erasure.rs        # Certificates + verification
│   │   │   ├── retention.rs      # TTL + PII tagging
│   │   │   └── dsar.rs           # Data Subject Access Request (NEW)
│   │   └── tests/
│   ├── indlovu-server/
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── lib.rs
│   │   │   ├── routes.rs
│   │   │   ├── handlers.rs
│   │   │   └── state.rs
│   │   └── tests/                # HTTP integration tests (NEW)
│   └── indlovu-python/
│       ├── src/lib.rs
│       ├── Cargo.toml
│       └── pyproject.toml        # maturin config (NEW)
├── benches/                      # Criterion benchmarks (NEW)
├── examples/
│   ├── quickstart.rs
│   ├── popia_mode.rs             # (NEW)
│   └── hybrid_search.rs          # (NEW)
├── docs/                         # MkDocs site (NEW)
└── docker/
    ├── Dockerfile                # (NEW)
    └── docker-compose.yml        # (NEW)
```

---

*Hand this file to Claude Code. It has full context on the existing scaffold (CLAUDE.md), traits, architecture, and what's already built. Start with Phase 1.1 (tests) and work forward sequentially.*
