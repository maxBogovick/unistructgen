//! # Blog API - UniStructGen Showcase
//!
//! This example demonstrates the power of UniStructGen:
//! - OpenAPI spec → Rust types (with validation!)
//! - Type-safe API handlers
//! - Automatic request/response validation
//! - Zero boilerplate
//!
//! ## The Magic
//!
//! All types below are auto-generated from `blog-api.yaml`
//! We just write business logic!

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, RwLock},
};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;
use validator::Validate;

// =============================================================================
// AUTO-GENERATED TYPES (from blog-api.yaml)
// =============================================================================
//
// In production, these would be generated via:
// ```rust
// unistructgen::openapi_to_rust! {
//     file = "blog-api.yaml"
// }
// ```
//
// For this demo, we define them manually to show what gets generated:

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Post {
    pub id: Uuid,

    #[validate(length(min = 5, max = 200))]
    pub title: String,

    #[validate(length(min = 10, max = 10000))]
    pub content: String,

    #[validate(length(min = 2, max = 100), regex = "^[a-zA-Z0-9_-]+$")]
    pub author: String,

    #[validate(length(max = 10))]
    pub tags: Vec<String>,

    pub status: PostStatus,

    #[validate(range(min = 0))]
    pub view_count: u32,

    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PostStatus {
    Draft,
    Published,
    Archived,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreatePost {
    #[validate(length(min = 5, max = 200))]
    pub title: String,

    #[validate(length(min = 10, max = 10000))]
    pub content: String,

    #[validate(length(min = 2, max = 100), regex = "^[a-zA-Z0-9_-]+$")]
    pub author: String,

    #[serde(default)]
    #[validate(length(max = 10))]
    pub tags: Vec<String>,

    #[serde(default)]
    pub status: Option<PostStatus>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdatePost {
    #[validate(length(min = 5, max = 200))]
    pub title: Option<String>,

    #[validate(length(min = 10, max = 10000))]
    pub content: Option<String>,

    #[validate(length(max = 10))]
    pub tags: Option<Vec<String>>,

    pub status: Option<PostStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Comment {
    pub id: Uuid,
    pub post_id: Uuid,

    #[validate(length(min = 2, max = 100), regex = "^[a-zA-Z0-9_-]+$")]
    pub author: String,

    #[validate(length(min = 1, max = 1000))]
    pub content: String,

    pub created_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateComment {
    #[validate(length(min = 2, max = 100), regex = "^[a-zA-Z0-9_-]+$")]
    pub author: String,

    #[validate(length(min = 1, max = 1000))]
    pub content: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ListParams {
    #[validate(range(min = 1, max = 100))]
    #[serde(default = "default_limit")]
    pub limit: u32,

    #[validate(range(min = 0))]
    #[serde(default)]
    pub offset: u32,
}

fn default_limit() -> u32 {
    10
}

#[derive(Debug, Serialize)]
pub struct PostsList {
    pub posts: Vec<Post>,
    pub total: usize,
}

#[derive(Debug, Serialize)]
pub struct ApiError {
    pub error: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

// =============================================================================
// APPLICATION STATE
// =============================================================================

#[derive(Clone)]
pub struct AppState {
    posts: Arc<RwLock<HashMap<Uuid, Post>>>,
    comments: Arc<RwLock<HashMap<Uuid, Vec<Comment>>>>,
}

impl AppState {
    fn new() -> Self {
        Self {
            posts: Arc::new(RwLock::new(HashMap::new())),
            comments: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

// =============================================================================
// HANDLERS - Pure Business Logic! 🎉
// =============================================================================

async fn list_posts(
    State(state): State<AppState>,
    Query(params): Query<ListParams>,
) -> Result<Json<PostsList>, (StatusCode, Json<ApiError>)> {
    // Validate query params
    params.validate().map_err(validation_error)?;

    let posts = state.posts.read().unwrap();
    let mut all_posts: Vec<_> = posts.values().cloned().collect();
    all_posts.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    let total = all_posts.len();
    let offset = params.offset as usize;
    let limit = params.limit as usize;

    let posts = all_posts
        .into_iter()
        .skip(offset)
        .take(limit)
        .collect();

    Ok(Json(PostsList { posts, total }))
}

async fn create_post(
    State(state): State<AppState>,
    Json(create): Json<CreatePost>,
) -> Result<(StatusCode, Json<Post>), (StatusCode, Json<ApiError>)> {
    // Validate input
    create.validate().map_err(validation_error)?;

    let now = Utc::now();
    let post = Post {
        id: Uuid::new_v4(),
        title: create.title,
        content: create.content,
        author: create.author,
        tags: create.tags,
        status: create.status.unwrap_or(PostStatus::Draft),
        view_count: 0,
        created_at: now,
        updated_at: now,
    };

    state.posts.write().unwrap().insert(post.id, post.clone());

    Ok((StatusCode::CREATED, Json(post)))
}

async fn get_post(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Post>, (StatusCode, Json<ApiError>)> {
    let mut posts = state.posts.write().unwrap();

    let post = posts
        .get_mut(&id)
        .ok_or_else(|| not_found_error("Post not found"))?;

    // Increment view count
    post.view_count += 1;
    let post = post.clone();

    Ok(Json(post))
}

async fn update_post(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(update): Json<UpdatePost>,
) -> Result<Json<Post>, (StatusCode, Json<ApiError>)> {
    // Validate input
    update.validate().map_err(validation_error)?;

    let mut posts = state.posts.write().unwrap();

    let post = posts
        .get_mut(&id)
        .ok_or_else(|| not_found_error("Post not found"))?;

    // Update fields
    if let Some(title) = update.title {
        post.title = title;
    }
    if let Some(content) = update.content {
        post.content = content;
    }
    if let Some(tags) = update.tags {
        post.tags = tags;
    }
    if let Some(status) = update.status {
        post.status = status;
    }

    post.updated_at = Utc::now();

    Ok(Json(post.clone()))
}

async fn delete_post(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<ApiError>)> {
    let mut posts = state.posts.write().unwrap();

    posts
        .remove(&id)
        .ok_or_else(|| not_found_error("Post not found"))?;

    // Also remove comments
    state.comments.write().unwrap().remove(&id);

    Ok(StatusCode::NO_CONTENT)
}

async fn get_comments(
    State(state): State<AppState>,
    Path(post_id): Path<Uuid>,
) -> Result<Json<Vec<Comment>>, (StatusCode, Json<ApiError>)> {
    // Check if post exists
    state
        .posts
        .read()
        .unwrap()
        .get(&post_id)
        .ok_or_else(|| not_found_error("Post not found"))?;

    let comments = state.comments.read().unwrap();
    let post_comments = comments.get(&post_id).cloned().unwrap_or_default();

    Ok(Json(post_comments))
}

async fn add_comment(
    State(state): State<AppState>,
    Path(post_id): Path<Uuid>,
    Json(create): Json<CreateComment>,
) -> Result<(StatusCode, Json<Comment>), (StatusCode, Json<ApiError>)> {
    // Validate input
    create.validate().map_err(validation_error)?;

    // Check if post exists
    state
        .posts
        .read()
        .unwrap()
        .get(&post_id)
        .ok_or_else(|| not_found_error("Post not found"))?;

    let comment = Comment {
        id: Uuid::new_v4(),
        post_id,
        author: create.author,
        content: create.content,
        created_at: Utc::now(),
    };

    state
        .comments
        .write()
        .unwrap()
        .entry(post_id)
        .or_default()
        .push(comment.clone());

    Ok((StatusCode::CREATED, Json(comment)))
}

// =============================================================================
// ERROR HELPERS
// =============================================================================

fn validation_error(errors: validator::ValidationErrors) -> (StatusCode, Json<ApiError>) {
    (
        StatusCode::BAD_REQUEST,
        Json(ApiError {
            error: "ValidationError".to_string(),
            message: "Request validation failed".to_string(),
            details: Some(serde_json::to_value(errors).unwrap()),
        }),
    )
}

fn not_found_error(message: &str) -> (StatusCode, Json<ApiError>) {
    (
        StatusCode::NOT_FOUND,
        Json(ApiError {
            error: "NotFound".to_string(),
            message: message.to_string(),
            details: None,
        }),
    )
}

// =============================================================================
// MAIN - Application Setup
// =============================================================================

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "blog_api=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Create app state
    let state = AppState::new();

    // Build router
    let app = Router::new()
        // Posts endpoints
        .route("/posts", get(list_posts).post(create_post))
        .route(
            "/posts/:id",
            get(get_post).put(update_post).delete(delete_post),
        )
        // Comments endpoints
        .route("/posts/:id/comments", get(get_comments).post(add_comment))
        // Middleware
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("🚀 Blog API server starting on {}", addr);
    tracing::info!("📖 API Documentation: http://localhost:3000/docs (coming soon!)");
    tracing::info!("");
    tracing::info!("✨ All types auto-generated from blog-api.yaml");
    tracing::info!("✅ Automatic validation from OpenAPI constraints");
    tracing::info!("🎯 Type-safe handlers with zero boilerplate");
    tracing::info!("");
    tracing::info!("Try:");
    tracing::info!("  curl -X POST http://localhost:3000/posts \\");
    tracing::info!("    -H 'Content-Type: application/json' \\");
    tracing::info!("    -d '{{\"title\":\"Hello World\",\"content\":\"This is my first post!\",\"author\":\"john_doe\"}}'");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
