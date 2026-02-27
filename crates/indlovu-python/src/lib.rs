//! # Indlovu Python Bindings
//!
//! PyO3 bindings providing a Pythonic interface to the Indlovu vector database.
//!
//! ```python
//! from indlovu import Collection
//!
//! db = Collection("my_docs", dimensions=384)
//! db.add(vectors=[[0.1, 0.2, ...]], metadata=[{"source": "doc1"}])
//! results = db.search(query=[0.1, 0.2, ...], top_k=5)
//! db.erase(source_document_id="doc1")  # POPIA/GDPR right-to-erasure
//! ```

use pyo3::prelude::*;

/// Indlovu Python module — privacy-first vector search.
#[pymodule]
fn indlovu(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyCollection>()?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}

/// A vector collection with built-in compliance support.
#[pyclass(name = "Collection")]
struct PyCollection {
    name: String,
    dimensions: usize,
}

#[pymethods]
impl PyCollection {
    #[new]
    #[pyo3(signature = (name, dimensions=384))]
    fn new(name: String, dimensions: usize) -> Self {
        Self { name, dimensions }
    }

    /// Get the collection name.
    #[getter]
    fn name(&self) -> &str {
        &self.name
    }

    /// Get the vector dimensions.
    #[getter]
    fn dimensions(&self) -> usize {
        self.dimensions
    }

    fn __repr__(&self) -> String {
        format!("Collection(name='{}', dimensions={})", self.name, self.dimensions)
    }
}
