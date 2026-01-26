use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;
use unistructgen_core::{CodeGenerator, Parser};
use unistructgen_codegen::{RenderOptions, RustRenderer};
use unistructgen_openapi_parser::{OpenApiParser, OpenApiParserOptions};

pub struct ClientGenerator {
    pub spec_path: Option<PathBuf>,
    pub spec_url: Option<String>,
    pub output_dir: PathBuf,
    pub client_name: String,
    pub include_examples: bool,
}

impl ClientGenerator {
    pub fn generate(&self) -> Result<()> {
        println!("🚀 UniStructGen Client Generator");
        println!("================================\n");

        // Load OpenAPI spec
        let spec_content = if let Some(path) = &self.spec_path {
            println!("📖 Reading OpenAPI spec from: {}", path.display());
            fs::read_to_string(path)
                .with_context(|| format!("Failed to read spec file: {}", path.display()))?
        } else if let Some(url) = &self.spec_url {
            println!("🌐 Fetching OpenAPI spec from: {}", url);
            self.fetch_from_url(url)?
        } else {
            anyhow::bail!("Either --spec or --url must be provided");
        };

        // Parse OpenAPI spec
        println!("🔍 Parsing OpenAPI specification...");
        let options = OpenApiParserOptions::builder()
            .generate_client(false) // We'll generate our own custom client
            .generate_validation(true)
            .derive_serde(true)
            .build();

        let mut parser = OpenApiParser::new(options);
        let ir_module = parser
            .parse(&spec_content)
            .context("Failed to parse OpenAPI specification")?;

        println!("✓ Found {} types", ir_module.types.len());

        // Generate types
        println!("\n📝 Generating Rust types...");
        let generator = RustRenderer::new(RenderOptions {
            add_header: true,
            add_clippy_allows: true,
        });

        let types_code = generator
            .generate(&ir_module)
            .context("Failed to generate types")?;

        // Create output directory
        fs::create_dir_all(&self.output_dir).with_context(|| {
            format!(
                "Failed to create output directory: {}",
                self.output_dir.display()
            )
        })?;

        // Write types.rs
        let types_path = self.output_dir.join("types.rs");
        fs::write(&types_path, types_code)
            .with_context(|| format!("Failed to write types file: {}", types_path.display()))?;
        println!("✓ Generated: {}", types_path.display());

        // Generate client.rs
        println!("\n🔧 Generating HTTP client...");
        let client_code = self.generate_client_code(&self.client_name);
        let client_path = self.output_dir.join("client.rs");
        fs::write(&client_path, client_code).with_context(|| {
            format!("Failed to write client file: {}", client_path.display())
        })?;
        println!("✓ Generated: {}", client_path.display());

        // Generate lib.rs
        let lib_code = self.generate_lib_code(&self.client_name);
        let lib_path = self.output_dir.join("lib.rs");
        fs::write(&lib_path, lib_code)
            .with_context(|| format!("Failed to write lib file: {}", lib_path.display()))?;
        println!("✓ Generated: {}", lib_path.display());

        // Generate Cargo.toml
        let cargo_toml = self.generate_cargo_toml(&self.client_name);
        let cargo_path = self.output_dir.join("Cargo.toml");
        fs::write(&cargo_path, cargo_toml)
            .with_context(|| format!("Failed to write Cargo.toml: {}", cargo_path.display()))?;
        println!("✓ Generated: {}", cargo_path.display());

        // Generate examples if requested
        if self.include_examples {
            println!("\n📚 Generating examples...");
            let examples_dir = self.output_dir.join("examples");
            fs::create_dir_all(&examples_dir)?;

            let example_code = self.generate_example_code(&self.client_name);
            let example_path = examples_dir.join("main");
            fs::write(&example_path, example_code)?;
            println!("✓ Generated: {}", example_path.display());
        }

        // Generate README
        let readme = self.generate_readme(&self.client_name);
        let readme_path = self.output_dir.join("../../README_OLD.md");
        fs::write(&readme_path, readme)?;
        println!("✓ Generated: {}", readme_path.display());

        println!("\n🎉 Client generation complete!");
        println!("\n📦 Your client is ready at: {}", self.output_dir.display());
        println!("\n🚀 Quick start:");
        println!("   cd {}", self.output_dir.display());
        println!("   cargo build");
        if self.include_examples {
            println!("   cargo run --example basic");
        }
        println!();

        Ok(())
    }

    fn fetch_from_url(&self, _url: &str) -> Result<String> {
        // TODO: Implement URL fetching with ureq
        Err(anyhow::anyhow!("URL fetching not implemented yet for client generator"))
    }

    fn generate_client_code(&self, client_name: &str) -> String {
        format!(
            r#"//! HTTP Client for {}
//!
//! Generated by UniStructGen - https://github.com/your-repo/unistructgen

use reqwest::{{Client as ReqwestClient, Method, RequestBuilder}};
use serde::{{Deserialize, Serialize}};
use std::time::Duration;

use crate::types::*;

/// Error type for client operations
#[derive(Debug, thiserror::Error)]
pub enum ClientError {{
    #[error("HTTP request failed: {{0}}")]
    Http(#[from] reqwest::Error),

    #[error("Serialization error: {{0}}")]
    Serialization(#[from] serde_json::Error),

    #[error("API error: {{status}} - {{message}}")]
    Api {{ status: u16, message: String }},

    #[error("Validation error: {{0}}")]
    Validation(#[from] validator::ValidationErrors),
}}

pub type Result<T> = std::result::Result<T, ClientError>;

/// Configuration for the {} client
#[derive(Debug, Clone)]
pub struct ClientConfig {{
    /// Base URL for the API
    pub base_url: String,

    /// Optional API key or token
    pub auth_token: Option<String>,

    /// Request timeout in seconds
    pub timeout_secs: u64,

    /// Custom user agent
    pub user_agent: Option<String>,
}}

impl Default for ClientConfig {{
    fn default() -> Self {{
        Self {{
            base_url: "https://api.example.com".to_string(),
            auth_token: None,
            timeout_secs: 30,
            user_agent: Some("unistructgen-client/1.0".to_string()),
        }}
    }}
}}

/// {} API Client
///
/// # Examples
///
/// ```no_run
/// use {}_client::{{Client, ClientConfig}};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {{
///     let config = ClientConfig {{
///         base_url: "https://api.example.com".to_string(),
///         auth_token: Some("your-token".to_string()),
///         ..Default::default()
///     }};
///
///     let client = Client::new(config);
///     // Use the client...
///     Ok(())
/// }}
/// ```
pub struct Client {{
    http: ReqwestClient,
    config: ClientConfig,
}}

impl Client {{
    /// Create a new client with the given configuration
    pub fn new(config: ClientConfig) -> Self {{
        let mut builder = ReqwestClient::builder()
            .timeout(Duration::from_secs(config.timeout_secs));

        if let Some(user_agent) = &config.user_agent {{
            builder = builder.user_agent(user_agent);
        }}

        let http = builder.build().expect("Failed to build HTTP client");

        Self {{ http, config }}
    }}

    /// Create a new client with default configuration
    pub fn with_base_url(base_url: impl Into<String>) -> Self {{
        let config = ClientConfig {{
            base_url: base_url.into(),
            ..Default::default()
        }};
        Self::new(config)
    }}

    /// Set authentication token
    pub fn with_auth_token(mut self, token: impl Into<String>) -> Self {{
        self.config.auth_token = Some(token.into());
        self
    }}

    /// Build a request with common headers
    fn request(&self, method: Method, path: &str) -> RequestBuilder {{
        let url = format!("{{}}/{{}}", self.config.base_url.trim_end_matches('/'), path.trim_start_matches('/'));
        let mut req = self.http.request(method, url);

        if let Some(token) = &self.config.auth_token {{
            req = req.header("Authorization", format!("Bearer {{}}", token));
        }}

        req = req.header("Content-Type", "application/json");

        req
    }}

    /// Execute a GET request
    async fn get<T: for<'de> Deserialize<'de>>(&self, path: &str) -> Result<T> {{
        let response = self.request(Method::GET, path).send().await?;

        if !response.status().is_success() {{
            return Err(ClientError::Api {{
                status: response.status().as_u16(),
                message: response.text().await.unwrap_or_default(),
            }});
        }}

        Ok(response.json().await?)
    }}

    /// Execute a POST request
    async fn post<T: Serialize, R: for<'de> Deserialize<'de>>(
        &self,
        path: &str,
        body: &T,
    ) -> Result<R> {{
        let response = self
            .request(Method::POST, path)
            .json(body)
            .send()
            .await?;

        if !response.status().is_success() {{
            return Err(ClientError::Api {{
                status: response.status().as_u16(),
                message: response.text().await.unwrap_or_default(),
            }});
        }}

        Ok(response.json().await?)
    }}

    /// Execute a PUT request
    async fn put<T: Serialize, R: for<'de> Deserialize<'de>>(
        &self,
        path: &str,
        body: &T,
    ) -> Result<R> {{
        let response = self.request(Method::PUT, path).json(body).send().await?;

        if !response.status().is_success() {{
            return Err(ClientError::Api {{
                status: response.status().as_u16(),
                message: response.text().await.unwrap_or_default(),
            }});
        }}

        Ok(response.json().await?)
    }}

    /// Execute a DELETE request
    async fn delete(&self, path: &str) -> Result<()> {{
        let response = self.request(Method::DELETE, path).send().await?;

        if !response.status().is_success() {{
            return Err(ClientError::Api {{
                status: response.status().as_u16(),
                message: response.text().await.unwrap_or_default(),
            }});
        }}

        Ok(())
    }}

    // Add your API methods here
    // Example:
    //
    // pub async fn get_user(&self, user_id: &str) -> Result<User> {{
    //     self.get(&format!("users/{{}}", user_id)).await
    // }}
}}

#[cfg(test)]
mod tests {{
    use super::*;

    #[test]
    fn test_client_creation() {{
        let client = Client::with_base_url("https://api.example.com");
        assert_eq!(client.config.base_url, "https://api.example.com");
    }}

    #[test]
    fn test_client_with_auth() {{
        let client = Client::with_base_url("https://api.example.com")
            .with_auth_token("test-token");
        assert_eq!(client.config.auth_token, Some("test-token".to_string()));
    }}
}}
"#,
            client_name, client_name, client_name, client_name.to_lowercase()
        )
    }

    fn generate_lib_code(&self, _client_name: &str) -> String {
        r#"//! API Client Library
//!
//! This library provides a type-safe client for interacting with the API.
//! All types are generated from the OpenAPI specification.
//!
//! # Example
//!
//! ```no_run
//! use api_client::{Client, ClientConfig};
//!
//! #[tokio::main]
//! async fn main() {
//!     let client = Client::with_base_url("https://api.example.com")
//!         .with_auth_token("your-api-token");
//!
//!     // Use the client methods here
//! }
//! ```

pub mod client;
pub mod types;

pub use client::{Client, ClientConfig, ClientError, Result};
pub use types::*;

// Re-export commonly used types
pub use validator::Validate;
"#
        .to_string()
    }

    fn generate_cargo_toml(&self, client_name: &str) -> String {
        format!(
            r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"
authors = ["Generated by UniStructGen"]
description = "Type-safe API client generated from OpenAPI specification"
license = "MIT OR Apache-2.0"

[dependencies]
reqwest = {{ version = "0.11", features = ["json"] }}
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"
tokio = {{ version = "1.0", features = ["full"] }}
thiserror = "1.0"
validator = {{ version = "0.18", features = ["derive"] }}
chrono = {{ version = "0.4", features = ["serde"] }}

[dev-dependencies]
tokio-test = "0.4"

[[example]]
name = "basic"
path = "examples/main"
"#,
            client_name.to_lowercase().replace(' ', "-")
        )
    }

    fn generate_example_code(&self, client_name: &str) -> String {
        format!(
            r#"//! Basic usage example for {} client
//!
//! Run with: cargo run --example basic

use {}_client::{{Client, ClientConfig}};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {{
    // Create client with configuration
    let config = ClientConfig {{
        base_url: "https://api.example.com".to_string(),
        auth_token: std::env::var("API_TOKEN").ok(),
        timeout_secs: 30,
        user_agent: Some("my-app/1.0".to_string()),
    }};

    let client = Client::new(config);

    println!("✓ Client created successfully");
    println!("  Base URL: {{}}", client.config.base_url);
    println!("  Auth: {{}}", if client.config.auth_token.is_some() {{ "✓ Configured" }} else {{ "✗ Not configured" }});

    // Add your API calls here
    // Example:
    // let user = client.get_user("123").await?;
    // println!("User: {{:?}}", user);

    Ok(())
}}
"#,
            client_name,
            client_name.to_lowercase().replace(' ', "-")
        )
    }

    fn generate_readme(&self, client_name: &str) -> String {
        format!(
            r#"# {} Client

Type-safe Rust client library generated from OpenAPI specification using [UniStructGen](https://github.com/your-repo/unistructgen).

## Features

- ✨ **Fully type-safe** - All types generated from OpenAPI spec
- 🔒 **Validation** - Built-in request/response validation
- 🚀 **Async/await** - Modern async Rust with Tokio
- 📝 **Well-documented** - Auto-generated docs from OpenAPI
- 🎯 **Easy to use** - Clean, intuitive API

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
{} = {{ path = "." }}
tokio = {{ version = "1.0", features = ["full"] }}
```

## Quick Start

```rust
use {}_client::{{Client, ClientConfig}};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {{
    // Create client
    let client = Client::with_base_url("https://api.example.com")
        .with_auth_token("your-api-token");

    // Make API calls
    // let result = client.some_method().await?;
    // println!("{{:?}}", result);

    Ok(())
}}
```

## Configuration

```rust
use {}_client::ClientConfig;

let config = ClientConfig {{
    base_url: "https://api.example.com".to_string(),
    auth_token: Some("your-token".to_string()),
    timeout_secs: 30,
    user_agent: Some("my-app/1.0".to_string()),
}};

let client = Client::new(config);
```

## Examples

Run the examples:

```bash
# Basic usage
cargo run --example basic

# Set API token
API_TOKEN=your-token cargo run --example basic
```

## Development

```bash
# Build
cargo build

# Run tests
cargo test

# Check code
cargo clippy
```

## Generated with UniStructGen

This client was generated using [UniStructGen](https://github.com/your-repo/unistructgen) -
a powerful Rust code generator that creates type-safe code from JSON, OpenAPI specs, and more.

### Regenerating the client

```bash
unistructgen client --spec your-spec.yaml --output . --name "{}"
```

## License

MIT OR Apache-2.0
"#,
            client_name,
            client_name.to_lowercase().replace(' ', "-"),
            client_name.to_lowercase().replace(' ', "-"),
            client_name.to_lowercase().replace(' ', "-"),
            client_name
        )
    }
}
