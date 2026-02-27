//! Compliance-aware wrapper around any VectorStore.
//!
//! Intercepts all operations to maintain an audit trail and enforce
//! retention policies. This is the primary integration point between
//! the core engine and the compliance layer.

use crate::audit::InMemoryAuditLog;
use crate::retention::RetentionPolicy;
use indlovu_core::metadata::Filter;
use indlovu_core::traits::{AuditAction, AuditEntry, AuditLog, ErasureSupport, VectorStore};
use indlovu_core::types::{Distance, SearchResult, Vector, VectorRecord};
use indlovu_core::Result;
use uuid::Uuid;

/// A compliance-aware wrapper that adds audit logging and policy enforcement
/// to any underlying `VectorStore + ErasureSupport`.
pub struct CompliantStore<S: VectorStore + ErasureSupport> {
    inner: S,
    audit_log: InMemoryAuditLog,
    collection_name: String,
    policy: RetentionPolicy,
    actor: Option<String>,
}

impl<S: VectorStore + ErasureSupport> CompliantStore<S> {
    pub fn new(
        inner: S,
        audit_log: InMemoryAuditLog,
        collection_name: String,
        policy: RetentionPolicy,
    ) -> Self {
        let actor = None;
        let store = Self {
            inner,
            audit_log: audit_log.clone(),
            collection_name: collection_name.clone(),
            policy,
            actor,
        };

        // Log collection creation
        let _ = audit_log.log(AuditEntry {
            timestamp: chrono::Utc::now(),
            collection: collection_name,
            action: AuditAction::CollectionCreated {
                name: store.collection_name.clone(),
            },
            actor: None,
        });

        store
    }

    /// Set the current actor (user/service) for audit logging.
    pub fn set_actor(&mut self, actor: impl Into<String>) {
        self.actor = Some(actor.into());
    }

    /// Get the audit log for inspection.
    pub fn audit_log(&self) -> &InMemoryAuditLog {
        &self.audit_log
    }

    /// Get the retention policy.
    pub fn policy(&self) -> &RetentionPolicy {
        &self.policy
    }

    fn log_action(&self, action: AuditAction) {
        let _ = self.audit_log.log(AuditEntry {
            timestamp: chrono::Utc::now(),
            collection: self.collection_name.clone(),
            action,
            actor: self.actor.clone(),
        });
    }
}

impl<S: VectorStore + ErasureSupport> VectorStore for CompliantStore<S> {
    fn insert(&mut self, record: VectorRecord) -> Result<Uuid> {
        let id = self.inner.insert(record)?;
        self.log_action(AuditAction::Insert { record_id: id });
        Ok(id)
    }

    fn insert_batch(&mut self, records: Vec<VectorRecord>) -> Result<Vec<Uuid>> {
        let ids = self.inner.insert_batch(records)?;
        for &id in &ids {
            self.log_action(AuditAction::Insert { record_id: id });
        }
        Ok(ids)
    }

    fn search(
        &self,
        query: &Vector,
        top_k: usize,
        filter: Option<&Filter>,
    ) -> Result<Vec<SearchResult>> {
        let results = self.inner.search(query, top_k, filter)?;
        self.log_action(AuditAction::Search {
            top_k,
            results_count: results.len(),
        });
        Ok(results)
    }

    fn get(&self, id: &Uuid) -> Result<Option<VectorRecord>> {
        self.inner.get(id)
    }

    fn delete(&mut self, id: &Uuid) -> Result<bool> {
        let deleted = self.inner.delete(id)?;
        if deleted {
            self.log_action(AuditAction::Delete { record_id: *id });
        }
        Ok(deleted)
    }

    fn count(&self) -> usize {
        self.inner.count()
    }

    fn dimensions(&self) -> usize {
        self.inner.dimensions()
    }

    fn distance(&self) -> Distance {
        self.inner.distance()
    }
}

impl<S: VectorStore + ErasureSupport> ErasureSupport for CompliantStore<S> {
    fn erase_by_source(&mut self, source_document_id: &str) -> Result<Vec<Uuid>> {
        let deleted_ids = self.inner.erase_by_source(source_document_id)?;
        self.log_action(AuditAction::Erase {
            source_document_id: source_document_id.to_string(),
            deleted_ids: deleted_ids.clone(),
        });
        tracing::warn!(
            source = source_document_id,
            count = deleted_ids.len(),
            "Right-to-erasure executed"
        );
        Ok(deleted_ids)
    }

    fn find_by_source(&self, source_document_id: &str) -> Result<Vec<Uuid>> {
        self.inner.find_by_source(source_document_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indlovu_core::Collection;
    use serde_json::json;

    #[test]
    fn test_compliant_store_audit_trail() {
        let collection = Collection::new("test", 64, Distance::Cosine).unwrap();
        let audit_log = InMemoryAuditLog::new();
        let mut store = CompliantStore::new(
            collection,
            audit_log.clone(),
            "test".into(),
            RetentionPolicy::default(),
        );

        store.set_actor("user:ameen");

        // Insert
        let record = VectorRecord::new(vec![1.0; 64], json!({}), Some("doc-1".into()), true);
        store.insert(record).unwrap();

        // Search
        let _ = store.search(&vec![1.0; 64], 5, None).unwrap();

        // Erase
        store.erase_by_source("doc-1").unwrap();

        // Verify audit trail
        let entries = audit_log.entries();
        assert_eq!(entries.len(), 4); // create + insert + search + erase
        assert!(entries.iter().all(|e| e.collection == "test"));
    }
}
