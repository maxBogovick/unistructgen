use anyhow::{Context, Result};
use openapiv3::{
    OpenAPI, Operation, Parameter, ParameterSchemaOrContent, PathItem, ReferenceOr, RequestBody,
    Response, Schema, SecurityScheme, StatusCode,
};
use std::fs;
use std::path::PathBuf;
use unistructgen::codegen::{RenderOptions, RustRenderer};
use unistructgen::core::{CodeGenerator, IRTypeRef, Parser};
use unistructgen::parsers::openapi::types::{
    extract_type_name_from_ref, openapi_type_to_ir, sanitize_field_name, to_pascal_case,
    to_snake_case,
};
use unistructgen::parsers::openapi::{OpenApiParser, OpenApiParserOptions};
use ureq;

pub struct ClientGenerator {
    pub spec_path: Option<PathBuf>,
    pub spec_url: Option<String>,
    pub output_dir: PathBuf,
    pub client_name: String,
    pub include_examples: bool,
}

struct OperationParam {
    original_name: String,
    rust_name: String,
    rust_type: String,
    required: bool,
}

struct OperationBody {
    rust_type: String,
    required: bool,
    content_type: String,
    kind: BodyKind,
}

struct OperationInfo {
    fn_name: String,
    method: String,
    path: String,
    path_params: Vec<OperationParam>,
    query_params: Vec<OperationParam>,
    header_params: Vec<OperationParam>,
    body: Option<OperationBody>,
    response_type: String,
    response_content_type: Option<String>,
    auth_schemes: Vec<AuthSchemeDef>,
}

#[derive(Clone)]
enum AuthSchemeDef {
    Bearer,
    Basic,
    ApiKeyHeader(String),
    ApiKeyQuery(String),
}

#[derive(Clone, Copy)]
enum BodyKind {
    Json,
    Form,
    Text,
    Binary,
    Other,
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
        let openapi_spec = self.parse_openapi_spec(&spec_content)?;

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
        let operations = self.collect_operations(&openapi_spec);
        let client_code = self.generate_client_code(&self.client_name, &operations);
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
            let example_path = examples_dir.join("basic.rs");
            fs::write(&example_path, example_code)?;
            println!("✓ Generated: {}", example_path.display());
        }

        // Generate README
        let readme = self.generate_readme(&self.client_name);
        let readme_path = self.output_dir.join("README.md");
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

    fn fetch_from_url(&self, url: &str) -> Result<String> {
        let response = ureq::get(url)
            .set("User-Agent", "unistructgen-client-generator/0.1")
            .call()
            .with_context(|| format!("Failed to fetch OpenAPI spec from: {}", url))?;

        if response.status() >= 400 {
            anyhow::bail!("Failed to fetch OpenAPI spec: HTTP {}", response.status());
        }

        response
            .into_string()
            .with_context(|| format!("Failed to read OpenAPI spec body from: {}", url))
    }

    fn parse_openapi_spec(&self, input: &str) -> Result<OpenAPI> {
        if let Ok(spec) = serde_json::from_str::<OpenAPI>(input) {
            return Ok(spec);
        }

        serde_yaml::from_str::<OpenAPI>(input)
            .context("Failed to parse OpenAPI spec as JSON or YAML")
    }

    fn collect_operations(&self, spec: &OpenAPI) -> Vec<OperationInfo> {
        let mut operations = Vec::new();

        for (path, path_item_ref) in &spec.paths.paths {
            let path_item = match path_item_ref {
                ReferenceOr::Item(item) => item,
                ReferenceOr::Reference { .. } => continue,
            };

            if let Some(op) = &path_item.get {
                self.push_operation(spec, path, path_item, "get", op, &mut operations);
            }
            if let Some(op) = &path_item.post {
                self.push_operation(spec, path, path_item, "post", op, &mut operations);
            }
            if let Some(op) = &path_item.put {
                self.push_operation(spec, path, path_item, "put", op, &mut operations);
            }
            if let Some(op) = &path_item.delete {
                self.push_operation(spec, path, path_item, "delete", op, &mut operations);
            }
            if let Some(op) = &path_item.patch {
                self.push_operation(spec, path, path_item, "patch", op, &mut operations);
            }
        }

        operations
    }

    fn push_operation(
        &self,
        spec: &OpenAPI,
        path: &str,
        path_item: &PathItem,
        method: &str,
        operation: &Operation,
        out: &mut Vec<OperationInfo>,
    ) {
        let op_name = operation
            .operation_id
            .as_deref()
            .map(|s| s.to_string())
            .unwrap_or_else(|| self.fallback_operation_name(method, path));
        let fn_name = sanitize_field_name(&to_snake_case(&op_name));

        let mut path_params = Vec::new();
        let mut query_params = Vec::new();
        let mut header_params = Vec::new();

        let mut all_params = Vec::new();
        all_params.extend(path_item.parameters.iter());
        all_params.extend(operation.parameters.iter());

        for param_ref in all_params {
            let param = match self.resolve_parameter(spec, param_ref) {
                Some(p) => p,
                None => continue,
            };

            match param {
                Parameter::Path { parameter_data, .. } => {
                    let rust_name = sanitize_field_name(&parameter_data.name);
                    path_params.push(OperationParam {
                        original_name: parameter_data.name.clone(),
                        rust_name,
                        rust_type: "String".to_string(),
                        required: true,
                    });
                }
                Parameter::Query { parameter_data, .. } => {
                    let rust_type =
                        self.parameter_schema_to_rust_type(&parameter_data.format, &parameter_data.name);
                    let rust_name = sanitize_field_name(&parameter_data.name);
                    query_params.push(OperationParam {
                        original_name: parameter_data.name.clone(),
                        rust_name,
                        rust_type,
                        required: parameter_data.required,
                    });
                }
                Parameter::Header { parameter_data, .. } => {
                    let rust_type =
                        self.parameter_schema_to_rust_type(&parameter_data.format, &parameter_data.name);
                    let rust_name = sanitize_field_name(&parameter_data.name);
                    header_params.push(OperationParam {
                        original_name: parameter_data.name.clone(),
                        rust_name,
                        rust_type,
                        required: parameter_data.required,
                    });
                }
                _ => {}
            }
        }

        let body = self.extract_request_body(spec, operation);
        let (response_type, response_content_type) = self.extract_response_type(spec, operation);
        let auth_schemes = self.extract_auth_schemes(spec, operation);

        out.push(OperationInfo {
            fn_name,
            method: method.to_string(),
            path: path.to_string(),
            path_params,
            query_params,
            header_params,
            body,
            response_type,
            response_content_type,
            auth_schemes,
        });
    }

    fn resolve_parameter<'a>(
        &self,
        spec: &'a OpenAPI,
        param_ref: &'a ReferenceOr<Parameter>,
    ) -> Option<&'a Parameter> {
        match param_ref {
            ReferenceOr::Item(param) => Some(param),
            ReferenceOr::Reference { reference } => {
                let name = reference.rsplit('/').next()?;
                let components = spec.components.as_ref()?;
                let resolved = components.parameters.get(name)?;
                match resolved {
                    ReferenceOr::Item(param) => Some(param),
                    ReferenceOr::Reference { .. } => None,
                }
            }
        }
    }

    fn resolve_request_body<'a>(
        &self,
        spec: &'a OpenAPI,
        body_ref: &'a ReferenceOr<RequestBody>,
    ) -> Option<&'a RequestBody> {
        match body_ref {
            ReferenceOr::Item(body) => Some(body),
            ReferenceOr::Reference { reference } => {
                let name = reference.rsplit('/').next()?;
                let components = spec.components.as_ref()?;
                let resolved = components.request_bodies.get(name)?;
                match resolved {
                    ReferenceOr::Item(body) => Some(body),
                    ReferenceOr::Reference { .. } => None,
                }
            }
        }
    }

    fn resolve_response<'a>(
        &self,
        spec: &'a OpenAPI,
        response_ref: &'a ReferenceOr<Response>,
    ) -> Option<&'a Response> {
        match response_ref {
            ReferenceOr::Item(response) => Some(response),
            ReferenceOr::Reference { reference } => {
                let name = reference.rsplit('/').next()?;
                let components = spec.components.as_ref()?;
                let resolved = components.responses.get(name)?;
                match resolved {
                    ReferenceOr::Item(response) => Some(response),
                    ReferenceOr::Reference { .. } => None,
                }
            }
        }
    }

    fn extract_request_body(&self, spec: &OpenAPI, operation: &Operation) -> Option<OperationBody> {
        let body_ref = operation.request_body.as_ref()?;
        let body = self.resolve_request_body(spec, body_ref)?;
        let required = body.required;

        let (content_type, media_type) = self.select_content_type(&body.content, true)?;

        let schema_ref = media_type.schema.as_ref()?;
        let rust_type = self.schema_ref_to_rust_type(schema_ref, Some("RequestBody"));
        let kind = self.body_kind_for(&content_type);

        Some(OperationBody {
            rust_type,
            required,
            content_type,
            kind,
        })
    }

    fn extract_response_type(
        &self,
        spec: &OpenAPI,
        operation: &Operation,
    ) -> (String, Option<String>) {
        let mut best: Option<(u16, &ReferenceOr<Response>)> = None;

        for (status, response_ref) in &operation.responses.responses {
            if !self.is_success_status(status) {
                continue;
            }

            let code = match status {
                StatusCode::Code(code) => *code,
                StatusCode::Range(range) => (range * 100) as u16,
            };

            if best.map_or(true, |(best_code, _)| code < best_code) {
                best = Some((code, response_ref));
            }
        }

        let response_ref = match best {
            Some((_, r)) => r,
            None => return ("serde_json::Value".to_string(), None),
        };

        let response = match self.resolve_response(spec, response_ref) {
            Some(r) => r,
            None => return ("serde_json::Value".to_string(), None),
        };

        if response.content.is_empty() {
            return ("()".to_string(), None);
        }

        let (content_type, media_type) =
            match self.select_content_type(&response.content, false) {
                Some(result) => result,
                None => return ("()".to_string(), None),
            };

        let schema_ref = match media_type.schema.as_ref() {
            Some(schema) => schema,
            None => return ("serde_json::Value".to_string(), Some(content_type)),
        };

        (
            self.schema_ref_to_rust_type(schema_ref, Some("Response")),
            Some(content_type),
        )
    }

    fn extract_auth_schemes(&self, spec: &OpenAPI, operation: &Operation) -> Vec<AuthSchemeDef> {
        let requirements = if let Some(reqs) = &operation.security {
            if reqs.is_empty() {
                return Vec::new();
            }
            reqs
        } else if let Some(reqs) = &spec.security {
            reqs
        } else {
            return Vec::new();
        };

        let mut schemes = Vec::new();
        for requirement in requirements {
            for (scheme_name, _scopes) in requirement {
                if let Some(def) = self.resolve_security_scheme(spec, scheme_name) {
                    if !schemes.iter().any(|s| self.auth_scheme_eq(s, &def)) {
                        schemes.push(def);
                    }
                }
            }
        }

        schemes
    }

    fn resolve_security_scheme(&self, spec: &OpenAPI, name: &str) -> Option<AuthSchemeDef> {
        let components = spec.components.as_ref()?;
        let scheme_ref = components.security_schemes.get(name)?;
        let scheme = match scheme_ref {
            ReferenceOr::Item(s) => s,
            ReferenceOr::Reference { .. } => return None,
        };

        match scheme {
            SecurityScheme::HTTP { scheme, .. } => match scheme.as_str() {
                "bearer" => Some(AuthSchemeDef::Bearer),
                "basic" => Some(AuthSchemeDef::Basic),
                _ => None,
            },
            SecurityScheme::OAuth2 { .. } => Some(AuthSchemeDef::Bearer),
            SecurityScheme::OpenIDConnect { .. } => Some(AuthSchemeDef::Bearer),
            SecurityScheme::APIKey { name, location, .. } => match location {
                openapiv3::APIKeyLocation::Header => {
                    Some(AuthSchemeDef::ApiKeyHeader(name.clone()))
                }
                openapiv3::APIKeyLocation::Query => {
                    Some(AuthSchemeDef::ApiKeyQuery(name.clone()))
                }
                openapiv3::APIKeyLocation::Cookie => None,
            },
        }
    }

    fn auth_scheme_eq(&self, left: &AuthSchemeDef, right: &AuthSchemeDef) -> bool {
        match (left, right) {
            (AuthSchemeDef::Bearer, AuthSchemeDef::Bearer) => true,
            (AuthSchemeDef::Basic, AuthSchemeDef::Basic) => true,
            (AuthSchemeDef::ApiKeyHeader(a), AuthSchemeDef::ApiKeyHeader(b)) => a == b,
            (AuthSchemeDef::ApiKeyQuery(a), AuthSchemeDef::ApiKeyQuery(b)) => a == b,
            _ => false,
        }
    }

    fn is_success_status(&self, status: &StatusCode) -> bool {
        match status {
            StatusCode::Code(code) => (200..300).contains(code),
            StatusCode::Range(range) => *range == 2,
        }
    }

    fn select_content_type<'a>(
        &self,
        content: &'a indexmap::IndexMap<String, openapiv3::MediaType>,
        prefer_request: bool,
    ) -> Option<(String, &'a openapiv3::MediaType)> {
        if content.is_empty() {
            return None;
        }

        if let Some(mt) = content.get("application/json") {
            return Some(("application/json".to_string(), mt));
        }

        let json_like = content
            .iter()
            .find(|(key, _)| key.ends_with("+json"))
            .map(|(key, mt)| (key.clone(), mt));
        if let Some((key, mt)) = json_like {
            return Some((key, mt));
        }

        if prefer_request {
            if let Some(mt) = content.get("application/x-www-form-urlencoded") {
                return Some(("application/x-www-form-urlencoded".to_string(), mt));
            }
            if let Some(mt) = content.get("multipart/form-data") {
                return Some(("multipart/form-data".to_string(), mt));
            }
            if let Some((key, mt)) = content.iter().find(|(key, _)| key.starts_with("text/"))
            {
                return Some((key.clone(), mt));
            }
        }

        content
            .iter()
            .next()
            .map(|(key, mt)| (key.clone(), mt))
    }

    fn body_kind_for(&self, content_type: &str) -> BodyKind {
        let ct = content_type.to_ascii_lowercase();
        if ct == "application/json" || ct.ends_with("+json") {
            BodyKind::Json
        } else if ct.starts_with("application/x-www-form-urlencoded")
            || ct.starts_with("multipart/form-data")
        {
            BodyKind::Form
        } else if ct.starts_with("text/") {
            BodyKind::Text
        } else if ct.starts_with("application/octet-stream") {
            BodyKind::Binary
        } else {
            BodyKind::Other
        }
    }

    fn parameter_schema_to_rust_type(
        &self,
        schema_or_content: &ParameterSchemaOrContent,
        name_hint: &str,
    ) -> String {
        match schema_or_content {
            ParameterSchemaOrContent::Schema(schema_ref) => {
                self.schema_ref_to_rust_type(schema_ref, Some(name_hint))
            }
            ParameterSchemaOrContent::Content(_) => "serde_json::Value".to_string(),
        }
    }

    fn schema_ref_to_rust_type(
        &self,
        schema_ref: &ReferenceOr<Schema>,
        name_hint: Option<&str>,
    ) -> String {
        match schema_ref {
            ReferenceOr::Reference { reference } => extract_type_name_from_ref(reference),
            ReferenceOr::Item(schema) => {
                match openapi_type_to_ir(schema, name_hint).ok() {
                    Some(ir) => self.ir_type_to_rust_type(ir),
                    None => "serde_json::Value".to_string(),
                }
            }
        }
    }

    fn ir_type_to_rust_type(&self, ty: IRTypeRef) -> String {
        match ty {
            IRTypeRef::Primitive(p) => p.rust_type_name().to_string(),
            IRTypeRef::Option(inner) => format!("Option<{}>", self.ir_type_to_rust_type(*inner)),
            IRTypeRef::Vec(inner) => format!("Vec<{}>", self.ir_type_to_rust_type(*inner)),
            IRTypeRef::Named(name) => name,
            IRTypeRef::Map(key, value) => format!(
                "std::collections::HashMap<{}, {}>",
                self.ir_type_to_rust_type(*key),
                self.ir_type_to_rust_type(*value)
            ),
        }
    }

    fn fallback_operation_name(&self, method: &str, path: &str) -> String {
        let mut name = String::new();
        name.push_str(method);
        name.push('_');
        name.push_str(
            &path
                .trim_matches('/')
                .replace('/', "_")
                .replace('{', "by_")
                .replace('}', ""),
        );
        name
    }

    fn generate_client_code(&self, client_name: &str, operations: &[OperationInfo]) -> String {
        let crate_name = self.crate_name(client_name);
        let repo_url = env!("CARGO_PKG_REPOSITORY");
        let operation_types = self.generate_operation_types(operations);
        let operation_methods = self.generate_operation_methods(operations);
        format!(
            r#"//! HTTP Client for {}
//!
//! Generated by UniStructGen - {}

use reqwest::{{Client as ReqwestClient, Method, RequestBuilder}};
use serde::{{Deserialize, Serialize}};
use std::time::Duration;

use crate::types::*;

use std::collections::HashMap;
use base64::Engine;

#[derive(Debug, Clone, Copy)]
enum AuthScheme {{
    Bearer,
    Basic,
    ApiKeyHeader(&'static str),
    ApiKeyQuery(&'static str),
}}

{}

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

    /// Optional bearer token (HTTP Bearer)
    pub bearer_token: Option<String>,

    /// Optional Basic auth credentials
    pub basic_auth: Option<(String, String)>,

    /// API keys sent as headers
    pub api_key_headers: HashMap<String, String>,

    /// API keys sent as query parameters
    pub api_key_queries: HashMap<String, String>,

    /// Request timeout in seconds
    pub timeout_secs: u64,

    /// Custom user agent
    pub user_agent: Option<String>,
}}

impl Default for ClientConfig {{
    fn default() -> Self {{
        Self {{
            base_url: "https://api.example.com".to_string(),
            bearer_token: None,
            basic_auth: None,
            api_key_headers: HashMap::new(),
            api_key_queries: HashMap::new(),
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
/// use {}::{{Client, ClientConfig}};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {{
///     let config = ClientConfig {{
///         base_url: "https://api.example.com".to_string(),
///         bearer_token: Some("your-token".to_string()),
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
    pub fn with_bearer_token(mut self, token: impl Into<String>) -> Self {{
        self.config.bearer_token = Some(token.into());
        self
    }}

    /// Set basic auth credentials
    pub fn with_basic_auth(
        mut self,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> Self {{
        self.config.basic_auth = Some((username.into(), password.into()));
        self
    }}

    /// Set API key header
    pub fn with_api_key_header(
        mut self,
        header_name: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {{
        self.config.api_key_headers.insert(header_name.into(), value.into());
        self
    }}

    /// Set API key query parameter
    pub fn with_api_key_query(
        mut self,
        param_name: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {{
        self.config.api_key_queries.insert(param_name.into(), value.into());
        self
    }}

    /// Access current configuration
    pub fn config(&self) -> &ClientConfig {{
        &self.config
    }}

    /// Build a request with common headers
    fn request(&self, method: Method, path: &str) -> RequestBuilder {{
        let url = format!("{{}}/{{}}", self.config.base_url.trim_end_matches('/'), path.trim_start_matches('/'));
        self.http.request(method, url)
    }}

    fn apply_auth(&self, mut req: RequestBuilder, schemes: &[AuthScheme]) -> RequestBuilder {{
        for scheme in schemes {{
            match scheme {{
                AuthScheme::Bearer => {{
                    if let Some(token) = &self.config.bearer_token {{
                        req = req.header("Authorization", format!("Bearer {{}}", token));
                    }}
                }}
                AuthScheme::Basic => {{
                    if let Some((username, password)) = &self.config.basic_auth {{
                        let encoded = base64::engine::general_purpose::STANDARD
                            .encode(format!("{{}}:{{}}", username, password));
                        req = req.header("Authorization", format!("Basic {{}}", encoded));
                    }}
                }}
                AuthScheme::ApiKeyHeader(name) => {{
                    if let Some(value) = self.config.api_key_headers.get(*name) {{
                        req = req.header(*name, value);
                    }}
                }}
                AuthScheme::ApiKeyQuery(name) => {{
                    if let Some(value) = self.config.api_key_queries.get(*name) {{
                        req = req.query(&[(*name, value.as_str())]);
                    }}
                }}
            }}
        }}

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

{}
}}

#[cfg(test)]
mod tests {{
    use super::*;

    #[test]
    fn test_client_creation() {{
        let client = Client::with_base_url("https://api.example.com");
        assert_eq!(client.config().base_url, "https://api.example.com");
    }}

    #[test]
    fn test_client_with_auth() {{
        let client = Client::with_base_url("https://api.example.com")
            .with_bearer_token("test-token");
        assert_eq!(client.config().bearer_token, Some("test-token".to_string()));
    }}
}}
"#,
            client_name, repo_url, operation_types, client_name, client_name, crate_name, operation_methods
        )
    }

    fn generate_operation_types(&self, operations: &[OperationInfo]) -> String {
        let mut output = String::new();

        for op in operations {
            let base = to_pascal_case(&op.fn_name);

            if !op.query_params.is_empty() {
                let struct_name = format!("{}Query", base);
                output.push_str(&self.render_param_struct(&struct_name, &op.query_params));
                output.push('\n');
            }

            if !op.header_params.is_empty() {
                let struct_name = format!("{}Headers", base);
                output.push_str(&self.render_param_struct(&struct_name, &op.header_params));
                output.push('\n');
            }
        }

        output
    }

    fn render_param_struct(&self, name: &str, params: &[OperationParam]) -> String {
        let mut output = String::new();
        output.push_str("#[derive(Debug, Serialize)]\n");
        output.push_str(&format!("pub struct {} {{\n", name));

        for param in params {
            if !param.required {
                output.push_str("    #[serde(skip_serializing_if = \"Option::is_none\")]\n");
            }
            if param.original_name != param.rust_name {
                output.push_str(&format!(
                    "    #[serde(rename = \"{}\")]\n",
                    param.original_name
                ));
            }

            let ty = if param.required {
                param.rust_type.clone()
            } else {
                format!("Option<{}>", param.rust_type)
            };

            output.push_str(&format!("    pub {}: {},\n", param.rust_name, ty));
        }

        output.push_str("}\n");
        output
    }

    fn generate_operation_methods(&self, operations: &[OperationInfo]) -> String {
        let mut output = String::new();

        for op in operations {
            let base = to_pascal_case(&op.fn_name);
            let query_name = format!("{}Query", base);
            let headers_name = format!("{}Headers", base);

            output.push_str(&format!(
                "    /// {} {}\n",
                op.method.to_uppercase(),
                op.path
            ));
            output.push_str(&format!("    pub async fn {}(", op.fn_name));
            output.push_str("&self");

            let path_param_names = self.path_param_names(op);
            for param in &path_param_names {
                output.push_str(&format!(
                    ", {}: impl std::fmt::Display",
                    param
                ));
            }

            if !op.query_params.is_empty() {
                output.push_str(&format!(", query: Option<&{}>", query_name));
            }

            if !op.header_params.is_empty() {
                output.push_str(&format!(", headers: Option<&{}>", headers_name));
            }

            if let Some(body) = &op.body {
                if body.required {
                    output.push_str(&format!(", body: &{}", body.rust_type));
                } else {
                    output.push_str(&format!(", body: Option<&{}>", body.rust_type));
                }
            }

            output.push_str(&format!(
                ") -> Result<{}> {{\n",
                op.response_type
            ));

            let (path_fmt, path_args) = self.build_path_format(&op.path);
            if path_args.is_empty() {
                output.push_str(&format!("        let path = \"{}\".to_string();\n", op.path));
            } else {
                output.push_str(&format!("        let path = format!(\"{}\"", path_fmt));
                for arg in path_args {
                    output.push_str(&format!(", {}", arg));
                }
                output.push_str(");\n");
            }

            output.push_str(&format!(
                "        let mut req = self.request(Method::{}, &path);\n",
                op.method.to_uppercase()
            ));

            if !op.auth_schemes.is_empty() {
                let auth_list = self.render_auth_scheme_list(&op.auth_schemes);
                output.push_str(&format!(
                    "        req = self.apply_auth(req, &{});\n",
                    auth_list
                ));
            }

            if let Some(content_type) = &op.response_content_type {
                output.push_str(&format!(
                    "        req = req.header(\"Accept\", \"{}\");\n",
                    content_type
                ));
            }

            if !op.query_params.is_empty() {
                output.push_str("        if let Some(query) = query {\n");
                output.push_str("            req = req.query(query);\n");
                output.push_str("        }\n");
            }

            if !op.header_params.is_empty() {
                output.push_str("        if let Some(headers) = headers {\n");
                for param in &op.header_params {
                    if param.required {
                        output.push_str(&format!(
                            "            req = req.header(\"{}\", headers.{}.to_string());\n",
                            param.original_name, param.rust_name
                        ));
                    } else {
                        output.push_str(&format!(
                            "            if let Some(value) = &headers.{} {{\n",
                            param.rust_name
                        ));
                        output.push_str(&format!(
                            "                req = req.header(\"{}\", value.to_string());\n",
                            param.original_name
                        ));
                        output.push_str("            }\n");
                    }
                }
                output.push_str("        }\n");
            }

            if let Some(body) = &op.body {
                output.push_str(&self.render_body_apply(body));
            }

            output.push_str("        let response = req.send().await?;\n\n");
            output.push_str("        if !response.status().is_success() {\n");
            output.push_str("            return Err(ClientError::Api {\n");
            output.push_str("                status: response.status().as_u16(),\n");
            output.push_str("                message: response.text().await.unwrap_or_default(),\n");
            output.push_str("            });\n");
            output.push_str("        }\n\n");

        if op.response_type == "()" {
            output.push_str("        Ok(())\n");
        } else {
            output.push_str("        Ok(response.json().await?)\n");
        }

            output.push_str("    }\n\n");
        }

        output
    }

    fn render_auth_scheme_list(&self, schemes: &[AuthSchemeDef]) -> String {
        let mut parts = Vec::new();
        for scheme in schemes {
            let expr = match scheme {
                AuthSchemeDef::Bearer => "AuthScheme::Bearer".to_string(),
                AuthSchemeDef::Basic => "AuthScheme::Basic".to_string(),
                AuthSchemeDef::ApiKeyHeader(name) => {
                    format!("AuthScheme::ApiKeyHeader(\"{}\")", name)
                }
                AuthSchemeDef::ApiKeyQuery(name) => {
                    format!("AuthScheme::ApiKeyQuery(\"{}\")", name)
                }
            };
            if !parts.contains(&expr) {
                parts.push(expr);
            }
        }

        format!("[{}]", parts.join(", "))
    }

    fn render_body_apply(&self, body: &OperationBody) -> String {
        let mut output = String::new();
        let content_type = body.content_type.as_str();

        let apply = match body.kind {
            BodyKind::Json => "req = req.json(body);\n".to_string(),
            BodyKind::Form => "req = req.form(body);\n".to_string(),
            BodyKind::Text => "req = req.body(body.to_string());\n".to_string(),
            BodyKind::Binary => "req = req.body(body.clone());\n".to_string(),
            BodyKind::Other => "req = req.body(serde_json::to_vec(body)?);\n".to_string(),
        };

        if body.required {
            output.push_str(&format!(
                "        req = req.header(\"Content-Type\", \"{}\");\n",
                content_type
            ));
            output.push_str("        ");
            output.push_str(&apply);
        } else {
            output.push_str("        if let Some(body) = body {\n");
            output.push_str(&format!(
                "            req = req.header(\"Content-Type\", \"{}\");\n",
                content_type
            ));
            output.push_str("            ");
            output.push_str(&apply);
            output.push_str("        }\n");
        }

        output
    }

    fn build_path_format(&self, path: &str) -> (String, Vec<String>) {
        let mut fmt = String::new();
        let mut args = Vec::new();
        let mut chars = path.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '{' {
                let mut name = String::new();
                while let Some(next) = chars.next() {
                    if next == '}' {
                        break;
                    }
                    name.push(next);
                }
                let rust_name = sanitize_field_name(&name);
                args.push(rust_name);
                fmt.push_str("{}");
            } else {
                fmt.push(ch);
            }
        }

        (fmt, args)
    }

    fn path_param_names(&self, op: &OperationInfo) -> Vec<String> {
        let mut names = op
            .path_params
            .iter()
            .map(|p| p.rust_name.clone())
            .collect::<Vec<_>>();
        let (_, path_args) = self.build_path_format(&op.path);

        for arg in path_args {
            if !names.contains(&arg) {
                names.push(arg);
            }
        }

        names
    }

    fn generate_lib_code(&self, client_name: &str) -> String {
        let crate_name = self.crate_name(client_name);
        format!(
            r#"//! API Client Library
//!
//! This library provides a type-safe client for interacting with the API.
//! All types are generated from the OpenAPI specification.
//!
//! # Example
//!
//! ```no_run
//! use {}::{{Client, ClientConfig}};
//!
//! #[tokio::main]
//! async fn main() {{
//!     let client = Client::with_base_url("https://api.example.com")
//!         .with_bearer_token("your-api-token");
//!
//!     // Use the client methods here
//! }}
//! ```

pub mod client;
pub mod types;

pub use client::{{Client, ClientConfig, ClientError, Result}};
pub use types::*;

// Re-export commonly used types
pub use validator::Validate;
"#,
            crate_name
        )
    }

    fn generate_cargo_toml(&self, client_name: &str) -> String {
        let crate_name = self.crate_name(client_name);
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
base64 = "0.22"

[dev-dependencies]
tokio-test = "0.4"

[[example]]
name = "basic"
path = "examples/basic.rs"
"#,
            crate_name
        )
    }

    fn generate_example_code(&self, client_name: &str) -> String {
        let crate_name = self.crate_name(client_name);
        format!(
            r#"//! Basic usage example for {} client
//!
//! Run with: cargo run --example basic

use {}::{{Client, ClientConfig}};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {{
    // Create client with configuration
    let config = ClientConfig {{
        base_url: "https://api.example.com".to_string(),
        bearer_token: std::env::var("API_TOKEN").ok(),
        basic_auth: None,
        api_key_headers: std::collections::HashMap::new(),
        api_key_queries: std::collections::HashMap::new(),
        timeout_secs: 30,
        user_agent: Some("my-app/1.0".to_string()),
    }};

    let client = Client::new(config);

    println!("✓ Client created successfully");
    println!("  Base URL: {{}}", client.config().base_url);
    println!("  Auth: {{}}", if client.config().bearer_token.is_some() {{ "✓ Configured" }} else {{ "✗ Not configured" }});

    // Add your API calls here
    // Example:
    // let user = client.get_user("123").await?;
    // println!("User: {{:?}}", user);

    Ok(())
}}
"#,
            client_name,
            crate_name
        )
    }

    fn generate_readme(&self, client_name: &str) -> String {
        let crate_name = self.crate_name(client_name);
        let repo_url = env!("CARGO_PKG_REPOSITORY");
        format!(
            r#"# {} Client

Type-safe Rust client library generated from OpenAPI specification using [UniStructGen]({}).

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
use {}::{{Client, ClientConfig}};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {{
    // Create client
    let client = Client::with_base_url("https://api.example.com")
        .with_bearer_token("your-api-token");

    // Make API calls
    // let result = client.some_method().await?;
    // println!("{{:?}}", result);

    Ok(())
}}
```

## Configuration

```rust
use {}::ClientConfig;

let config = ClientConfig {{
    base_url: "https://api.example.com".to_string(),
    bearer_token: Some("your-token".to_string()),
    basic_auth: None,
    api_key_headers: std::collections::HashMap::new(),
    api_key_queries: std::collections::HashMap::new(),
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

This client was generated using [UniStructGen]({}) -
a powerful Rust code generator that creates type-safe code from JSON, OpenAPI specs, and more.

### Regenerating the client

```bash
unistructgen client --spec your-spec.yaml --output . --name "{}"
```

## License

MIT OR Apache-2.0
"#,
            client_name,
            repo_url,
            crate_name,
            crate_name,
            crate_name,
            repo_url,
            client_name
        )
    }

    fn crate_name(&self, client_name: &str) -> String {
        let mut slug = String::new();
        let mut last_dash = false;
        for ch in client_name.chars() {
            let lower = ch.to_ascii_lowercase();
            if lower.is_ascii_alphanumeric() {
                slug.push(lower);
                last_dash = false;
            } else if !last_dash {
                slug.push('-');
                last_dash = true;
            }
        }

        let slug = slug.trim_matches('-');
        let base = if slug.is_empty() { "api" } else { slug };
        format!("{}-client", base)
    }
}
