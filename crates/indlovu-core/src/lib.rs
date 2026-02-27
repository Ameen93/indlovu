//! # Indlovu Core
//!
//! Core vector storage engine with HNSW indexing, metadata filtering,
//! and compliance-aware data lifecycle management.

pub mod collection;
pub mod error;
pub mod metadata;
pub mod traits;
pub mod types;

pub use collection::Collection;
pub use error::{Error, Result};
pub use types::{Distance, SearchResult, Vector, VectorRecord};
