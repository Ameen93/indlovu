---
project: indlovu
type: overview
status: early-prototype
stack: rust, axum, usearch, pyo3, tokio, serde, sha2
domain: database, privacy, compliance, vector-search
last_analyzed: 2026-03-15
tags: indlovu, rust, vector-database, privacy, popia, gdpr, compliance
---

# Indlovu — Project Overview

> Privacy-first vector database with built-in POPIA/GDPR compliance — right-to-erasure, audit logging, and erasure certificates baked in.

## What This Is

Indlovu is a Rust workspace implementing a privacy-first vector search database. It wraps HNSW vector search (via usearch) with a compliance layer that provides automatic audit logging, right-to-erasure support (cascading deletion by source document ID across collections), and tamper-evident erasure certificates (SHA-256 hashed). Exposed as an HTTP API (Axum on :6333) with planned Python bindings (PyO3).

The name means "elephant" in isiZulu — "the elephant never forgets, unless you ask it to."

## Problem It Solves

Vector databases typically lack built-in compliance features. If you store personal data as embeddings (e.g., customer support tickets, user profiles), POPIA/GDPR requires right-to-erasure and audit trails. Most teams bolt this on at the application layer, which is error-prone and hard to verify. Indlovu bakes compliance into the database layer so it can't be bypassed.

## Target User

Developers building AI/ML applications that handle personal data in POPIA/GDPR-regulated environments, particularly in South Africa.

## Architecture

Layered crate design where each layer wraps the one below:

```
indlovu-server (Axum HTTP API on :6333)
    ↓ uses
indlovu-compliance (CompliantStore wrapper, audit log, erasure certificates)
    ↓ wraps
indlovu-core (Collection with HNSW via usearch, metadata filtering)

indlovu-python (PyO3 bindings — separate, currently a stub)
```

### Core Pattern

`CompliantStore<S: VectorStore + ErasureSupport>` is a generic wrapper that intercepts all data operations on any store implementing the core traits, adding:
- Audit logging of every insert, search, delete, and erasure
- Actor tracking (which user/service performed the action)
- Retention policy association

### Key Traits (`indlovu-core/src/traits.rs`)

- **`VectorStore`** — insert, insert_batch, search (with optional metadata Filter), get, delete, count, dimensions, distance
- **`ErasureSupport`** — erase_by_source, find_by_source — enables cascading deletion by source_document_id
- **`AuditLog`** — log and query audit entries by collection and time range

### Metadata Filtering

The `Filter` enum supports: `Eq`, `Ne`, `Gt`, `Lt`, `In`, `And`, `Or`. Filters are applied post-HNSW-search (the search over-fetches by 4x when filters are present to compensate for filtered-out results).

### Erasure Flow

1. `erase_by_source(source_document_id)` finds all records with that source
2. Deletes each from the record HashMap and usearch HNSW index
3. `CompliantStore` wrapper adds audit logging of the erasure
4. `execute_erasure()` handles multi-collection erasure and produces an `ErasureCertificate`
5. Certificate includes SHA-256 hash for tamper-evidence, with a `verify()` method

## Current Status (as of 2026-03-15)

**Early prototype / functional scaffold.** Core functionality works end-to-end.

### What Works

| Component | Status | Details |
|-----------|--------|---------|
| HNSW vector search | ✅ Working | usearch backend, cosine/euclidean/inner-product |
| Metadata filtering | ✅ Working | Eq, Ne, Gt, Lt, In, And, Or on JSON metadata |
| Compliance wrapper | ✅ Working | CompliantStore wraps Collection with audit logging |
| Audit logging | ✅ Working | In-memory append-only log, queryable by collection/time |
| Right-to-erasure | ✅ Working | Cascading delete by source_document_id |
| Erasure certificates | ✅ Working | SHA-256 hashed, tamper-detectable, JSON-serializable |
| HTTP API | ✅ Working | 7 endpoints: health, create collection, insert, search, erase, conversion tracking |
| CI pipeline | ✅ Working | GitHub Actions: fmt → clippy → test → build examples |
| Quickstart example | ✅ Working | Full insert → search → erase → audit trail demo |
| Landing page | ✅ Working | Dark-themed site with A/B CTA experiment |
| Tests | ✅ Passing | 11 tests (core: 7, compliance: 3, server: 1) |

### What's Not Implemented

| Component | Status | Notes |
|-----------|--------|-------|
| Disk persistence | ❌ | All data in-memory, lost on restart. Plan: WAL + snapshots |
| Full-text search | ❌ | tantivy is a workspace dep but completely unused |
| Hybrid search | ❌ | No RRF implementation yet |
| Python bindings | ⚠️ Stub | PyCollection has name/dimensions only, no vector operations |
| Retention enforcement | ⚠️ Defined | RetentionPolicy structs exist but TTL is not enforced |
| Persistent audit log | ❌ | In-memory only, no disk persistence or chain-hashing |
| DSAR endpoint | ❌ | Not implemented |
| List/info/delete collection endpoints | ❌ | Only create exists |
| Benchmarks | ❌ | Not yet run |
| Docker | ❌ | No Dockerfile |

## Crate Details

### indlovu-core (`crates/indlovu-core/`)

The storage engine. Key files:
- `collection.rs` — `Collection` struct with usearch HNSW index, HashMap<Uuid, VectorRecord> for records, key mapping for usearch u64 keys ↔ Uuid IDs
- `types.rs` — `Vector` (Vec<f32>), `VectorRecord` (id, vector, metadata, source_document_id, created_at, contains_pii), `SearchResult`, `Distance` enum
- `metadata.rs` — `Filter` enum with `matches()` method for post-search filtering
- `traits.rs` — Core trait definitions + `AuditAction` enum + `AuditEntry` struct
- `error.rs` — Error types (CollectionNotFound, DimensionMismatch, RecordNotFound, etc.)

Dependencies: usearch, serde, uuid, chrono, thiserror, tracing

### indlovu-compliance (`crates/indlovu-compliance/`)

Privacy/compliance layer. Key files:
- `compliant_store.rs` — `CompliantStore<S>` generic wrapper
- `audit.rs` — `InMemoryAuditLog` with Arc<Mutex<Vec<AuditEntry>>>
- `erasure.rs` — `ErasureCertificate` struct + `execute_erasure()` function
- `retention.rs` — `RetentionPolicy` struct + `ComplianceFramework` enum (POPIA, GDPR)

Additional dependency: sha2 (for certificate hashing)

### indlovu-server (`crates/indlovu-server/`)

HTTP API. Key files:
- `main.rs` — Tokio main, tracing setup, TcpListener on 0.0.0.0:6333
- `state.rs` — `AppState` with collections map + conversion event tracking + `ConversionSummary`
- `routes.rs` — Route definitions with CORS and tracing middleware
- `handlers.rs` — 7 handler functions with serde request/response types

### indlovu-python (`crates/indlovu-python/`)

PyO3 bindings stub. Single file (`lib.rs`) with:
- `PyCollection` class — stores name and dimensions, has `__repr__`
- No actual vector operations connected

## Key Design Decisions

1. **Compliance is structural, not optional** — The `CompliantStore` wrapper intercepts all operations at the trait level. You can't bypass audit logging.
2. **Source document tracking** — Every vector can have a `source_document_id` enabling cascading erasure of all derived data.
3. **Tamper-evident certificates** — Erasure certificates use SHA-256 hashing so any modification is detectable.
4. **In-memory first** — Current implementation is pure in-memory. Persistence is planned but not yet built.
5. **Trait-based abstraction** — `VectorStore` + `ErasureSupport` traits allow swapping storage backends.

## Implementation Plan

A detailed phased implementation plan exists in `IMPLEMENTATION_PLAN.md` covering:
- Phase 1: Core hardening (tests, persistence, hybrid search)
- Phase 2: Compliance layer (persistent audit, retention enforcement, DSAR)
- Phase 3: Server & Python DX
- Phase 4: Polish & launch (benchmarks, docs, packaging)

## File Structure

```
indlovu/
├── Cargo.toml                         # Workspace root
├── CLAUDE.md                          # AI coding context
├── README.md                          # Project README
├── LICENSE                            # MIT
├── IMPLEMENTATION_PLAN.md             # Detailed build plan
├── .github/workflows/ci.yml           # GitHub Actions CI
├── crates/
│   ├── indlovu-core/                  # Storage engine
│   │   └── src/ (lib, collection, types, metadata, traits, error)
│   ├── indlovu-compliance/            # Privacy layer
│   │   └── src/ (lib, compliant_store, audit, erasure, retention)
│   ├── indlovu-server/                # HTTP API
│   │   └── src/ (main, lib, routes, handlers, state)
│   └── indlovu-python/                # PyO3 bindings (stub)
│       └── src/ (lib)
├── examples/
│   └── quickstart.rs                  # Working example
├── docs/
│   ├── PROJECT_OVERVIEW.md            # This file
│   └── KNOWLEDGE_BASE_ENTRY.md        # KB entry
└── site/
    └── index.html                     # Landing page with A/B experiment
```

## Relationships to Other Projects

- **popia-kit** — Both address POPIA compliance from different angles (popia-kit is a toolkit, indlovu is a database)
- **carsearch** — Could use indlovu for privacy-compliant vector search on customer data

## Commercial Potential

Strong positioning for the South African market where POPIA compliance is legally required. The "compliance built-in, not bolted-on" angle is genuinely differentiated — most vector databases (Pinecone, Chroma, Qdrant, Weaviate) don't offer this at the database layer.

Key questions to resolve:
- Is the target audience large enough? (SA companies using vector DBs AND caring about POPIA)
- Open-source strategic asset vs. commercial product?
- Integration story: does it need LangChain/LlamaIndex integrations to be useful?
