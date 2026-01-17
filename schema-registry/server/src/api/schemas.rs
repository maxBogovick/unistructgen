//! Schema management endpoints

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
    response::IntoResponse,
};
use schema_registry_common::{ApiResponse, Schema, SchemaFormat};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use super::{AppState, error_response};
use crate::db;

#[derive(Debug, Deserialize)]
pub struct ListSchemasQuery {
    team: Option<String>,
}

/// List all schemas
pub async fn list_schemas(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListSchemasQuery>,
) -> impl IntoResponse {
    match db::list_schemas(&state.pool, params.team.as_deref()).await {
        Ok(schemas) => (StatusCode::OK, Json(ApiResponse::success(schemas))),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateSchemaRequest {
    pub name: String,
    pub version: String,
    pub format: SchemaFormat,
    pub team: String,
    pub description: Option<String>,
    pub content: String,
    pub created_by: String,
}

/// Create new schema
pub async fn create_schema(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateSchemaRequest>,
) -> impl IntoResponse {
    // Validate content based on format
    if let Err(e) = validate_schema_content(&req.content, req.format) {
        return error_response(StatusCode::BAD_REQUEST, format!("Invalid schema: {}", e));
    }

    match db::create_schema(
        &state.pool,
        req.name,
        req.version,
        req.format,
        &req.team,
        req.description,
        req.content,
        req.created_by,
    )
    .await
    {
        Ok(schema) => (StatusCode::CREATED, Json(ApiResponse::success(schema))),
        Err(e) => {
            if e.to_string().contains("duplicate key") {
                error_response(StatusCode::CONFLICT, "Schema version already exists")
            } else {
                error_response(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
            }
        }
    }
}

/// Get schema by name (latest version)
pub async fn get_schema(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    match db::get_latest_schema(&state.pool, &name).await {
        Ok(Some(schema)) => (StatusCode::OK, Json(ApiResponse::success(schema))),
        Ok(None) => error_response(StatusCode::NOT_FOUND, "Schema not found"),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    }
}

/// Get schema by name and version
pub async fn get_schema_version(
    State(state): State<Arc<AppState>>,
    Path((name, version)): Path<(String, String)>,
) -> impl IntoResponse {
    match db::get_schema(&state.pool, &name, &version).await {
        Ok(Some(schema)) => (StatusCode::OK, Json(ApiResponse::success(schema))),
        Ok(None) => error_response(StatusCode::NOT_FOUND, "Schema version not found"),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    }
}

/// List versions for a schema
pub async fn list_versions(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    match db::list_versions(&state.pool, &name).await {
        Ok(versions) => (StatusCode::OK, Json(ApiResponse::success(versions))),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    }
}

/// Validate schema content
fn validate_schema_content(content: &str, format: SchemaFormat) -> Result<(), String> {
    match format {
        SchemaFormat::OpenApi => {
            serde_yaml::from_str::<serde_json::Value>(content)
                .map_err(|e| format!("Invalid YAML: {}", e))?;
            Ok(())
        }
        SchemaFormat::JsonSchema => {
            serde_json::from_str::<serde_json::Value>(content)
                .map_err(|e| format!("Invalid JSON: {}", e))?;
            Ok(())
        }
        _ => Ok(()), // Skip validation for other formats for now
    }
}
