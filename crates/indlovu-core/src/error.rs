//! Error types for Indlovu Core.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Collection '{0}' not found")]
    CollectionNotFound(String),

    #[error("Vector dimension mismatch: expected {expected}, got {got}")]
    DimensionMismatch { expected: usize, got: usize },

    #[error("Record '{0}' not found")]
    RecordNotFound(uuid::Uuid),

    #[error("Collection '{0}' already exists")]
    CollectionAlreadyExists(String),

    #[error("Index error: {0}")]
    IndexError(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
