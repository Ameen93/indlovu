//! Indlovu Server — HTTP API for the privacy-first vector database.

use indlovu_server::{create_router, AppState};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "indlovu=debug,tower_http=debug".into()),
        )
        .init();

    let state = AppState::new();
    let app = create_router(state);

    let addr = "0.0.0.0:6333";
    tracing::info!("🐘 Indlovu server listening on {}", addr);

    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
