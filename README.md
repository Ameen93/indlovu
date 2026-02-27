<p align="center">
  <h1 align="center">🐘 Indlovu</h1>
  <p align="center"><strong>Privacy-first vector database with built-in POPIA/GDPR compliance</strong></p>
  <p align="center">
    <a href="#quick-start">Quick Start</a> •
    <a href="#features">Features</a> •
    <a href="#compliance">Compliance</a> •
    <a href="#architecture">Architecture</a> •
    <a href="#benchmarks">Benchmarks</a> •
    <a href="#roadmap">Roadmap</a>
  </p>
  <p align="center">
    <img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License: MIT">
    <img src="https://img.shields.io/badge/rust-2024_edition-orange.svg" alt="Rust 2024">
    <img src="https://img.shields.io/badge/POPIA-compliant-green.svg" alt="POPIA Compliant">
    <img src="https://img.shields.io/badge/GDPR-compliant-green.svg" alt="GDPR Compliant">
  </p>
</p>

---

**Indlovu** (isiZulu for *elephant*) is a vector database that treats privacy as a first-class feature, not an afterthought. Every insert, search, and deletion is audit-logged. Every vector can be traced back to its source document. And when someone exercises their right to erasure, the elephant forgets — completely, verifiably, and with a cryptographic receipt.

## Features

- 🔍 **HNSW Vector Search** — Sub-millisecond approximate nearest neighbor search via [usearch](https://github.com/unum-cloud/usearch)
- 📝 **Full-Text Search** — BM25 keyword search via [tantivy](https://github.com/quickwit-oss/tantivy) *(planned)*
- 🛡️ **Right-to-Erasure** — Cascading deletion with audit-proof erasure certificates (SHA-256 signed)
- 📋 **Audit Trails** — Append-only log of every data operation, queryable by collection and time range
- 🏷️ **PII Tagging** — Mark vectors containing personal data; enforce stricter retention policies automatically
- ⏰ **Retention Policies** — Configurable per-collection data lifecycle (POPIA: purpose limitation, GDPR: storage limitation)
- 🐍 **Python Bindings** — Native Python API via PyO3; `pip install indlovu`
- 🌐 **HTTP API** — RESTful server with Axum; deploy anywhere
- 🔧 **Metadata Filtering** — Rich query filters (eq, gt, lt, in, and/or) on vector metadata

## Architecture

```
┌─────────────────────────────────────────────────────┐
│                   Applications                       │
│         Python SDK  │  HTTP API  │  Rust API         │
└────────┬────────────┴─────┬──────┴──────┬────────────┘
         │                  │             │
┌────────▼──────────────────▼─────────────▼────────────┐
│              indlovu-server (Axum HTTP)               │
│         Routes · Handlers · State Management          │
└────────────────────────┬─────────────────────────────┘
                         │
┌────────────────────────▼─────────────────────────────┐
│           indlovu-compliance (Privacy Layer)          │
│  CompliantStore · AuditLog · RetentionPolicy · PII   │
│         Right-to-Erasure · Erasure Certificates      │
└────────────────────────┬─────────────────────────────┘
                         │
┌────────────────────────▼─────────────────────────────┐
│              indlovu-core (Storage Engine)            │
│    Collection · HNSW Index (usearch) · Metadata      │
│      VectorStore trait · ErasureSupport trait         │
└──────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────┐
│            indlovu-python (PyO3 Bindings)            │
│              Native Python ↔ Rust bridge             │
└──────────────────────────────────────────────────────┘
```

### Crate Structure

| Crate | Description |
|-------|-------------|
| `indlovu-core` | Vector storage engine — HNSW indexing, metadata filtering, core traits |
| `indlovu-compliance` | Privacy layer — audit logging, PII tracking, erasure certificates, retention |
| `indlovu-server` | HTTP API — Axum routes, handlers, shared state |
| `indlovu-python` | Python bindings — PyO3 wrapper for the full stack |

## Quick Start

### Rust

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

### Python

```python
from indlovu import Collection

# Create a collection
db = Collection("my_docs", dimensions=384)

# Insert vectors with metadata
db.add(
    vectors=[[0.1, 0.2, ...] * 384],
    metadata=[{"source": "doc-001", "contains_pii": True}],
    source_document_id="doc-001"
)

# Search
results = db.search(query=[0.1, 0.2, ...], top_k=5)

# Right-to-erasure (POPIA/GDPR)
db.erase(source_document_id="doc-001")
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

### Landing Page A/B CTA Experiment

The landing page (`site/index.html`) now includes a simple CTA copy experiment with two variants (`a` and `b`).

- Variant assignment is feature-flagged via `cta_variant` query param (`?cta_variant=a` or `?cta_variant=b`).
- If no query param is provided, a random variant is assigned and persisted in `localStorage`.
- CTA clicks are sent to `POST /analytics/conversion` with `variant` and `cta_id`.

#### Read experiment results

```bash
# Summary of total conversions + breakdown by variant and CTA location
curl http://localhost:6333/analytics/conversions
```

Example response:

```json
{
  "success": true,
  "data": {
    "total_events": 12,
    "by_variant": {"a": 5, "b": 7},
    "by_cta": {"hero-primary-cta": 8, "footer-primary-cta": 4}
  },
  "error": null
}
```

## Compliance

Indlovu is designed for organizations operating under **POPIA** (South Africa's Protection of Personal Information Act) and **GDPR** (EU General Data Protection Regulation).

### Right-to-Erasure

When a data subject requests deletion:

1. **Cascading deletion** — All vectors linked to a `source_document_id` are removed across collections
2. **Erasure certificate** — A JSON certificate is generated with:
   - Timestamp of erasure
   - Subject/source identifier
   - List of deleted vector IDs
   - SHA-256 hash of the certificate for tamper-evidence
3. **Audit trail** — The erasure event is logged in the append-only audit log

```json
{
  "certificate_id": "a1b2c3d4-...",
  "timestamp": "2025-02-27T06:52:00Z",
  "subject_id": "user-sipho-001",
  "vectors_deleted": ["uuid-1", "uuid-2"],
  "collections_affected": ["customers"],
  "sha256_hash": "e3b0c44298fc1c149afb..."
}
```

### PII Tagging

Every vector record can be flagged as `contains_pii: true`. Collections with PII-tagged records automatically get:
- Stricter retention enforcement
- Enhanced audit logging
- Priority processing for erasure requests

### Retention Policies

Configure per-collection retention policies aligned with legal requirements:

```rust
// POPIA: 90-day retention for support data
let policy = RetentionPolicy::popia(Duration::from_secs(90 * 86400));

// GDPR: 365-day retention for analytics
let policy = RetentionPolicy::gdpr(Duration::from_secs(365 * 86400));
```

### Audit Trail

Every operation is logged with:
- **Timestamp** (UTC)
- **Collection** name
- **Action** (insert, delete, search, erase, collection created/dropped)
- **Actor** (user or service identity)

Audit logs are append-only and queryable by collection and time range.

## Benchmarks

> ⚠️ Benchmarks are preliminary. Run on your own hardware for accurate numbers.

| Operation | Vectors | Dimensions | Time | Throughput |
|-----------|---------|------------|------|------------|
| Insert (batch) | 100,000 | 384 | TBD | TBD |
| Search (top-10) | 1M indexed | 384 | TBD | TBD |
| Erasure (cascading) | 1,000 deleted / 1M total | 384 | TBD | TBD |
| Audit log query | 100K entries | — | TBD | TBD |

### Running Benchmarks

```bash
cargo bench
```

## Roadmap

- [x] HNSW vector indexing (usearch)
- [x] Metadata filtering
- [x] Audit trail logging
- [x] PII tagging
- [x] Right-to-erasure with cascading deletion
- [x] Erasure certificates
- [x] HTTP API (Axum)
- [x] Python bindings (PyO3)
- [ ] BM25 full-text search (tantivy)
- [ ] Hybrid search (vector + keyword)
- [ ] Persistent storage (disk-backed collections)
- [ ] Multi-tenancy with per-tenant compliance policies
- [ ] Encryption at rest
- [ ] Role-based access control (RBAC)
- [ ] Distributed mode / sharding
- [ ] POPIA/GDPR compliance report generation
- [ ] Web dashboard for audit trail inspection
- [ ] Docker image & Helm chart

## Building from Source

```bash
git clone https://github.com/ameen/indlovu.git
cd indlovu
cargo build --release

# Run the example
cargo run --example quickstart

# Run the server
cargo run -p indlovu-server --release

# Run tests
cargo test
```

## Contributing

Contributions are welcome! Please open an issue or PR. For major changes, open a discussion first.

## License

MIT — see [LICENSE](LICENSE) for details.

---

<p align="center">
  <strong>🐘 The elephant never forgets — unless you ask it to.</strong>
</p>
