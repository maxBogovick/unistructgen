use std::io::{self, BufRead};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use unistructgen_core::{Context, ToolRegistry};
use crate::protocol::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};
use crate::server::McpServer;

/// Start an MCP server over stdio
///
/// This function blocks until stdin is closed.
/// It reads JSON-RPC requests from stdin (line-delimited) and writes responses to stdout.
/// Logging should be configured to write to stderr to avoid corrupting the protocol stream.
pub async fn serve_stdio(registry: Arc<ToolRegistry>, context: Context) -> anyhow::Result<()> {
    // Ensure we are not writing logs to stdout
    // tracing_subscriber should be configured by the caller to use stderr
    
    let server = Arc::new(McpServer::new(
        registry,
        context,
        "unistructgen-mcp",
        env!("CARGO_PKG_VERSION"),
    ));

    let stdin = tokio::io::stdin();
    let mut stdout = tokio::io::stdout();
    let mut reader = BufReader::new(stdin);
    let mut line = String::new();

    loop {
        line.clear();
        let bytes_read = reader.read_line(&mut line).await?;
        if bytes_read == 0 {
            break; // EOF
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Parse request
        let req: Result<JsonRpcRequest, _> = serde_json::from_str(trimmed);
        match req {
            Ok(request) => {
                let server_clone = server.clone();
                // Process in background to allow concurrency?
                // For stdio, we typically process sequentially or spawn. 
                // Let's spawn to prevent blocking reading loop, but we need to serialize writes to stdout.
                // However, simple agents often expect sequential processing. 
                // Let's do sequential for simplicity and safety first.
                
                if let Some(response) = server_clone.handle_request(request).await {
                    let resp_str = serde_json::to_string(&response)?;
                    stdout.write_all(resp_str.as_bytes()).await?;
                    stdout.write_all(b"\n").await?;
                    stdout.flush().await?;
                }
            }
            Err(e) => {
                // If we can't parse it as a request, it might be invalid JSON or just noise.
                // We should try to report a ParseError if it looks like JSON-RPC but failed.
                // But blindly reporting error to stdout might break things if it wasn't intended for us.
                // We'll log to stderr.
                eprintln!("Failed to parse JSON-RPC request: {}", e);
                eprintln!("Raw input: {}", trimmed);
                
                // Construct a parse error response if we can
                let error_resp = JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: None,
                    error: Some(JsonRpcError::new(-32700, format!("Parse error: {}", e))),
                    id: None,
                };
                let resp_str = serde_json::to_string(&error_resp)?;
                stdout.write_all(resp_str.as_bytes()).await?;
                stdout.write_all(b"\n").await?;
                stdout.flush().await?;
            }
        }
    }

    Ok(())
}
