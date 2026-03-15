<p align="center">
  <h1 align="center">🐘 Indlovu</h1>
  <p align="center"><strong>Privacy-first vector database with built-in POPIA/GDPR compliance</strong></p>
  <p align="center">
    <a href="#quick-start">Quick Start</a> •
    <a href="#features">Features</a> •
    <a href="#compliance">Compliance</a> •
    <a href="#architecture">Architecture</a> •
    <a href="#current-status">Status</a> •
    <a href="#roadmap">Roadmap</a>
  </p>
  <p align="center">
    <img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License: MIT">
    <img src="https://img.shields.io/badge/rust-2024_edition-orange.svg" alt="Rust 2024">
    <img src="https://img.shields.io/badge/POPIA-designed_for-green.svg" alt="POPIA Designed-For">
    <img src="https://img.shields.io/badge/GDPR-designed_for-green.svg" alt="GDPR Designed-For">
  </p>
</p>

---

**Indlovu** (isiZulu for *elephant*) is a vector database that treats privacy as a first-class feature, not an afterthought. Every insert, search, and deletion is audit-logged. Every vector can be traced back to its source document. And when someone exercises their right to erasure, the elephant forgets — completely, verifiably, and with a cryptographic receipt.

> **Note:** Indlovu is designed to *support* POPIA/GDPR compliance at the database layer. It is not a certified compliance solution — legal review is the user's responsibility.

## Current Status

**Early prototype / functional scaffold.** The core architecture is implemented and working:

- ✅ In-memory HNSW vector search (usearch) — working
- ✅ Metadata filtering (eq, ne, gt, lt, in, and, or) — working
- ✅ Compliance wrapper with audit logging — working
- ✅ Right-to-erasure with cascading deletion — working
- ✅ SHA-256 tamper-evident erasure certificates — working
- ✅ HTTP API server (Axum on :6333) — working
- ✅ Landing page with CTA A/B testing — working
- ✅ Quickstart example — working
- ✅ CI pipeline (fmt + clippy + test) — working
- ⚠️ Python bindings — **stub only** (creates Collection object but no actual vector operations)
- ❌ Disk persistence — **not implemented** (all data is in-memory, lost on restart)
- ❌ Full-text/BM25 search — **not implemented** (tantivy is a workspace dependency but unused)
- ❌ Hybrid search — **not implemented**
- ❌ Retention policy enforcement — **defined but not enforced** (TTL expiry not implemented)
- ❌ Benchmarks — **not yet run**

**Tests:** 11 tests across the workspace, all passing. Coverage is light — the implementation plan calls for comprehensive test additions.

**Build:** Compiles clean on Rust 2024 edition. Clippy clean (one minor warning).

## Features

### Implemented

- 🔍 **HNSW Vector Search** — Sub-millisecond approximate nearest neighbor search via [usearch](https://github.com/unum-cloud/usearch). Supports cosine, euclidean, and inner product distance metrics.
- 🛡️ **Right-to-Erasure** — Cascading deletion of all vectors linked to a `source_document_id`, with cryptographic erasure certificates (SHA-256 hashed) for audit-proof compliance.
- 📋 **Audit Trails** — In-memory append-only log of every insert, search, deletion, and erasure event. Queryable by collection and time range.
- 🏷️ **PII Tagging** — Mark individual vector records as containing personal information. Per-collection PII flagging.
- 🔧 **Metadata Filtering** — Rich query filters on vector metadata: `Eq`, `Ne`, `Gt`, `Lt`, `In`, `And`, `Or`.
- ⏰ **Retention Policies** — Configurable per-collection retention policy structs (POPIA/GDPR presets) — *defined but not yet enforced at runtime*.
- 🌐 **HTTP API** — RESTful server on port 6333 with collection management, vector insert/search, erasure, and conversion analytics endpoints.
- 📊 **Landing Page A/B Testing** — Built-in CTA experiment with variant assignment and conversion tracking.

### Planned (Not Yet Implemented)

- 📝 **Full-Text Search** — BM25 keyword search via [tantivy](https://github.com/quickwit-oss/tantivy)
- 🔄 **Hybrid Search** — Combined vector + keyword search with Reciprocal Rank Fusion
- 💾 **Disk Persistence** — WAL + snapshot for surviving restarts
- 🐍 **Python Bindings** — Full PyO3 API (currently stub only)
- 🔐 **Encryption at rest**
- 👥 **Multi-tenancy**

## Architecture

```
┌─────────────────────────────────────────────────────┐
│                   Applications                       │
│         Python SDK  │  HTTP API  │  Rust API         │
└────────┬────────────┴─────┬──────┴──────┬────────────┘
         │                  │             │
┌────────▼──────────────────▼─────────────▼────────────┐
│              indlovu-server (Axum HTTP on :6333)      │
│         Routes · Handlers · State Management          │
│         CTA A/B experiment · Conversion analytics     │
└────────────────────────┬─────────────────────────────┘
                         │
┌────────────────────────▼─────────────────────────────┐
│           indlovu-compliance (Privacy Layer)          │
│  CompliantStore<S> · InMemoryAuditLog · RetentionPolicy│
│       ErasureCertificate (SHA-256) · PII tracking    │
└────────────────────────┬─────────────────────────────┘
                         │
┌────────────────────────▼─────────────────────────────┐
│              indlovu-core (Storage Engine)            │
│    Collection · HNSW Index (usearch) · Metadata      │
│    Filter enum · VectorStore trait · ErasureSupport   │
└──────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────┐
│            indlovu-python (PyO3 Bindings)            │
│          Stub — Collection class only (WIP)          │
└──────────────────────────────────────────────────────┘
```

### Crate Structure

| Crate | Description | Status |
|-------|-------------|--------|
| `indlovu-core` | Vector storage engine — HNSW indexing via usearch, metadata filtering, core traits (`VectorStore`, `ErasureSupport`, `AuditLog`) | ✅ Working |
| `indlovu-compliance` | Privacy layer — `CompliantStore` wrapper, in-memory audit logging, erasure certificates, retention policy structs | ✅ Working |
| `indlovu-server` | HTTP API — Axum routes/handlers, shared state with `Arc<RwLock>`, conversion analytics | ✅ Working |
| `indlovu-python` | Python bindings — PyO3 `Collection` class stub (name + dimensions only, no vector ops) | ⚠️ Stub |

## Quick Start

### Rust (Library)

```rust
use indlovu_compliance::{CompliantStore, InMemoryAuditLog, RetentionPolicy};
use indlovu_core::traits::{ErasureSupport, VectorStore};
use indlovu_core::{Collection, Distance, VectorRecord};
use serde_json::json;
use std::time::Duration;

fn main() {
    // Create a POPIA-compliant collection
    let collection = Collection::new("customers", 384, Distance::Cosine).unwrap();
    let audit = InMemoryAuditLog::new();
    let policy = RetentionPolicy::popia(Duration::from_secs(90 * 86400));
    let mut store = CompliantStore::new(collection, audit, "customers".into(), policy);

    // Insert a PII-tagged vector with source tracking
    let record = VectorRecord::new(
        vec![0.1; 384],
        json!({"name": "Sipho", "type": "embedding"}),
        Some("user-sipho-001".into()),
        true, // contains PII
    );
    let id = store.insert(record).unwrap();
    println!("Inserted: {id}");

    // Search
    let results = store.search(&vec![0.1; 384], 5, None).unwrap();
    println!("Found {} results", results.len());

    // Right-to-erasure: delete all vectors from a source
    let erased = store.erase_by_source("user-sipho-001").unwrap();
    println!("Erased {} vectors", erased.len());
}
```

### HTTP API

```bash
# Start the server
cargo run -p indlovu-server

# Create a collection
curl -X POST http://localhost:6333/collections \
  -H "Content-Type: application/json" \
  -d '{"name": "docs", "dimensions": 384, "contains_pii": true}'

# Insert a vector
curl -X POST http://localhost:6333/collections/docs/vectors \
  -H "Content-Type: application/json" \
  -d '{"vector": [0.1, 0.2, ...], "metadata": {"source": "doc1"}, "source_document_id": "doc1"}'

# Search
curl -X POST http://localhost:6333/collections/docs/search \
  -H "Content-Type: application/json" \
  -d '{"vector": [0.1, 0.2, ...], "top_k": 5}'

# Right-to-erasure
curl -X POST http://localhost:6333/collections/docs/erase \
  -H "Content-Type: application/json" \
  -d '{"source_document_id": "doc1"}'
```

### HTTP API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/health` | Health check (returns version) |
| `POST` | `/collections` | Create a collection |
| `POST` | `/collections/{name}/vectors` | Insert a vector |
| `POST` | `/collections/{name}/search` | Search vectors (top_k) |
| `POST` | `/collections/{name}/erase` | Right-to-erasure by source_document_id |
| `POST` | `/analytics/conversion` | Record CTA conversion event |
| `GET` | `/analytics/conversions` | Get conversion summary |

### Landing Page A/B Experiment

The landing page (`site/index.html`) includes a CTA copy experiment:

- Variants `a` and `b` with different CTA text
- Assignment via `?cta_variant=a` query param or random with localStorage persistence
- Clicks tracked via `POST /analytics/conversion`
- Results via `GET /analytics/conversions`

## Compliance

Indlovu is designed for organizations operating under **POPIA** (South Africa's Protection of Personal Information Act) and **GDPR** (EU General Data Protection Regulation).

### Right-to-Erasure

When a data subject requests deletion:

1. **Cascading deletion** — All vectors linked to a `source_document_id` are removed
2. **Erasure certificate** — A JSON certificate is generated with SHA-256 hash for tamper-evidence
3. **Audit trail** — The erasure event is logged in the append-only audit log

```json
{
  "certificate_id": "a1b2c3d4-...",
  "timestamp": "2026-02-27T06:52:00Z",
  "subject_id": "user-sipho-001",
  "vectors_deleted": ["uuid-1", "uuid-2"],
  "vectors_deleted_count": 2,
  "collections_affected": ["customers"],
  "initiated_by": "compliance-officer",
  "sha256_hash": "e3b0c44298fc1c149afb..."
}
```

Certificates include a `verify()` method that recomputes the hash to detect tampering.

### PII Tagging

Every `VectorRecord` has a `contains_pii: bool` field. Collections can enforce PII-aware retention policies.

### Retention Policies

Per-collection retention policies with POPIA/GDPR presets:

```rust
let policy = RetentionPolicy::popia(Duration::from_secs(90 * 86400));  // 90 days
let policy = RetentionPolicy::gdpr(Duration::from_secs(365 * 86400));  // 1 year
```

> **Note:** Retention policy structs are defined but automatic TTL enforcement is not yet implemented.

### Audit Trail

Every operation is logged with timestamp (UTC), collection name, action type, and actor identity. The current implementation is in-memory only.

## Building from Source

```bash
git clone https://github.com/ameen/indlovu.git
cd indlovu
cargo build --release

# Run the quickstart example
cargo run --example quickstart

# Run the server
cargo run -p indlovu-server --release

# Run tests
cargo test --workspace

# Lint
cargo clippy --workspace --all-targets -- -D warnings
```

### Requirements

- Rust 2024 edition (1.85+)
- C++ compiler (for usearch native bindings)

## Roadmap

- [x] HNSW vector indexing (usearch) with cosine/euclidean/inner-product
- [x] Metadata filtering (eq, ne, gt, lt, in, and, or)
- [x] Audit trail logging (in-memory)
- [x] PII tagging on records
- [x] Right-to-erasure with cascading deletion
- [x] SHA-256 erasure certificates with tamper detection
- [x] HTTP API (Axum on :6333)
- [x] CI pipeline (GitHub Actions)
- [x] Quickstart example
- [x] Landing page with A/B experiment
- [ ] Comprehensive test coverage
- [ ] Disk persistence (WAL + snapshots)
- [ ] BM25 full-text search (tantivy)
- [ ] Hybrid search (vector + keyword + RRF)
- [ ] Retention TTL enforcement
- [ ] Persistent audit log (chain-hashed JSONL)
- [ ] Complete Python bindings (PyO3)
- [ ] DSAR endpoint (Data Subject Access Request)
- [ ] Benchmarks
- [ ] Docker image
- [ ] Multi-tenancy
- [ ] Encryption at rest
- [ ] RBAC

## Contributing

Contributions are welcome! Please open an issue or PR. For major changes, open a discussion first.

## License

MIT — see [LICENSE](LICENSE) for details.

---

<p align="center">
  <strong>🐘 The elephant never forgets — unless you ask it to.</strong>
  <br>
  <em>Built in South Africa 🇿🇦</em>
</p>
