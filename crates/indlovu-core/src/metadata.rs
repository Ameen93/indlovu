//! Metadata filtering for vector search queries.

use serde::{Deserialize, Serialize};

/// A filter condition for metadata-based query refinement.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Filter {
    /// Field equals value.
    Eq { field: String, value: serde_json::Value },
    /// Field not equal to value.
    Ne { field: String, value: serde_json::Value },
    /// Field greater than value (numeric).
    Gt { field: String, value: f64 },
    /// Field less than value (numeric).
    Lt { field: String, value: f64 },
    /// Field value is in the given set.
    In { field: String, values: Vec<serde_json::Value> },
    /// All conditions must match.
    And(Vec<Filter>),
    /// At least one condition must match.
    Or(Vec<Filter>),
}

impl Filter {
    /// Evaluate this filter against a metadata JSON value.
    pub fn matches(&self, metadata: &serde_json::Value) -> bool {
        match self {
            Filter::Eq { field, value } => metadata.get(field) == Some(value),
            Filter::Ne { field, value } => metadata.get(field) != Some(value),
            Filter::Gt { field, value } => metadata
                .get(field)
                .and_then(|v| v.as_f64())
                .is_some_and(|v| v > *value),
            Filter::Lt { field, value } => metadata
                .get(field)
                .and_then(|v| v.as_f64())
                .is_some_and(|v| v < *value),
            Filter::In { field, values } => metadata
                .get(field)
                .is_some_and(|v| values.contains(v)),
            Filter::And(filters) => filters.iter().all(|f| f.matches(metadata)),
            Filter::Or(filters) => filters.iter().any(|f| f.matches(metadata)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_eq_filter() {
        let metadata = json!({"category": "science", "score": 0.95});
        let filter = Filter::Eq {
            field: "category".into(),
            value: json!("science"),
        };
        assert!(filter.matches(&metadata));
    }

    #[test]
    fn test_gt_filter() {
        let metadata = json!({"score": 0.95});
        let filter = Filter::Gt { field: "score".into(), value: 0.5 };
        assert!(filter.matches(&metadata));
    }

    #[test]
    fn test_and_filter() {
        let metadata = json!({"category": "science", "score": 0.95});
        let filter = Filter::And(vec![
            Filter::Eq { field: "category".into(), value: json!("science") },
            Filter::Gt { field: "score".into(), value: 0.5 },
        ]);
        assert!(filter.matches(&metadata));
    }
}
