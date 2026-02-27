//! Retention policy configuration for PII-tagged collections.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Retention policy for data lifecycle management.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    /// Maximum age for records before automatic purging.
    pub max_age: Option<Duration>,

    /// Whether this collection contains PII (triggers stricter controls).
    pub contains_pii: bool,

    /// Applicable compliance frameworks.
    pub frameworks: Vec<ComplianceFramework>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum ComplianceFramework {
    Popia,
    Gdpr,
}

impl Default for RetentionPolicy {
    fn default() -> Self {
        Self {
            max_age: None,
            contains_pii: false,
            frameworks: Vec::new(),
        }
    }
}

impl RetentionPolicy {
    /// Create a POPIA-compliant retention policy.
    pub fn popia(max_age: Duration) -> Self {
        Self {
            max_age: Some(max_age),
            contains_pii: true,
            frameworks: vec![ComplianceFramework::Popia],
        }
    }

    /// Create a GDPR-compliant retention policy.
    pub fn gdpr(max_age: Duration) -> Self {
        Self {
            max_age: Some(max_age),
            contains_pii: true,
            frameworks: vec![ComplianceFramework::Gdpr],
        }
    }
}
