//! Core traits defining the Indlovu storage and compliance interfaces.
//!
//! These traits form the boundary between the core engine and the compliance layer,
//! allowing the compliance crate to intercept and audit all data operations.

use crate::error::Result;
use crate::metadata::Filter;
use crate::types::{Distance, SearchResult, Vector, VectorRecord};
use uuid::Uuid;

/// Core vector storage operations.
///
/// Implementors provide the actual vector indexing and retrieval.
/// The compliance layer wraps this trait to add audit logging,
/// PII tracking, and right-to-erasure support.
pub trait VectorStore: Send + Sync {
    /// Insert a vector record into the store.
    fn insert(&mut self, record: VectorRecord) -> Result<Uuid>;

    /// Insert multiple vector records in a batch.
    fn insert_batch(&mut self, records: Vec<VectorRecord>) -> Result<Vec<Uuid>>;

    /// Search for the top-k nearest vectors.
    fn search(
        &self,
        query: &Vector,
        top_k: usize,
        filter: Option<&Filter>,
    ) -> Result<Vec<SearchResult>>;

    /// Retrieve a record by its ID.
    fn get(&self, id: &Uuid) -> Result<Option<VectorRecord>>;

    /// Delete a record by its ID. Returns true if the record existed.
    fn delete(&mut self, id: &Uuid) -> Result<bool>;

    /// Return the number of records in the store.
    fn count(&self) -> usize;

    /// Return the vector dimensionality of this store.
    fn dimensions(&self) -> usize;

    /// Return the distance metric used by this store.
    fn distance(&self) -> Distance;
}

/// Right-to-erasure support for compliance with POPIA/GDPR.
///
/// This trait enables cascading deletion: when a source document is erased,
/// all vectors derived from it are also removed, and the erasure is logged.
pub trait ErasureSupport: VectorStore {
    /// Delete all vectors derived from the given source document ID.
    /// Returns the IDs of all deleted records.
    fn erase_by_source(&mut self, source_document_id: &str) -> Result<Vec<Uuid>>;

    /// Find all record IDs derived from a given source document.
    fn find_by_source(&self, source_document_id: &str) -> Result<Vec<Uuid>>;
}

/// Audit event types for compliance logging.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditAction {
    Insert { record_id: Uuid },
    Delete { record_id: Uuid },
    Search { top_k: usize, results_count: usize },
    Erase { source_document_id: String, deleted_ids: Vec<Uuid> },
    CollectionCreated { name: String },
    CollectionDropped { name: String },
}

/// An entry in the compliance audit log.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuditEntry {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub collection: String,
    pub action: AuditAction,
    pub actor: Option<String>,
}

/// Audit logging interface for compliance tracking.
pub trait AuditLog: Send + Sync {
    /// Record an audit event.
    fn log(&self, entry: AuditEntry) -> Result<()>;

    /// Query audit entries for a collection within a time range.
    fn query(
        &self,
        collection: &str,
        from: Option<chrono::DateTime<chrono::Utc>>,
        to: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<Vec<AuditEntry>>;
}
