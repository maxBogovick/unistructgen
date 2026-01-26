use crate::{LlmClient, CompletionRequest, Result, LlmError, Role};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;
use std::env;

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
}

#[async_trait]
impl LlmClient for OpenAiClient {
    async fn complete(&self, request: CompletionRequest) -> Result<String> {
        let url = format!("{}/chat/completions", self.base_url);

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
        });

        if let Some(temp) = request.temperature {
            body.as_object_mut().unwrap().insert("temperature".to_string(), json!(temp));
        }

        if let Some(schema) = &request.response_schema {
            // OpenAI Structured Outputs (2024-08-06)
            // https://platform.openai.com/docs/guides/structured-outputs
            body.as_object_mut().unwrap().insert("response_format".to_string(), json!({
                "type": "json_schema",
                "json_schema": {
                    "name": "response_schema", // Arbitrary name
                    "strict": true,
                    "schema": schema
                }
            }));
        } else {
            // Default to json_object if not strict schema but still generic JSON? 
            // Or just text. For now, text.
        }

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

    fn model(&self) -> &str {
        &self.model
    }
}
