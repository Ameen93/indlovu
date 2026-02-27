//! In-memory vector collection with HNSW indexing via usearch.

use crate::error::{Error, Result};
use crate::metadata::Filter;
use crate::traits::{ErasureSupport, VectorStore};
use crate::types::{Distance, SearchResult, Vector, VectorRecord};
use std::collections::HashMap;
use uuid::Uuid;

/// An in-memory vector collection backed by usearch HNSW index.
pub struct Collection {
    name: String,
    dimensions: usize,
    distance: Distance,
    records: HashMap<Uuid, VectorRecord>,
    index: usearch::Index,
    key_map: HashMap<Uuid, u64>,
    next_key: u64,
}

impl Collection {
    /// Create a new collection with the given name, dimensions, and distance metric.
    pub fn new(name: impl Into<String>, dimensions: usize, distance: Distance) -> Result<Self> {
        let metric = match distance {
            Distance::Cosine => usearch::MetricKind::Cos,
            Distance::Euclidean => usearch::MetricKind::L2sq,
            Distance::InnerProduct => usearch::MetricKind::IP,
        };

        let index = usearch::new_index(&usearch::IndexOptions {
            dimensions,
            metric,
            quantization: usearch::ScalarKind::F32,
            connectivity: 16,
            expansion_add: 128,
            expansion_search: 64,
            multi: false,
        })
        .map_err(|e| Error::IndexError(e.to_string()))?;

        index
            .reserve(1024)
            .map_err(|e| Error::IndexError(e.to_string()))?;

        Ok(Self {
            name: name.into(),
            dimensions,
            distance,
            records: HashMap::new(),
            index,
            key_map: HashMap::new(),
            next_key: 0,
        })
    }

    /// Get the name of this collection.
    pub fn name(&self) -> &str {
        &self.name
    }

    fn allocate_key(&mut self) -> u64 {
        let key = self.next_key;
        self.next_key += 1;
        key
    }
}

impl VectorStore for Collection {
    fn insert(&mut self, record: VectorRecord) -> Result<Uuid> {
        if record.vector.len() != self.dimensions {
            return Err(Error::DimensionMismatch {
                expected: self.dimensions,
                got: record.vector.len(),
            });
        }

        let id = record.id;
        let key = self.allocate_key();

        self.index
            .add(key, &record.vector)
            .map_err(|e| Error::IndexError(e.to_string()))?;

        self.key_map.insert(id, key);
        self.records.insert(id, record);

        Ok(id)
    }

    fn insert_batch(&mut self, records: Vec<VectorRecord>) -> Result<Vec<Uuid>> {
        records.into_iter().map(|r| self.insert(r)).collect()
    }

    fn search(
        &self,
        query: &Vector,
        top_k: usize,
        filter: Option<&Filter>,
    ) -> Result<Vec<SearchResult>> {
        if query.len() != self.dimensions {
            return Err(Error::DimensionMismatch {
                expected: self.dimensions,
                got: query.len(),
            });
        }

        // Search with extra candidates to account for filtering
        let search_k = if filter.is_some() { top_k * 4 } else { top_k };
        let results = self
            .index
            .search(query, search_k)
            .map_err(|e| Error::IndexError(e.to_string()))?;

        let key_to_id: HashMap<u64, Uuid> =
            self.key_map.iter().map(|(id, key)| (*key, *id)).collect();

        let mut search_results: Vec<SearchResult> = results
            .keys
            .iter()
            .zip(results.distances.iter())
            .filter_map(|(key, dist)| {
                let id = key_to_id.get(key)?;
                let record = self.records.get(id)?;

                if let Some(f) = filter {
                    if !f.matches(&record.metadata) {
                        return None;
                    }
                }

                Some(SearchResult {
                    record: record.clone(),
                    distance: *dist,
                })
            })
            .take(top_k)
            .collect();

        search_results.sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());
        Ok(search_results)
    }

    fn get(&self, id: &Uuid) -> Result<Option<VectorRecord>> {
        Ok(self.records.get(id).cloned())
    }

    fn delete(&mut self, id: &Uuid) -> Result<bool> {
        if let Some(key) = self.key_map.remove(id) {
            self.index
                .remove(key)
                .map_err(|e| Error::IndexError(e.to_string()))?;
            self.records.remove(id);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn count(&self) -> usize {
        self.records.len()
    }

    fn dimensions(&self) -> usize {
        self.dimensions
    }

    fn distance(&self) -> Distance {
        self.distance
    }
}

impl ErasureSupport for Collection {
    fn erase_by_source(&mut self, source_document_id: &str) -> Result<Vec<Uuid>> {
        let ids = self.find_by_source(source_document_id)?;
        for id in &ids {
            self.delete(id)?;
        }
        Ok(ids)
    }

    fn find_by_source(&self, source_document_id: &str) -> Result<Vec<Uuid>> {
        Ok(self
            .records
            .values()
            .filter(|r| r.source_document_id.as_deref() == Some(source_document_id))
            .map(|r| r.id)
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_record(dims: usize, source: Option<&str>) -> VectorRecord {
        let vector: Vec<f32> = (0..dims).map(|i| i as f32 / dims as f32).collect();
        VectorRecord::new(
            vector,
            json!({"test": true}),
            source.map(String::from),
            false,
        )
    }

    #[test]
    fn test_insert_and_search() {
        let mut col = Collection::new("test", 128, Distance::Cosine).unwrap();
        let record = make_record(128, None);
        let id = col.insert(record).unwrap();

        assert_eq!(col.count(), 1);

        let query: Vec<f32> = (0..128).map(|i| i as f32 / 128.0).collect();
        let results = col.search(&query, 5, None).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].record.id, id);
    }

    #[test]
    fn test_delete() {
        let mut col = Collection::new("test", 64, Distance::Cosine).unwrap();
        let record = make_record(64, None);
        let id = col.insert(record).unwrap();
        assert_eq!(col.count(), 1);

        assert!(col.delete(&id).unwrap());
        assert_eq!(col.count(), 0);
    }

    #[test]
    fn test_right_to_erasure() {
        let mut col = Collection::new("test", 64, Distance::Cosine).unwrap();

        // Insert 3 records, 2 from same source
        let r1 = VectorRecord::new(vec![1.0; 64], json!({}), Some("doc-123".into()), true);
        let r2 = VectorRecord::new(vec![0.5; 64], json!({}), Some("doc-123".into()), true);
        let r3 = VectorRecord::new(vec![0.0; 64], json!({}), Some("doc-456".into()), false);

        col.insert(r1).unwrap();
        col.insert(r2).unwrap();
        col.insert(r3).unwrap();
        assert_eq!(col.count(), 3);

        // Erase all vectors from doc-123
        let erased = col.erase_by_source("doc-123").unwrap();
        assert_eq!(erased.len(), 2);
        assert_eq!(col.count(), 1);
    }

    #[test]
    fn test_dimension_mismatch() {
        let mut col = Collection::new("test", 64, Distance::Cosine).unwrap();
        let record = make_record(128, None);
        assert!(col.insert(record).is_err());
    }
}
