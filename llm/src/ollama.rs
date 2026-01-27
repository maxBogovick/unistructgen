use crate::{LlmClient, CompletionRequest, Result, LlmError, Role, LlmStream};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;
use futures_util::StreamExt;

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

    fn build_body(&self, request: CompletionRequest, stream: bool) -> serde_json::Value {
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

        let mut options = serde_json::Map::new();
        if let Some(temp) = request.temperature {
            options.insert("temperature".to_string(), json!(temp));
        }

        let mut body = json!({
            "model": self.model,
            "messages": messages,
            "stream": stream,
            "options": options
        });

        if let Some(schema) = &request.response_schema {
            body.as_object_mut().unwrap().insert("format".to_string(), json!("json"));
            let schema_instruction = format!(
                "\nYou must output valid JSON that strictly matches this schema:\n{}", 
                serde_json::to_string_pretty(schema).unwrap_or_default()
            );
            
            // Prepend system message with schema
            let mut msgs = body["messages"].as_array().unwrap().clone();
            msgs.insert(0, json!({
                "role": "system",
                "content": format!("Response must be JSON.\n{}", schema_instruction)
            }));
            body["messages"] = json!(msgs);
        }

        body
    }
}

#[async_trait]
impl LlmClient for OllamaClient {
    async fn complete(&self, request: CompletionRequest) -> Result<String> {
        let url = format!("{}/api/chat", self.base_url);
        let body = self.build_body(request, false);

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

    async fn complete_stream(&self, request: CompletionRequest) -> Result<LlmStream> {
        let url = format!("{}/api/chat", self.base_url);
        let body = self.build_body(request, true);

        let response = self.client.post(&url)
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(LlmError::Api(format!("Ollama error: {}", error_text)));
        }

        // Parse JSON stream
        let stream = response.bytes_stream().map(|item| {
            match item {
                Ok(bytes) => {
                    // Ollama sends JSON objects line by line
                    let text = String::from_utf8_lossy(&bytes);
                    let mut tokens = Vec::new();
                    for line in text.lines() {
                        if let Ok(val) = serde_json::from_str::<serde_json::Value>(line) {
                            if let Some(token) = val["message"]["content"].as_str() {
                                tokens.push(Ok(token.to_string()));
                            }
                        }
                    }
                    futures_util::stream::iter(tokens)
                }
                Err(e) => futures_util::stream::iter(vec![Err(LlmError::Network(e))]),
            }
        }).flatten();

        Ok(Box::pin(stream))
    }

    fn model(&self) -> &str {
        &self.model
    }
}