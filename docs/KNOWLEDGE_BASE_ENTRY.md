---
project: indlovu
type: kb-entry
status: needs-review
stack: rust, axum, usearch, tantivy, pyo3, tokio
domain: database, privacy, compliance
last_analyzed: 2026-03-14
tags: indlovu, rust, vector-database, privacy, popia, gdpr
---

# Knowledge Base Entry — Indlovu

> Canonical single-file reference for cross-project knowledge base.

## Tags

database, vector-search, privacy, popia, gdpr, compliance, rust, axum, needs-review

## Summary

Rust workspace implementing a privacy-first vector database with HNSW search (usearch), built-in POPIA/GDPR compliance (right-to-erasure, audit logging, SHA-256 erasure certificates). Four crates: core (Collection + search + metadata), compliance (CompliantStore wrapper), server (Axum HTTP API on :6333), python (PyO3 bindings). Early prototype, needs review to determine completeness.

## Relationships to Other Projects

- **popia-kit** — Both address POPIA compliance from different angles (popia-kit is a toolkit, indlovu is a database)
- **carsearch** — Could use indlovu for privacy-compliant vector search on customer conversation embeddings

## Reusable Patterns

- **Compliance wrapper pattern** — `CompliantStore<S: VectorStore + ErasureSupport>` wraps any store to add compliance. Generic and reusable.
- **Layered workspace architecture** — core → compliance → server → python bindings. Clean separation for any Rust database project.
- **Erasure certificates** — SHA-256 tamper-evident proof of deletion. Useful pattern for any POPIA/GDPR system.
- **Trait-based store abstraction** — VectorStore + ErasureSupport + AuditLog traits allow swapping implementations.

## Lessons & Insights

- Baking compliance into the database layer (not application layer) ensures it can't be bypassed
- Rust workspace with multiple crates enables clean architectural separation
- PyO3 bindings make a Rust database accessible from Python ML/AI pipelines
