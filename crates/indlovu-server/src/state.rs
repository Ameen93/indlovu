//! Application state shared across request handlers.

use indlovu_compliance::{CompliantStore, InMemoryAuditLog, RetentionPolicy};
use indlovu_core::Collection;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub type ManagedCollection = CompliantStore<Collection>;

/// Shared application state.
#[derive(Clone)]
pub struct AppState {
    pub collections: Arc<RwLock<HashMap<String, Arc<RwLock<ManagedCollection>>>>>,
    pub audit_log: InMemoryAuditLog,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            collections: Arc::new(RwLock::new(HashMap::new())),
            audit_log: InMemoryAuditLog::new(),
        }
    }

    pub fn create_collection(
        &self,
        name: &str,
        dimensions: usize,
        policy: RetentionPolicy,
    ) -> Result<(), String> {
        let mut collections = self.collections.write().unwrap();
        if collections.contains_key(name) {
            return Err(format!("Collection '{}' already exists", name));
        }

        let collection =
            Collection::new(name, dimensions, indlovu_core::Distance::Cosine)
                .map_err(|e| e.to_string())?;
        let compliant =
            CompliantStore::new(collection, self.audit_log.clone(), name.to_string(), policy);
        collections.insert(name.to_string(), Arc::new(RwLock::new(compliant)));
        Ok(())
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
