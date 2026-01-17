//! Team management endpoints

use axum::{
    extract::State,
    http::StatusCode,
    Json,
    response::IntoResponse,
};
use schema_registry_common::ApiResponse;
use serde::Deserialize;
use std::sync::Arc;

use super::{AppState, error_response};
use crate::db;

/// List all teams
pub async fn list_teams(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match db::list_teams(&state.pool).await {
        Ok(teams) => (StatusCode::OK, Json(ApiResponse::success(teams))),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateTeamRequest {
    pub name: String,
    pub description: Option<String>,
}

/// Create new team
pub async fn create_team(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateTeamRequest>,
) -> impl IntoResponse {
    match db::create_team(&state.pool, req.name, req.description).await {
        Ok(team) => (StatusCode::CREATED, Json(ApiResponse::success(team))),
        Err(e) => {
            if e.to_string().contains("duplicate key") {
                error_response(StatusCode::CONFLICT, "Team already exists")
            } else {
                error_response(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
            }
        }
    }
}
