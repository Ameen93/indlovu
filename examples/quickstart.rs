//! # Indlovu Quickstart Example
//!
//! Demonstrates: create collection → insert vectors → search → right-to-erasure.
//!
//! ```sh
//! cargo run --example quickstart
//! ```

use indlovu_compliance::{CompliantStore, InMemoryAuditLog, RetentionPolicy};
use indlovu_core::traits::{AuditLog, ErasureSupport, VectorStore};
use indlovu_core::types::{Distance, VectorRecord};
use serde_json::json;
use std::time::Duration;

fn main() {
    println!("🐘 Indlovu Quickstart\n");

    // ── 1. Create a collection with POPIA compliance ────────────
    let collection =
        indlovu_core::Collection::new("customer_embeddings", 4, Distance::Cosine).unwrap();

    let audit_log = InMemoryAuditLog::new();
    let policy = RetentionPolicy::popia(Duration::from_secs(90 * 24 * 3600)); // 90 days

    let mut store = CompliantStore::new(
        collection,
        audit_log.clone(),
        "customer_embeddings".into(),
        policy,
    );

    store.set_actor("system:ingestion-pipeline");
    println!("✅ Created POPIA-compliant collection 'customer_embeddings'");

    // ── 2. Insert vectors with source document tracking ─────────
    let records = vec![
        VectorRecord::new(
            vec![0.1, 0.2, 0.3, 0.4],
            json!({"customer": "Sipho", "type": "support_ticket"}),
            Some("ticket-001".into()),
            true, // contains PII
        ),
        VectorRecord::new(
            vec![0.5, 0.6, 0.7, 0.8],
            json!({"customer": "Sipho", "type": "purchase_history"}),
            Some("ticket-001".into()),
            true,
        ),
        VectorRecord::new(
            vec![0.9, 0.1, 0.2, 0.3],
            json!({"customer": "Naledi", "type": "support_ticket"}),
            Some("ticket-002".into()),
            true,
        ),
    ];

    let ids = store.insert_batch(records).unwrap();
    println!("✅ Inserted {} vectors (PII-tagged, source-tracked)", ids.len());

    // ── 3. Search ───────────────────────────────────────────────
    let query = vec![0.1, 0.2, 0.3, 0.4];
    let results = store.search(&query, 2, None).unwrap();

    println!("\n🔍 Search results (top 2):");
    for result in &results {
        println!(
            "   → {} (distance: {:.4}) metadata: {}",
            result.record.id, result.distance, result.record.metadata
        );
    }

    // ── 4. Right-to-erasure (POPIA Section 24) ──────────────────
    println!("\n⚠️  Sipho requests data deletion (POPIA right-to-erasure)...");
    store.set_actor("compliance:erasure-handler");

    let erased = store.erase_by_source("ticket-001").unwrap();
    println!(
        "🗑️  Erased {} vectors derived from source 'ticket-001'",
        erased.len()
    );
    println!("   Remaining records: {}", store.count());

    // ── 5. Audit trail ──────────────────────────────────────────
    println!("\n📋 Audit trail ({} entries):", audit_log.len());
    for entry in audit_log.entries() {
        println!(
            "   [{}] {:?} by {:?}",
            entry.timestamp.format("%H:%M:%S"),
            entry.action,
            entry.actor.unwrap_or_else(|| "system".into())
        );
    }

    println!("\n🐘 Done. The elephant never forgets — unless you ask it to.");
}
