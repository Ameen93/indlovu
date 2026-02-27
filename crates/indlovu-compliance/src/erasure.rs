//! Right-to-erasure implementation with audit-proof erasure certificates.

use indlovu_core::traits::ErasureSupport;
use indlovu_core::Result;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

/// An audit-proof erasure certificate generated after right-to-erasure execution.
///
/// Contains all information needed to prove that erasure was performed,
/// including a SHA-256 hash for tamper-evidence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErasureCertificate {
    /// Unique certificate ID.
    pub certificate_id: Uuid,
    /// When the erasure was performed.
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// The subject or source document ID that was erased.
    pub subject_id: String,
    /// IDs of all vectors that were deleted.
    pub vectors_deleted: Vec<Uuid>,
    /// Number of vectors deleted.
    pub vectors_deleted_count: usize,
    /// Collections that were affected.
    pub collections_affected: Vec<String>,
    /// The actor who initiated the erasure.
    pub initiated_by: Option<String>,
    /// SHA-256 hash of the certificate contents (excluding this field) for tamper-evidence.
    pub sha256_hash: String,
}

impl ErasureCertificate {
    /// Compute the SHA-256 hash of the certificate's core fields.
    fn compute_hash(
        certificate_id: &Uuid,
        timestamp: &chrono::DateTime<chrono::Utc>,
        subject_id: &str,
        vectors_deleted: &[Uuid],
        collections_affected: &[String],
    ) -> String {
        let mut hasher = Sha256::new();
        hasher.update(certificate_id.to_string().as_bytes());
        hasher.update(timestamp.to_rfc3339().as_bytes());
        hasher.update(subject_id.as_bytes());
        for id in vectors_deleted {
            hasher.update(id.to_string().as_bytes());
        }
        for col in collections_affected {
            hasher.update(col.as_bytes());
        }
        format!("{:x}", hasher.finalize())
    }

    /// Verify the integrity of this certificate.
    pub fn verify(&self) -> bool {
        let expected = Self::compute_hash(
            &self.certificate_id,
            &self.timestamp,
            &self.subject_id,
            &self.vectors_deleted,
            &self.collections_affected,
        );
        self.sha256_hash == expected
    }

    /// Serialize to JSON string.
    pub fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }
}

/// Execute right-to-erasure across one or more collections and produce an erasure certificate.
///
/// This is the headline compliance feature: given a subject/source document ID,
/// find and delete all their vectors across all provided collections,
/// and generate a cryptographic receipt proving the erasure occurred.
pub fn execute_erasure<S: ErasureSupport>(
    stores: &mut [(String, &mut S)],
    subject_id: &str,
    initiated_by: Option<String>,
) -> Result<ErasureCertificate> {
    let certificate_id = Uuid::new_v4();
    let timestamp = chrono::Utc::now();
    let mut all_deleted: Vec<Uuid> = Vec::new();
    let mut affected_collections: Vec<String> = Vec::new();

    for (collection_name, store) in stores.iter_mut() {
        let deleted = store.erase_by_source(subject_id)?;
        if !deleted.is_empty() {
            affected_collections.push(collection_name.clone());
            all_deleted.extend(deleted);
        }
    }

    let sha256_hash = ErasureCertificate::compute_hash(
        &certificate_id,
        &timestamp,
        subject_id,
        &all_deleted,
        &affected_collections,
    );

    Ok(ErasureCertificate {
        certificate_id,
        timestamp,
        subject_id: subject_id.to_string(),
        vectors_deleted_count: all_deleted.len(),
        vectors_deleted: all_deleted,
        collections_affected: affected_collections,
        initiated_by,
        sha256_hash,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use indlovu_core::types::{Distance, VectorRecord};
    use indlovu_core::Collection;
    use serde_json::json;

    #[test]
    fn test_erasure_certificate() {
        let mut col = Collection::new("test", 4, Distance::Cosine).unwrap();

        // Insert records from two sources
        use indlovu_core::traits::VectorStore;
        let r1 = VectorRecord::new(vec![1.0; 4], json!({}), Some("user-123".into()), true);
        let r2 = VectorRecord::new(vec![0.5; 4], json!({}), Some("user-123".into()), true);
        let r3 = VectorRecord::new(vec![0.0, 1.0, 0.0, 1.0], json!({}), Some("user-456".into()), false);
        col.insert(r1).unwrap();
        col.insert(r2).unwrap();
        col.insert(r3).unwrap();

        // Execute erasure
        let mut stores: Vec<(String, &mut Collection)> =
            vec![("test".to_string(), &mut col)];
        let cert = execute_erasure(&mut stores, "user-123", Some("compliance-officer".into())).unwrap();

        assert_eq!(cert.vectors_deleted_count, 2);
        assert_eq!(cert.collections_affected, vec!["test"]);
        assert_eq!(cert.subject_id, "user-123");
        assert!(cert.verify());

        // Verify JSON serialization
        let json = cert.to_json().unwrap();
        assert!(json.contains("user-123"));

        // Remaining count
        assert_eq!(col.count(), 1);
    }

    #[test]
    fn test_certificate_tamper_detection() {
        let mut col = Collection::new("test", 4, Distance::Cosine).unwrap();
        use indlovu_core::traits::VectorStore;
        let r = VectorRecord::new(vec![1.0; 4], json!({}), Some("user-1".into()), true);
        col.insert(r).unwrap();

        let mut stores = vec![("test".to_string(), &mut col)];
        let mut cert = execute_erasure(&mut stores, "user-1", None).unwrap();

        assert!(cert.verify());

        // Tamper with the certificate
        cert.subject_id = "user-2".to_string();
        assert!(!cert.verify());
    }
}
