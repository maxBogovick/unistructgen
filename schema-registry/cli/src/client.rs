//! HTTP client for Schema Registry API

use anyhow::{Context, Result};
use reqwest::{Client, Response};
use schema_registry_common::ApiResponse;
use serde::de::DeserializeOwned;

pub struct RegistryClient {
    base_url: String,
    client: Client,
}

impl RegistryClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            client: Client::new(),
        }
    }

    async fn handle_response<T: DeserializeOwned>(response: Response) -> Result<T> {
        let status = response.status();
        let api_response: ApiResponse<T> = response
            .json()
            .await
            .context("Failed to parse response")?;

        if api_response.success {
            api_response
                .data
                .context("Response succeeded but has no data")
        } else {
            anyhow::bail!(
                "API error ({}): {}",
                status,
                api_response.error.unwrap_or_else(|| "Unknown error".to_string())
            )
        }
    }

    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let response = self.client.get(&url).send().await?;
        Self::handle_response(response).await
    }

    pub async fn post<T: DeserializeOwned, B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let response = self.client.post(&url).json(body).send().await?;
        Self::handle_response(response).await
    }
}
