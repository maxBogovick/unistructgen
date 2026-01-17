//! Statistics endpoints

use axum::{
    extract::State,
    http::StatusCode,
    Json,
    response::IntoResponse,
};
use schema_registry_common::ApiResponse;
use std::sync::Arc;

use super::{AppState, error_response};
use crate::db;

/// Get registry statistics
pub async fn get_stats(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match db::get_stats(&state.pool).await {
        Ok(stats) => (StatusCode::OK, Json(ApiResponse::success(stats))),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    }
}
