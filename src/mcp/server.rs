use std::sync::Arc;
use serde_json::{json, Value};
use crate::core::{Context, ToolRegistry};
use crate::mcp::protocol::*;

/// The MCP Server implementation logic
pub struct McpServer {
    registry: Arc<ToolRegistry>,
    context: Context,
    name: String,
    version: String,
}

impl McpServer {
    pub fn new(registry: Arc<ToolRegistry>, context: Context, name: &str, version: &str) -> Self {
        Self {
            registry,
            context,
            name: name.to_string(),
            version: version.to_string(),
        }
    }

    /// Handle a JSON-RPC request and return an optional response (None for notifications that don't need reply)
    pub async fn handle_request(&self, req: JsonRpcRequest) -> Option<JsonRpcResponse> {
        // If it's a notification (no ID), we usually don't send a response unless there's a logic error
        // But for MCP, some client messages might be notifications.
        // However, initialize/listTools/callTool all have IDs usually.
        let id = req.id.clone();
        
        // Helper to construct response
        let make_resp = |result: Value| Some(JsonRpcResponse {
            jsonrpc: JSONRPC_VERSION.to_string(),
            result: Some(result),
            error: None,
            id: id.clone(),
        });
        
        let make_err = |err: JsonRpcError| Some(JsonRpcResponse {
            jsonrpc: JSONRPC_VERSION.to_string(),
            result: None,
            error: Some(err),
            id: id.clone(),
        });

        match req.method.as_str() {
            "initialize" => {
                if let Some(params_val) = req.params {
                    let _params: InitializeParams = match serde_json::from_value(params_val) {
                        Ok(p) => p,
                        Err(e) => return make_err(JsonRpcError::invalid_params(e.to_string())),
                    };
                    
                    // In a real implementation, we might check protocol version compatibility here.
                    
                    let result = InitializeResult {
                        protocol_version: "2024-11-05".to_string(), // Latest known MCP version
                        capabilities: ServerCapabilities {
                            tools: Some(ToolCapabilities {
                                list_changed: Some(false), // We don't support dynamic tool addition yet
                            }),
                            ..Default::default()
                        },
                        server_info: Implementation {
                            name: self.name.clone(),
                            version: self.version.clone(),
                        },
                    };
                    
                    match serde_json::to_value(result) {
                        Ok(v) => make_resp(v),
                        Err(e) => make_err(JsonRpcError::internal_error(e.to_string())),
                    }
                } else {
                    make_err(JsonRpcError::invalid_params("Missing params for initialize"))
                }
            }
            "notifications/initialized" => {
                // Client acknowledging initialization. No response needed.
                None
            }
            "tools/list" => {
                let defs = self.registry.get_definitions();
                let tools: Vec<McpTool> = defs.into_iter().map(|def| {
                    // OpenAI format: { "type": "function", "function": { "name": "...", "description": "...", "parameters": { ... } } }
                    // We need to extract inner function part
                    let func = def.get("function").unwrap_or(&def); // Fallback if structure is different
                    
                    let name = func.get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_string();
                        
                    let description = func.get("description")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                        
                    let input_schema = func.get("parameters")
                        .cloned()
                        .unwrap_or(json!({"type": "object", "properties": {}}));

                    McpTool {
                        name,
                        description,
                        input_schema,
                    }
                }).collect();

                let result = ListToolsResult {
                    tools,
                    next_cursor: None,
                };

                match serde_json::to_value(result) {
                    Ok(v) => make_resp(v),
                    Err(e) => make_err(JsonRpcError::internal_error(e.to_string())),
                }
            }
            "tools/call" => {
                if let Some(params_val) = req.params {
                    let params: CallToolParams = match serde_json::from_value(params_val) {
                        Ok(p) => p,
                        Err(e) => return make_err(JsonRpcError::invalid_params(e.to_string())),
                    };

                    let args_str = params.arguments.to_string();
                    
                    match self.registry.execute(&params.name, &args_str, &self.context).await {
                        Ok(output) => {
                            let result = CallToolResult {
                                content: vec![Content::Text { text: output }],
                                is_error: false,
                            };
                            match serde_json::to_value(result) {
                                Ok(v) => make_resp(v),
                                Err(e) => make_err(JsonRpcError::internal_error(e.to_string())),
                            }
                        },
                        Err(e) => {
                            // Execution error is returned as a valid result with isError: true
                            // to allow the LLM to see the error message.
                            let result = CallToolResult {
                                content: vec![Content::Text { text: e.to_string() }],
                                is_error: true,
                            };
                            match serde_json::to_value(result) {
                                Ok(v) => make_resp(v),
                                Err(e) => make_err(JsonRpcError::internal_error(e.to_string())),
                            }
                        }
                    }
                } else {
                    make_err(JsonRpcError::invalid_params("Missing params for tools/call"))
                }
            }
            "ping" => {
                make_resp(json!({}))
            }
            _ => {
                // Check if it starts with known prefixes or just return MethodNotFound
                // For notifications, we might just ignore
                if id.is_some() {
                    make_err(JsonRpcError::method_not_found())
                } else {
                    None
                }
            }
        }
    }
}
