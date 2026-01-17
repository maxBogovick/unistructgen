//! API handlers

pub mod schemas;
pub mod diff;
pub mod generate;
pub mod stats;
pub mod teams;

use axum::{
    extract::State,
    http::StatusCode,
    Json,
    response::IntoResponse,
};
use schema_registry_common::ApiResponse;
use sqlx::PgPool;
use std::sync::Arc;

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
}

impl AppState {
    pub fn new(pool: PgPool) -> Arc<Self> {
        Arc::new(Self { pool })
    }
}

/// Health check endpoint
pub async fn health() -> impl IntoResponse {
    (StatusCode::OK, Json(ApiResponse::success("healthy")))
}

/// Helper to convert errors to responses
pub(crate) fn error_response(status: StatusCode, message: impl Into<String>) -> impl IntoResponse {
    (
        status,
        Json(ApiResponse::<()>::error(message.into())),
    )
}
