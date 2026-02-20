#[cfg(feature = "sse")]
use std::net::SocketAddr;
#[cfg(feature = "sse")]
use std::sync::Arc;
#[cfg(feature = "sse")]
use tokio::sync::mpsc;
#[cfg(feature = "sse")]
use axum::{
    routing::{get, post},
    Router,
    response::{sse::{Event, Sse}, IntoResponse},
    extract::{State, Query},
    Json,
};
#[cfg(feature = "sse")]
use futures_util::stream::{Stream, StreamExt};
#[cfg(feature = "sse")]
use dashmap::DashMap;
#[cfg(feature = "sse")]
use serde::Deserialize;
#[cfg(feature = "sse")]
use uuid::Uuid;
#[cfg(feature = "sse")]
use crate::core::{Context, ToolRegistry};
#[cfg(feature = "sse")]
use crate::mcp::protocol::{JsonRpcRequest, JsonRpcResponse};
#[cfg(feature = "sse")]
use crate::mcp::server::McpServer;

#[cfg(feature = "sse")]
struct AppState {
    server: Arc<McpServer>,
    // Map session_id -> channel sender
    sessions: Arc<DashMap<String, mpsc::UnboundedSender<Event>>>,
}

#[cfg(feature = "sse")]
#[derive(Deserialize)]
struct SseQuery {
    #[serde(default)]
    id: Option<String>,
}

#[cfg(feature = "sse")]
/// Start an MCP server over SSE (Server-Sent Events)
pub async fn serve_sse(
    registry: Arc<ToolRegistry>,
    context: Context,
    addr: SocketAddr,
) -> anyhow::Result<()> {
    let server = Arc::new(McpServer::new(
        registry,
        context,
        "unistructgen-mcp-sse",
        env!("CARGO_PKG_VERSION"),
    ));
    
    let sessions = Arc::new(DashMap::new());
    
    let state = Arc::new(AppState {
        server,
        sessions,
    });

    let app = Router::new()
        .route("/sse", get(sse_handler))
        .route("/messages", post(message_handler))
        .with_state(state);

    println!("MCP SSE server listening on http://{}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

#[cfg(feature = "sse")]
async fn sse_handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SseQuery>,
) -> impl IntoResponse {
    let session_id = query.id.unwrap_or_else(|| Uuid::new_v4().to_string());
    
    let (tx, rx) = mpsc::unbounded_channel();
    state.sessions.insert(session_id.clone(), tx.clone());
    
    // Send the endpoint URL as the first event, as per MCP SSE spec
    // The client needs to know where to send POST requests.
    // We assume the client knows the base URL, so we send the relative path "/messages"
    // appended with the session ID query param.
    let endpoint_event = Event::default()
        .event("endpoint")
        .data(format!("/messages?id={}", session_id));
        
    let _ = tx.send(endpoint_event);
    
    let stream = tokio_stream::wrappers::UnboundedReceiverStream::new(rx)
        .map(|e| Ok::<Event, std::convert::Infallible>(e));

    Sse::new(stream).keep_alive(axum::response::sse::KeepAlive::default())
}

#[cfg(feature = "sse")]
async fn message_handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SseQuery>,
    Json(req): Json<JsonRpcRequest>,
) -> impl IntoResponse {
    let session_id = match query.id {
        Some(id) => id,
        None => return (axum::http::StatusCode::BAD_REQUEST, "Missing session id").into_response(),
    };
    
    if !state.sessions.contains_key(&session_id) {
        return (axum::http::StatusCode::NOT_FOUND, "Session not found").into_response();
    }
    
    let server = state.server.clone();
    let sessions = state.sessions.clone();
    
    // Spawn processing to not block the request handler
    tokio::spawn(async move {
        if let Some(response) = server.handle_request(req).await {
            if let Some(tx) = sessions.get(&session_id) {
                if let Ok(json) = serde_json::to_string(&response) {
                    let event = Event::default()
                        .event("message")
                        .data(json);
                    let _ = tx.send(event);
                }
            }
        }
    });

    // Return accepted
    axum::http::StatusCode::ACCEPTED.into_response()
}
