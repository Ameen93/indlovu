//! HTTP request handlers for the Indlovu API.

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use indlovu_core::traits::{ErasureSupport, VectorStore};
use indlovu_core::types::VectorRecord;
use serde::{Deserialize, Serialize};

use crate::state::AppState;
use indlovu_compliance::RetentionPolicy;

// ── Request / Response types ────────────────────────────────────

#[derive(Deserialize)]
pub struct CreateCollectionRequest {
    pub name: String,
    pub dimensions: usize,
    #[serde(default)]
    pub contains_pii: bool,
}

#[derive(Deserialize)]
pub struct InsertRequest {
    pub vector: Vec<f32>,
    #[serde(default)]
    pub metadata: serde_json::Value,
    pub source_document_id: Option<String>,
    #[serde(default)]
    pub contains_pii: bool,
}

#[derive(Deserialize)]
pub struct SearchRequest {
    pub vector: Vec<f32>,
    #[serde(default = "default_top_k")]
    pub top_k: usize,
}

fn default_top_k() -> usize {
    10
}

#[derive(Deserialize)]
pub struct EraseRequest {
    pub source_document_id: String,
}

#[derive(Deserialize)]
pub struct ConversionRequest {
    pub variant: String,
    pub cta_id: String,
}

#[derive(Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    fn ok(data: T) -> Json<Self> {
        Json(Self {
            success: true,
            data: Some(data),
            error: None,
        })
    }

    fn err(msg: impl Into<String>) -> (StatusCode, Json<Self>) {
        (
            StatusCode::BAD_REQUEST,
            Json(Self {
                success: false,
                data: None,
                error: Some(msg.into()),
            }),
        )
    }
}

// ── Handlers ────────────────────────────────────────────────────

pub async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "version": env!("CARGO_PKG_VERSION"),
        "name": "indlovu"
    }))
}

pub async fn create_collection(
    State(state): State<AppState>,
    Json(req): Json<CreateCollectionRequest>,
) -> Result<Json<ApiResponse<String>>, (StatusCode, Json<ApiResponse<String>>)> {
    let policy = if req.contains_pii {
        RetentionPolicy {
            contains_pii: true,
            ..Default::default()
        }
    } else {
        RetentionPolicy::default()
    };

    state
        .create_collection(&req.name, req.dimensions, policy)
        .map_err(ApiResponse::err)?;

    Ok(ApiResponse::ok(format!(
        "Collection '{}' created",
        req.name
    )))
}

pub async fn insert_vector(
    State(state): State<AppState>,
    Path(collection_name): Path<String>,
    Json(req): Json<InsertRequest>,
) -> Result<Json<ApiResponse<String>>, (StatusCode, Json<ApiResponse<String>>)> {
    let collections = state.collections.read().unwrap();
    let col = collections
        .get(&collection_name)
        .ok_or_else(|| ApiResponse::<String>::err("Collection not found"))?;

    let record = VectorRecord::new(
        req.vector,
        req.metadata,
        req.source_document_id,
        req.contains_pii,
    );

    let id = col
        .write()
        .unwrap()
        .insert(record)
        .map_err(|e| ApiResponse::<String>::err(e.to_string()))?;

    Ok(ApiResponse::ok(id.to_string()))
}

pub async fn search_vectors(
    State(state): State<AppState>,
    Path(collection_name): Path<String>,
    Json(req): Json<SearchRequest>,
) -> Result<
    Json<ApiResponse<Vec<serde_json::Value>>>,
    (StatusCode, Json<ApiResponse<Vec<serde_json::Value>>>),
> {
    let collections = state.collections.read().unwrap();
    let col = collections
        .get(&collection_name)
        .ok_or_else(|| ApiResponse::<Vec<serde_json::Value>>::err("Collection not found"))?;

    let results = col
        .read()
        .unwrap()
        .search(&req.vector, req.top_k, None)
        .map_err(|e| ApiResponse::<Vec<serde_json::Value>>::err(e.to_string()))?;

    let results: Vec<serde_json::Value> = results
        .into_iter()
        .map(|r| {
            serde_json::json!({
                "id": r.record.id,
                "distance": r.distance,
                "metadata": r.record.metadata,
            })
        })
        .collect();

    Ok(ApiResponse::ok(results))
}

pub async fn erase_by_source(
    State(state): State<AppState>,
    Path(collection_name): Path<String>,
    Json(req): Json<EraseRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiResponse<serde_json::Value>>)>
{
    let collections = state.collections.read().unwrap();
    let col = collections
        .get(&collection_name)
        .ok_or_else(|| ApiResponse::<serde_json::Value>::err("Collection not found"))?;

    let deleted_ids = col
        .write()
        .unwrap()
        .erase_by_source(&req.source_document_id)
        .map_err(|e| ApiResponse::<serde_json::Value>::err(e.to_string()))?;

    Ok(ApiResponse::ok(serde_json::json!({
        "source_document_id": req.source_document_id,
        "deleted_count": deleted_ids.len(),
        "deleted_ids": deleted_ids,
    })))
}

pub async fn capture_conversion(
    State(state): State<AppState>,
    Json(req): Json<ConversionRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiResponse<serde_json::Value>>)>
{
    if req.variant.trim().is_empty() || req.cta_id.trim().is_empty() {
        return Err(ApiResponse::err("variant and cta_id are required"));
    }

    state.record_conversion(req.variant.clone(), req.cta_id.clone());

    Ok(ApiResponse::ok(serde_json::json!({
        "recorded": true,
        "variant": req.variant,
        "cta_id": req.cta_id,
    })))
}

pub async fn conversion_results(
    State(state): State<AppState>,
) -> Json<ApiResponse<serde_json::Value>> {
    let summary = state.conversion_summary();
    ApiResponse::ok(serde_json::json!({
        "total_events": summary.total_events,
        "by_variant": summary.by_variant,
        "by_cta": summary.by_cta,
    }))
}
