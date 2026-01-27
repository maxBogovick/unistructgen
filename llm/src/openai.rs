use crate::{LlmClient, CompletionRequest, Result, LlmError, Role, LlmStream};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;
use std::env;
use futures_util::StreamExt;

pub struct OpenAiClient {
    api_key: String,
    model: String,
    client: Client,
    base_url: String,
}

impl OpenAiClient {
    pub fn new(model: impl Into<String>) -> Result<Self> {
        let api_key = env::var("OPENAI_API_KEY")
            .map_err(|_| LlmError::Config("OPENAI_API_KEY not found in environment".to_string()))?;
        
        Ok(Self {
            api_key,
            model: model.into(),
            client: Client::new(),
            base_url: "https://api.openai.com/v1".to_string(),
        })
    }

    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
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

        let mut body = json!({
            "model": self.model,
            "messages": messages,
            "stream": stream,
        });

        if let Some(temp) = request.temperature {
            body.as_object_mut().unwrap().insert("temperature".to_string(), json!(temp));
        }

        if let Some(schema) = &request.response_schema {
            body.as_object_mut().unwrap().insert("response_format".to_string(), json!({
                "type": "json_schema",
                "json_schema": {
                    "name": "response_schema",
                    "strict": true,
                    "schema": schema
                }
            }));
        }

        body
    }
}

#[async_trait]
impl LlmClient for OpenAiClient {
    async fn complete(&self, request: CompletionRequest) -> Result<String> {
        let url = format!("{}/chat/completions", self.base_url);
        let body = self.build_body(request, false);

        let response = self.client.post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(LlmError::Api(format!("OpenAI error: {}", error_text)));
        }

        let resp_json: serde_json::Value = response.json().await?;
        let content = resp_json["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| LlmError::Api("Empty response content from OpenAI".to_string()))?;

        Ok(content.to_string())
    }

    async fn complete_stream(&self, request: CompletionRequest) -> Result<LlmStream> {
        let url = format!("{}/chat/completions", self.base_url);
        let body = self.build_body(request, true);

        let response = self.client.post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(LlmError::Api(format!("OpenAI error: {}", error_text)));
        }

        let stream = response.bytes_stream().map(|item| {
            match item {
                Ok(bytes) => {
                    let text = String::from_utf8_lossy(&bytes);
                    let mut tokens = Vec::new();
                    for line in text.lines() {
                        let line = line.trim();
                        if line.is_empty() || line == "data: [DONE]" { continue; }
                        
                        if let Some(data) = line.strip_prefix("data: ") {
                            if let Ok(val) = serde_json::from_str::<serde_json::Value>(data) {
                                if let Some(token) = val["choices"][0]["delta"]["content"].as_str() {
                                    tokens.push(Ok(token.to_string()));
                                }
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