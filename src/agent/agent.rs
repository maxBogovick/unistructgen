use crate::core::{Context, ToolRegistry};
use crate::llm::{CompletionRequest, LlmClient, Message};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Represents a single autonomous agent capable of using tools
pub struct Agent {
    name: String,
    client: Arc<dyn LlmClient>,
    registry: Arc<ToolRegistry>,
    context: Context,
    system_prompt: String,
    max_iterations: usize,
    _model: String,
}

#[derive(Default)]
pub struct AgentBuilder {
    name: Option<String>,
    client: Option<Arc<dyn LlmClient>>,
    registry: Option<Arc<ToolRegistry>>,
    context: Option<Context>,
    system_prompt: Option<String>,
    max_iterations: Option<usize>,
}

impl AgentBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn client(mut self, client: Arc<dyn LlmClient>) -> Self {
        self.client = Some(client);
        self
    }

    pub fn tools(mut self, registry: Arc<ToolRegistry>) -> Self {
        self.registry = Some(registry);
        self
    }

    pub fn context(mut self, context: Context) -> Self {
        self.context = Some(context);
        self
    }

    pub fn system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(prompt.into());
        self
    }

    pub fn max_iterations(mut self, max: usize) -> Self {
        self.max_iterations = Some(max);
        self
    }

    pub fn build(self) -> anyhow::Result<Agent> {
        let client = self.client.ok_or_else(|| anyhow::anyhow!("LlmClient is required"))?;
        let model = client.model().to_string();

        Ok(Agent {
            name: self.name.unwrap_or_else(|| "Agent".to_string()),
            client,
            registry: self.registry.unwrap_or_else(|| Arc::new(ToolRegistry::new())),
            context: self.context.unwrap_or_else(Context::new),
            system_prompt: self.system_prompt.unwrap_or_else(|| "You are a helpful assistant.".to_string()),
            max_iterations: self.max_iterations.unwrap_or(10),
            _model: model,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct ToolCallRequest {
    tool: String,
    args: Value,
}

impl Agent {
    pub fn builder() -> AgentBuilder {
        AgentBuilder::new()
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    /// Run the agent loop with a specific task
    pub async fn run(&self, task: &str) -> anyhow::Result<String> {
        info!("[{}] Starting task: {}", self.name, task);

        let mut messages = vec![
            Message::system(&self.build_system_prompt()),
            Message::user(task),
        ];

        let mut iterations = 0;
        let tool_regex = Regex::new(r"(?s)Action:\s*({.*?})").unwrap();

        while iterations < self.max_iterations {
            iterations += 1;
            debug!("[{}] Iteration {}/{}", self.name, iterations, self.max_iterations);

            // 1. Call LLM
            let request = CompletionRequest {
                messages: messages.clone(),
                temperature: Some(0.0), // Deterministic for tool use
                max_tokens: None,
                response_schema: None,
            };

            let response_text = self.client.complete(request).await?;
            info!("[{}] Thought: {}", self.name, response_text);
            
            // Add assistant response to history
            messages.push(Message::assistant(&response_text));

            // 2. Parse for Tool Calls (Action: { ... })
            if let Some(caps) = tool_regex.captures(&response_text) {
                let json_str = &caps[1];
                match serde_json::from_str::<ToolCallRequest>(json_str) {
                    Ok(call) => {
                        info!("[{}] Calling tool: {}", self.name, call.tool);
                        
                        let args_str = serde_json::to_string(&call.args)?;
                        
                        // 3. Execute Tool
                        let result = self.registry.execute(&call.tool, &args_str, &self.context).await;
                        
                        let output = match result {
                            Ok(ok) => ok,
                            Err(e) => format!("Error: {}", e),
                        };

                        info!("[{}] Tool Output: {}", self.name, output);

                        // 4. Feed back result
                        messages.push(Message::user(&format!("Tool '{}' result: {}", call.tool, output)));
                    }
                    Err(e) => {
                        warn!("[{}] Failed to parse tool arguments: {}", self.name, e);
                        messages.push(Message::user(&format!("System Error: Invalid JSON for Action. Ensure strict JSON format. Error: {}", e)));
                    }
                }
            } else {
                // No tool call found, assume this is the final answer or a question to user
                // If the LLM thinks it's done, it usually just speaks without Action.
                // We can return the last response.
                if response_text.contains("Action:") {
                     // It tried but regex failed?
                     warn!("[{}] malformed action detected", self.name);
                } else {
                    return Ok(response_text);
                }
            }
        }

        Ok("Max iterations reached without final answer.".to_string())
    }

    fn build_system_prompt(&self) -> String {
        let tools_def = self.registry.get_definitions();
        let tools_json = serde_json::to_string_pretty(&tools_def).unwrap_or_default();

        format!(
            "{}

            You have access to the following tools:
            {}

            To use a tool, you MUST output a valid JSON object prefixed by 'Action:'.
            Format:
            Action: {{ \"tool\": \"tool_name\", \"args\": {{ \"arg1\": \"value\" }} }}

            If you do not need to use a tool, just respond with text.",
            self.system_prompt,
            tools_json
        )
    }
}
