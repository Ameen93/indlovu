//! # Indlovu Server
//!
//! Axum-based HTTP API server for the Indlovu vector database.

pub mod handlers;
pub mod routes;
pub mod state;

pub use routes::create_router;
pub use state::AppState;
