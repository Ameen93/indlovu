//! # Indlovu Compliance
//!
//! POPIA/GDPR compliance layer providing:
//! - Append-only audit logging of all data operations
//! - PII tagging and tracking on collections
//! - Right-to-erasure with cascading deletion and audit proof
//! - Configurable retention policies

pub mod audit;
pub mod compliant_store;
pub mod erasure;
pub mod retention;

pub use audit::InMemoryAuditLog;
pub use compliant_store::CompliantStore;
pub use erasure::{execute_erasure, ErasureCertificate};
pub use retention::RetentionPolicy;
