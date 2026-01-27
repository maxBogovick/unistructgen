use std::collections::HashMap;
use serde_json::Value;
use thiserror::Error;
use std::sync::Arc;
use async_trait::async_trait;
use crate::context::Context;
use futures_util::future::join_all;

#[derive(Debug, Error)]
pub enum ToolError {
    #[error("Tool not found: {0}")]
    NotFound(String),
    #[error("Argument parsing error: {0}")]
    ArgumentError(#[from] serde_json::Error),
    #[error("Execution error: {0}")]
    ExecutionError(String),
    #[error("Context error: {0}")]
    ContextError(String),
}

pub type ToolResult = Result<String, ToolError>;

/// Represents a single tool call from an LLM
#[derive(Debug, Clone)]
pub struct ToolCall {
    pub name: String,
    pub arguments: String,
}

/// Trait that all AI tools must implement.
/// This will typically be implemented automatically by the #[ai_tool] macro.
#[async_trait]
pub trait AiTool: Send + Sync {
    /// The name of the tool (e.g., "calculate_shipping")
    fn name(&self) -> &str;

    /// A description of what the tool does
    fn description(&self) -> &str;

    /// The JSON Schema of the arguments this tool accepts
    fn parameters_schema(&self) -> Value;

    /// Execute the tool with the given JSON arguments and context
    async fn call(&self, arguments_json: &str, context: &Context) -> ToolResult;
}

/// A registry to manage and execute tools.
#[derive(Default, Clone)]
pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn AiTool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    /// Register a new tool
    pub fn register(&mut self, tool: impl AiTool + 'static) {
        self.tools.insert(tool.name().to_string(), Arc::new(tool));
    }

    /// Get definitions for all registered tools in OpenAI format
    pub fn get_definitions(&self) -> Vec<Value> {
        self.tools.values().map(|tool| {
            serde_json::json!({
                "type": "function",
                "function": {
                    "name": tool.name(),
                    "description": tool.description(),
                    "parameters": tool.parameters_schema(),
                }
            })
        }).collect()
    }

    /// Execute a tool by name with JSON arguments
    pub async fn execute(&self, name: &str, arguments_json: &str, context: &Context) -> ToolResult {
        if let Some(tool) = self.tools.get(name) {
            tool.call(arguments_json, context).await
        } else {
            Err(ToolError::NotFound(name.to_string()))
        }
    }

    /// Execute multiple tools in parallel
    pub async fn execute_batch(&self, calls: Vec<ToolCall>, context: &Context) -> Vec<(String, ToolResult)> {
        let futures = calls.into_iter().map(|call| async move {
            let name = call.name.clone();
            let result = self.execute(&name, &call.arguments, context).await;
            (name, result)
        });

        join_all(futures).await
    }
    
    /// Check if a tool exists
    pub fn has_tool(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }
}