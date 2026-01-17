//! Schema diff endpoints

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
    response::IntoResponse,
};
use schema_registry_common::{ApiResponse, SchemaDiff};
use serde::Deserialize;
use std::sync::Arc;

use super::{AppState, error_response};
use crate::db;

#[derive(Debug, Deserialize)]
pub struct CompareVersionsRequest {
    pub from_version: String,
    pub to_version: String,
}

/// Compare two versions of a schema
pub async fn compare_versions(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    Json(req): Json<CompareVersionsRequest>,
) -> impl IntoResponse {
    // Get both versions
    let from_schema = match db::get_schema(&state.pool, &name, &req.from_version).await {
        Ok(Some(s)) => s,
        Ok(None) => {
            return error_response(
                StatusCode::NOT_FOUND,
                format!("Version {} not found", req.from_version),
            );
        }
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    };

    let to_schema = match db::get_schema(&state.pool, &name, &req.to_version).await {
        Ok(Some(s)) => s,
        Ok(None) => {
            return error_response(
                StatusCode::NOT_FOUND,
                format!("Version {} not found", req.to_version),
            );
        }
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    };

    // Ensure same format
    if from_schema.format != to_schema.format {
        return error_response(
            StatusCode::BAD_REQUEST,
            "Cannot compare schemas of different formats",
        );
    }

    // Perform diff
    match schema_registry_common::diff::compare_schemas(
        &from_schema.content,
        &to_schema.content,
        from_schema.format,
        req.from_version,
        req.to_version,
    ) {
        Ok(diff) => (StatusCode::OK, Json(ApiResponse::success(diff))),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    }
}
