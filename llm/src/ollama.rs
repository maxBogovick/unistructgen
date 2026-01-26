use crate::{LlmClient, CompletionRequest, Result, LlmError, Role};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;

pub struct OllamaClient {
    base_url: String,
    model: String,
    client: Client,
}

impl OllamaClient {
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            base_url: "http://localhost:11434".to_string(),
            model: model.into(),
            client: Client::new(),
        }
    }

    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }
}

#[async_trait]
impl LlmClient for OllamaClient {
    async fn complete(&self, request: CompletionRequest) -> Result<String> {
        let url = format!("{}/api/chat", self.base_url);
        
        // Convert internal messages to Ollama format
        let messages: Vec<serde_json::Value> = request.messages.iter()
            .map(|m| json!({
                "role": match m.role {
                    Role::System => "system",
                    Role::User => "user",
                    Role::Assistant => "assistant",
                },
                "content": m.content
            }))
            .collect();

        // Handle JSON Schema: If present, we append instructions to System Prompt
        // Note: Ollama has native 'format: json', but for specific schemas, 
        // explicit instruction + format: json is often robust enough for Llama 3.
        let mut options = serde_json::Map::new();
        if let Some(temp) = request.temperature {
            options.insert("temperature".to_string(), json!(temp));
        }

        let mut body = json!({
            "model": self.model,
            "messages": messages,
            "stream": false,
            "options": options
        });

        // Enforce JSON mode if schema is provided
        if let Some(schema) = &request.response_schema {
            // 1. Enable JSON mode
            body.as_object_mut().unwrap().insert("format".to_string(), json!("json"));

            // 2. Inject Schema into System Prompt (if exists) or add new System Prompt
            let schema_instruction = format!(
                "\nYou must output valid JSON that strictly matches this schema:\n{}", 
                serde_json::to_string_pretty(schema).unwrap_or_default()
            );

            // Find system message or create one
            let has_system = messages.iter().any(|m| m["role"] == "system");
            if has_system {
                // Modify existing (tricky with serde_json::Value iteration), simpler to just prepend a specific instruction
                 let mut msgs = body["messages"].as_array().unwrap().clone();
                 msgs.insert(0, json!({
                     "role": "system",
                     "content": format!("Response must be JSON.\n{}", schema_instruction)
                 }));
                 body["messages"] = json!(msgs);
            } else {
                 let mut msgs = body["messages"].as_array().unwrap().clone();
                 msgs.insert(0, json!({
                     "role": "system",
                     "content": format!("Response must be JSON.\n{}", schema_instruction)
                 }));
                 body["messages"] = json!(msgs);
            }
        }

        let response = self.client.post(&url)
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(LlmError::Api(format!("Ollama error: {}", error_text)));
        }

        let resp_json: serde_json::Value = response.json().await?;
        
        let content = resp_json["message"]["content"]
            .as_str()
            .ok_or_else(|| LlmError::Api("Empty response content from Ollama".to_string()))?;

        Ok(content.to_string())
    }

    fn model(&self) -> &str {
        &self.model
    }
}
