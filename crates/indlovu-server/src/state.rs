//! Application state shared across request handlers.

use chrono::{DateTime, Utc};
use indlovu_compliance::{CompliantStore, InMemoryAuditLog, RetentionPolicy};
use indlovu_core::Collection;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub type ManagedCollection = CompliantStore<Collection>;

#[derive(Clone, Debug, Serialize)]
pub struct ConversionEvent {
    pub timestamp: DateTime<Utc>,
    pub variant: String,
    pub cta_id: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct ConversionSummary {
    pub total_events: usize,
    pub by_variant: HashMap<String, usize>,
    pub by_cta: HashMap<String, usize>,
}

/// Shared application state.
#[derive(Clone)]
pub struct AppState {
    pub collections: Arc<RwLock<HashMap<String, Arc<RwLock<ManagedCollection>>>>>,
    pub audit_log: InMemoryAuditLog,
    pub conversions: Arc<RwLock<Vec<ConversionEvent>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            collections: Arc::new(RwLock::new(HashMap::new())),
            audit_log: InMemoryAuditLog::new(),
            conversions: Arc::new(RwLock::new(Vec::new())),
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

        let collection = Collection::new(name, dimensions, indlovu_core::Distance::Cosine)
            .map_err(|e| e.to_string())?;
        let compliant =
            CompliantStore::new(collection, self.audit_log.clone(), name.to_string(), policy);
        collections.insert(name.to_string(), Arc::new(RwLock::new(compliant)));
        Ok(())
    }

    pub fn record_conversion(&self, variant: String, cta_id: String) {
        let mut events = self.conversions.write().unwrap();
        events.push(ConversionEvent {
            timestamp: Utc::now(),
            variant,
            cta_id,
        });
    }

    pub fn conversion_summary(&self) -> ConversionSummary {
        let events = self.conversions.read().unwrap();
        let mut by_variant = HashMap::new();
        let mut by_cta = HashMap::new();

        for event in events.iter() {
            *by_variant.entry(event.variant.clone()).or_insert(0) += 1;
            *by_cta.entry(event.cta_id.clone()).or_insert(0) += 1;
        }

        ConversionSummary {
            total_events: events.len(),
            by_variant,
            by_cta,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AppState;

    #[test]
    fn conversion_summary_counts_events_per_variant_and_cta() {
        let state = AppState::new();
        state.record_conversion("a".to_string(), "hero-primary".to_string());
        state.record_conversion("b".to_string(), "hero-primary".to_string());
        state.record_conversion("b".to_string(), "footer-primary".to_string());

        let summary = state.conversion_summary();
        assert_eq!(summary.total_events, 3);
        assert_eq!(summary.by_variant.get("a"), Some(&1));
        assert_eq!(summary.by_variant.get("b"), Some(&2));
        assert_eq!(summary.by_cta.get("hero-primary"), Some(&2));
        assert_eq!(summary.by_cta.get("footer-primary"), Some(&1));
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
