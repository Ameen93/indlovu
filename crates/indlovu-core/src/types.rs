//! Core data types for Indlovu.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A dense vector of f32 values.
pub type Vector = Vec<f32>;

/// Distance metric for vector similarity search.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Distance {
    Cosine,
    Euclidean,
    InnerProduct,
}

impl Default for Distance {
    fn default() -> Self {
        Self::Cosine
    }
}

/// A stored vector record with metadata and provenance tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorRecord {
    /// Unique identifier for this record.
    pub id: Uuid,

    /// The dense vector embedding.
    pub vector: Vector,

    /// Arbitrary JSON metadata attached to this record.
    pub metadata: serde_json::Value,

    /// Optional source document ID for right-to-erasure cascading.
    /// When a source document is erased, all derived vectors are also removed.
    pub source_document_id: Option<String>,

    /// Timestamp of insertion.
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// Whether this record contains or is derived from PII.
    pub contains_pii: bool,
}

impl VectorRecord {
    pub fn new(
        vector: Vector,
        metadata: serde_json::Value,
        source_document_id: Option<String>,
        contains_pii: bool,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            vector,
            metadata,
            source_document_id,
            created_at: chrono::Utc::now(),
            contains_pii,
        }
    }
}

/// A search result with distance score.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub record: VectorRecord,
    pub distance: f32,
}
