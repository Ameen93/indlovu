//! Route definitions for the Indlovu HTTP API.

use axum::routing::{get, post};
use axum::Router;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use crate::handlers;
use crate::state::AppState;

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(handlers::health))
        .route("/collections", post(handlers::create_collection))
        .route("/collections/{name}/vectors", post(handlers::insert_vector))
        .route("/collections/{name}/search", post(handlers::search_vectors))
        .route("/collections/{name}/erase", post(handlers::erase_by_source))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state)
}
