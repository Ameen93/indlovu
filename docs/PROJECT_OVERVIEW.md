---
project: indlovu
type: overview
status: needs-review
stack: rust, axum, usearch, tantivy, pyo3, tokio, serde
domain: database, privacy, compliance
last_analyzed: 2026-03-14
tags: indlovu, rust, vector-database, privacy, popia, gdpr, compliance
---

# Indlovu
> Privacy-first vector database with built-in POPIA/GDPR compliance — right-to-erasure, audit logging, and erasure certificates baked in.

## What This Is

Indlovu is a Rust workspace implementing a privacy-first vector search database. It wraps HNSW vector search (via usearch) with a compliance layer that provides automatic audit logging, right-to-erasure support (cascading deletion by source document ID across collections), and tamper-evident erasure certificates (SHA-256 hashed). Exposed as an HTTP API (Axum on :6333) and Python bindings (PyO3).

The layered architecture: `indlovu-core` (Collection + HNSW + metadata filtering) → `indlovu-compliance` (CompliantStore wrapper + audit + erasure) → `indlovu-server` (Axum HTTP API) + `indlovu-python` (PyO3 bindings).

## Problem It Solves

Vector databases typically lack built-in compliance features. If you store personal data as embeddings, POPIA/GDPR requires right-to-erasure and audit trails. Indlovu bakes these in at the database layer rather than requiring application-level compliance.

## Target User

Developers building AI/ML applications that handle personal data in POPIA/GDPR-regulated environments, particularly in South Africa.

## Current Status

**Needs review / Early prototype**

Per operator card: "Unknown state — possibly complete. Needs review to determine status."

The codebase is well-structured with 4 workspace crates, CI (GitHub Actions: fmt → clippy → test → build examples), MIT license, and a quickstart example. However, it's unclear how many features are fully implemented vs. stubbed.

## Key Links & Entry Points

| Item | Location |
|------|----------|
| Server entry | `crates/indlovu-server/src/main.rs` (Axum on :6333) |
| Core traits | `crates/indlovu-core/src/traits.rs` (VectorStore, ErasureSupport, AuditLog) |
| Compliance wrapper | `crates/indlovu-compliance/src/compliant_store.rs` |
| Erasure flow | `crates/indlovu-compliance/src/erasure.rs` |
| Python bindings | `crates/indlovu-python/src/lib.rs` |
| Quickstart | `examples/quickstart.rs` |
| CI | `.github/workflows/ci.yml` |
| Implementation plan | `IMPLEMENTATION_PLAN.md` |
| Landing page | `site/index.html` |
