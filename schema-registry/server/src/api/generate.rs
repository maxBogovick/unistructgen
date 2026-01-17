//! Code generation endpoints

use axum::{
    extract::State,
    http::StatusCode,
    Json,
    response::IntoResponse,
};
use schema_registry_common::{ApiResponse, GenerationRequest, GenerationResult, GenerationTarget, GeneratedFile};
use std::sync::Arc;

use super::{AppState, error_response};
use crate::db;

/// Generate code from schema
pub async fn generate_code(
    State(state): State<Arc<AppState>>,
    Json(req): Json<GenerationRequest>,
) -> impl IntoResponse {
    // Get schema
    let schema = match db::get_schema(&state.pool, &req.schema_name, &req.version).await {
        Ok(Some(s)) => s,
        Ok(None) => return error_response(StatusCode::NOT_FOUND, "Schema not found"),
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    };

    // Generate for each target
    let mut results = Vec::new();
    for target in &req.targets {
        match generate_for_target(&schema.content, *target) {
            Ok(files) => {
                results.push(GenerationResult {
                    target: *target,
                    files,
                });

                // Record generation event
                let _ = db::record_generation(
                    &state.pool,
                    schema.id,
                    target.as_str(),
                    "system", // TODO: get from auth
                )
                .await;
            }
            Err(e) => {
                return error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Generation failed for {}: {}", target, e),
                );
            }
        }
    }

    (StatusCode::OK, Json(ApiResponse::success(results)))
}

fn generate_for_target(
    _content: &str,
    target: GenerationTarget,
) -> Result<Vec<GeneratedFile>, String> {
    // Placeholder implementation
    // TODO: Integrate with UniStructGen
    Ok(vec![GeneratedFile {
        path: format!("generated/mod.{}", get_extension(target)),
        content: format!("// Generated code for {}\n", target),
    }])
}

fn get_extension(target: GenerationTarget) -> &'static str {
    match target {
        GenerationTarget::Rust => "rs",
        GenerationTarget::TypeScript => "ts",
        GenerationTarget::Python => "py",
        GenerationTarget::Go => "go",
        GenerationTarget::Java => "java",
        GenerationTarget::Kotlin => "kt",
        GenerationTarget::Swift => "swift",
        GenerationTarget::CSharp => "cs",
    }
}
