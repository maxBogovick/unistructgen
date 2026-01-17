//! Schema Registry Server

mod api;
mod db;
mod config;

use anyhow::Result;
use axum::{
    Router,
    routing::{get, post},
};
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;
use tower_http::cors::{CorsLayer, Any};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::api::AppState;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "schema_registry_server=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    dotenvy::dotenv().ok();
    let config = config::Config::from_env()?;

    // Connect to database
    tracing::info!("Connecting to database...");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await?;

    // Run migrations
    tracing::info!("Running migrations...");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;

    // Create shared state
    let state = AppState::new(pool);

    // Build router
    let app = Router::new()
        // Health check
        .route("/health", get(api::health))

        // Schema endpoints
        .route("/api/schemas", get(api::schemas::list_schemas))
        .route("/api/schemas", post(api::schemas::create_schema))
        .route("/api/schemas/:name", get(api::schemas::get_schema))
        .route("/api/schemas/:name/versions", get(api::schemas::list_versions))
        .route("/api/schemas/:name/:version", get(api::schemas::get_schema_version))

        // Diff endpoints
        .route("/api/diff/:name", post(api::diff::compare_versions))

        // Generation endpoints
        .route("/api/generate", post(api::generate::generate_code))

        // Stats endpoints
        .route("/api/stats", get(api::stats::get_stats))

        // Teams endpoints
        .route("/api/teams", get(api::teams::list_teams))
        .route("/api/teams", post(api::teams::create_team))

        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any))
        .with_state(state);

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
