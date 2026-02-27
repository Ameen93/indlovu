//! Append-only audit log implementation.

use indlovu_core::traits::{AuditEntry, AuditLog};
use indlovu_core::Result;
use std::sync::{Arc, Mutex};

/// In-memory append-only audit log.
///
/// In production, this would be backed by a write-ahead log on disk
/// to ensure durability and tamper-evidence.
#[derive(Debug, Clone, Default)]
pub struct InMemoryAuditLog {
    entries: Arc<Mutex<Vec<AuditEntry>>>,
}

impl InMemoryAuditLog {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get all entries (for testing/inspection).
    pub fn entries(&self) -> Vec<AuditEntry> {
        self.entries.lock().unwrap().clone()
    }

    /// Count of audit entries.
    pub fn len(&self) -> usize {
        self.entries.lock().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl AuditLog for InMemoryAuditLog {
    fn log(&self, entry: AuditEntry) -> Result<()> {
        tracing::info!(
            collection = %entry.collection,
            action = ?entry.action,
            "Audit log entry"
        );
        self.entries.lock().unwrap().push(entry);
        Ok(())
    }

    fn query(
        &self,
        collection: &str,
        from: Option<chrono::DateTime<chrono::Utc>>,
        to: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<Vec<AuditEntry>> {
        let entries = self.entries.lock().unwrap();
        Ok(entries
            .iter()
            .filter(|e| {
                e.collection == collection
                    && from.as_ref().is_none_or(|f| &e.timestamp >= f)
                    && to.as_ref().is_none_or(|t| &e.timestamp <= t)
            })
            .cloned()
            .collect())
    }
}
