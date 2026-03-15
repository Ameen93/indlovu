---
project: indlovu
type: kb-entry
status: early-prototype
stack: rust, axum, usearch, pyo3, tokio, serde, sha2
domain: database, privacy, compliance, vector-search
last_analyzed: 2026-03-15
tags: indlovu, rust, vector-database, privacy, popia, gdpr, compliance
---

# Knowledge Base Entry — Indlovu

> Canonical single-file reference for cross-project knowledge base.

## Tags

database, vector-search, privacy, popia, gdpr, compliance, rust, axum, early-prototype

## Summary

Rust workspace implementing a privacy-first vector database with HNSW search (usearch), built-in POPIA/GDPR compliance (right-to-erasure with cascading deletion, audit logging, SHA-256 tamper-evident erasure certificates). Four crates: core (Collection + search + metadata filtering), compliance (CompliantStore wrapper + audit + erasure certificates + retention policy structs), server (Axum HTTP API on :6333 with conversion analytics), python (PyO3 bindings — stub only).

**Status:** Early prototype. Core vector search, compliance wrapper, audit logging, erasure certificates, and HTTP API all work end-to-end. In-memory only (no persistence). Python bindings are a stub. 11 tests passing. Has a detailed `IMPLEMENTATION_PLAN.md` for completing the project.

## What Works

- HNSW vector search via usearch (cosine, euclidean, inner product)
- Metadata filtering (Eq, Ne, Gt, Lt, In, And, Or)
- CompliantStore wrapper with full audit logging
- Right-to-erasure with cascading deletion and SHA-256 certificates
- HTTP API (7 endpoints: health, create collection, insert, search, erase, conversion tracking)
- CI pipeline (GitHub Actions)
- Quickstart example
- Landing page with CTA A/B experiment

## What Doesn't Work Yet

- No disk persistence (all in-memory)
- Python bindings are a stub (no vector operations)
- tantivy is a dep but unused (no full-text/hybrid search)
- Retention TTL not enforced (structs defined only)
- Audit log in-memory only
- Missing API endpoints (list/get/delete collections, DSAR)

## Relationships to Other Projects

- **popia-kit** — Both address POPIA compliance from different angles (popia-kit is a toolkit, indlovu is a database)
- **carsearch** — Could use indlovu for privacy-compliant vector search on customer conversation embeddings

## Reusable Patterns

- **Compliance wrapper pattern** — `CompliantStore<S: VectorStore + ErasureSupport>` wraps any store to add compliance. Generic and reusable for any data store needing audit/erasure.
- **Layered workspace architecture** — core → compliance → server → python bindings. Clean separation pattern for Rust database projects.
- **Erasure certificates** — SHA-256 tamper-evident proof of deletion with `verify()` method. Useful pattern for any POPIA/GDPR system.
- **Trait-based store abstraction** — VectorStore + ErasureSupport + AuditLog traits allow swapping implementations without changing the compliance or server layers.
- **Source document tracking** — `source_document_id` on every record enables cascading erasure of all derived data.

## Lessons & Insights

- Baking compliance into the database layer (not application layer) ensures it can't be bypassed
- Rust workspace with multiple crates enables clean architectural separation
- PyO3 bindings make a Rust database accessible from Python ML/AI pipelines
- The compliance differentiation is genuinely unique — no major vector DB offers this built-in
- In-memory-first approach allows fast iteration but persistence is essential for any real use
