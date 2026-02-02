use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, parse::{Parse, ParseStream}, Token, LitStr, LitBool, Ident};
use unistructgen_core::Parser;
use unistructgen_codegen::{RenderOptions, RustRenderer};
use unistructgen_json_parser::{JsonParser, ParserOptions};
use unistructgen_openapi_parser::{OpenApiParser, OpenApiParserOptions};
use unistructgen_env_parser::{EnvParser, EnvParserOptions};
use unistructgen_sql_parser::{SqlParser, SqlParserOptions};
use unistructgen_graphql_parser::{GraphqlParser, GraphqlParserOptions};

mod ai_tool;
mod into_ir;

/// Derive macro for `IntoIR` trait.
///
/// Allows generating UniStructGen IR from Rust structs.
///
/// # Example
///
/// ```ignore
/// #[derive(IntoIR)]
/// struct User {
///     #[field(min_length = 5)]
///     username: String,
/// }
/// ```
#[proc_macro_derive(IntoIR, attributes(field))]
pub fn derive_into_ir(input: TokenStream) -> TokenStream {
    into_ir::impl_into_ir(input)
}

/// Attribute macro to convert a Rust function into an AI Tool.
///
/// Generates a struct implementing `AiTool` that can be registered in a `ToolRegistry`.
///
/// # Example
///
/// ```ignore
/// #[ai_tool]
/// fn calculate_sum(a: i32, b: i32) -> i32 {
///     a + b
/// }
/// ```
#[proc_macro_attribute]
pub fn ai_tool(attr: TokenStream, item: TokenStream) -> TokenStream {
    ai_tool::ai_tool_impl(attr, item)
}

/// Macro input for generate_struct_from_json
struct JsonStructInput {
    name: String,
    json: String,
    serde: bool,
    default: bool,
    optional: bool,
    reverse_ir: bool,
}

impl Parse for JsonStructInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut name = String::from("Generated");
        let mut json = String::new();
        let mut serde = true;
        let mut default = false;
        let mut optional = false;
        let mut reverse_ir = false;

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            input.parse::<Token![=]>()?;

            match key.to_string().as_str() {
                "name" => {
                    let value: LitStr = input.parse()?;
                    name = value.value();
                }
                "json" => {
                    let value: LitStr = input.parse()?;
                    json = value.value();
                }
                "serde" => {
                    let value: LitBool = input.parse()?;
                    serde = value.value;
                }
                "default" => {
                    let value: LitBool = input.parse()?;
                    default = value.value;
                }
                "optional" => {
                    let value: LitBool = input.parse()?;
                    optional = value.value;
                }
                "reverse_ir" => {
                    let value: LitBool = input.parse()?;
                    reverse_ir = value.value;
                }
                _ => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!("Unknown parameter: {}", key),
                    ));
                }
            }

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        if json.is_empty() {
            return Err(syn::Error::new(
                input.span(),
                "Missing required 'json' parameter",
            ));
        }

        Ok(JsonStructInput {
            name,
            json,
            serde,
            default,
            optional,
            reverse_ir,
        })
    }
}

/// Generate Rust struct from inline JSON
#[proc_macro]
pub fn generate_struct_from_json(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as JsonStructInput);

    // Parse JSON and generate IR
    let parser_options = ParserOptions {
        struct_name: input.name.clone(),
        derive_serde: input.serde,
        derive_default: input.default,
        make_fields_optional: input.optional,
    };

    let mut parser = JsonParser::new(parser_options);
    let mut ir_module = match parser.parse(&input.json) {
        Ok(module) => module,
        Err(e) => {
            return syn::Error::new(
                proc_macro2::Span::call_site(),
                format!("Failed to parse JSON: {}", e),
            )
            .to_compile_error()
            .into();
        }
    };

    // If reverse_ir is enabled, add IntoIR derive
    if input.reverse_ir {
        for ty in &mut ir_module.types {
            if let unistructgen_core::IRType::Struct(s) = ty {
                s.derives.push("IntoIR".to_string());
            }
        }
    }

    // Generate Rust code
    let renderer = RustRenderer::new(RenderOptions {
        add_header: false, // Don't add header in proc-macro
        add_clippy_allows: false,
    });

    let generated_code = match renderer.render(&ir_module) {
        Ok(code) => code,
        Err(e) => {
            return syn::Error::new(
                proc_macro2::Span::call_site(),
                format!("Failed to generate code: {}", e),
            )
            .to_compile_error()
            .into();
        }
    };

    // Parse generated code back into TokenStream
    generated_code.parse().unwrap_or_else(|e| {
        syn::Error::new(
            proc_macro2::Span::call_site(),
            format!("Failed to parse generated code: {}", e),
        )
        .to_compile_error()
        .into()
    })
}

/// Attribute macro parameters
struct AttributeParams {
    name: String,
    serde: bool,
    default: bool,
    optional: bool,
}

impl Parse for AttributeParams {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut name = String::from("Generated");
        let mut serde = true;
        let mut default = false;
        let mut optional = false;

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            input.parse::<Token![=]>()?;

            match key.to_string().as_str() {
                "name" => {
                    let value: LitStr = input.parse()?;
                    name = value.value();
                }
                "serde" => {
                    let value: LitBool = input.parse()?;
                    serde = value.value;
                }
                "default" => {
                    let value: LitBool = input.parse()?;
                    default = value.value;
                }
                "optional" => {
                    let value: LitBool = input.parse()?;
                    optional = value.value;
                }
                _ => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!("Unknown parameter: {}", key),
                    ));
                }
            }

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(AttributeParams {
            name,
            serde,
            default,
            optional,
        })
    }
}

/// Attribute macro to generate struct from JSON string
#[proc_macro_attribute]
pub fn json_struct(attr: TokenStream, item: TokenStream) -> TokenStream {
    let params = if attr.is_empty() {
        AttributeParams {
            name: "Generated".to_string(),
            serde: true,
            default: false,
            optional: false,
        }
    } else {
        parse_macro_input!(attr as AttributeParams)
    };

    let item = parse_macro_input!(item as syn::ItemConst);

    // Extract JSON string from const value
    let json_str = if let syn::Expr::Lit(lit) = &*item.expr {
        if let syn::Lit::Str(s) = &lit.lit {
            s.value()
        } else {
            return syn::Error::new_spanned(
                item,
                "Attribute must be applied to a string constant",
            )
            .to_compile_error()
            .into();
        }
    } else {
        return syn::Error::new_spanned(
            item,
            "Attribute must be applied to a string literal",
        )
        .to_compile_error()
        .into();
    };

    // Parse JSON and generate IR
    let parser_options = ParserOptions {
        struct_name: params.name.clone(),
        derive_serde: params.serde,
        derive_default: params.default,
        make_fields_optional: params.optional,
    };

    let mut parser = JsonParser::new(parser_options);
    let ir_module = match parser.parse(&json_str) {
        Ok(module) => module,
        Err(e) => {
            return syn::Error::new_spanned(
                item,
                format!("Failed to parse JSON: {}", e),
            )
            .to_compile_error()
            .into();
        }
    };

    // Generate Rust code
    let renderer = RustRenderer::new(RenderOptions {
        add_header: false,
        add_clippy_allows: false,
    });

    let generated_code = match renderer.render(&ir_module) {
        Ok(code) => code,
        Err(e) => {
            return syn::Error::new_spanned(
                item,
                format!("Failed to generate code: {}", e),
            )
            .to_compile_error()
            .into();
        }
    };

    // Parse and return both the original const and generated code
    let generated_tokens: proc_macro2::TokenStream = generated_code.parse().unwrap();
    let original_const = quote! { #item };

    let output = quote! {
        #original_const
        #generated_tokens
    };

    output.into()
}

/// Authentication method for API requests
#[derive(Clone, Debug)]
enum AuthMethod {
    /// Bearer token authentication (e.g., OAuth2, JWT)
    Bearer(String),
    /// API key in custom header
    ApiKey { header: String, value: String },
    /// HTTP Basic authentication
    Basic { username: String, password: String },
}

/// Macro input for struct_from_external_api
struct ExternalApiInput {
    struct_name: String,
    url: String,
    method: String,
    #[allow(dead_code)]
    consumer: String,
    timeout: u64,
    max_entity_count: Option<usize>,
    max_depth: Option<usize>,
    #[allow(dead_code)]
    conflict_strategy: ConflictStrategy,
    serde: bool,
    default: bool,
    optional: bool,
    auth: Option<AuthMethod>,
    auth_bearer_env: Option<String>,
    env_file: Option<String>,
}

#[derive(Clone)]
enum ConflictStrategy {
    Error,
    Rename,
    Override,
}

impl Parse for ExternalApiInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut struct_name = String::from("ApiResponse");
        let mut url = String::new();
        let mut method = String::from("GET");
        let mut consumer = String::from("json");
        let mut timeout = 30000;
        let mut max_entity_count = None;
        let mut max_depth = None;
        let mut conflict_strategy = ConflictStrategy::Rename;
        let mut serde = true;
        let mut default = false;
        let mut optional = false;
        let mut auth = None;
        let mut auth_bearer_env = None;
        let mut env_file = None;

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            input.parse::<Token![=]>()?;

            match key.to_string().as_str() {
                "struct_name" => {
                    let value: LitStr = input.parse()?;
                    struct_name = value.value();
                }
                "url" | "url_api" => {
                    let value: LitStr = input.parse()?;
                    url = value.value();
                }
                "method" => {
                    let value: LitStr = input.parse()?;
                    method = value.value().to_uppercase();
                }
                "consumer" => {
                    let value: LitStr = input.parse()?;
                    consumer = value.value().to_lowercase();
                }
                "timeout" | "request_timeout" => {
                    let value: syn::LitInt = input.parse()?;
                    timeout = value.base10_parse()?;
                }
                "max_entity_count" => {
                    let value: syn::LitInt = input.parse()?;
                    max_entity_count = Some(value.base10_parse()?);
                }
                "depth" | "max_depth" => {
                    let value: syn::LitInt = input.parse()?;
                    max_depth = Some(value.base10_parse()?);
                }
                "conflict_strategy" => {
                    let value: LitStr = input.parse()?;
                    conflict_strategy = match value.value().as_str() {
                        "error" => ConflictStrategy::Error,
                        "rename" => ConflictStrategy::Rename,
                        "override" => ConflictStrategy::Override,
                        other => {
                            return Err(syn::Error::new(
                                value.span(),
                                format!("Unknown conflict strategy: {}. Use 'error', 'rename', or 'override'", other),
                            ));
                        }
                    };
                }
                "serde" => {
                    let value: LitBool = input.parse()?;
                    serde = value.value;
                }
                "default" => {
                    let value: LitBool = input.parse()?;
                    default = value.value;
                }
                "optional" => {
                    let value: LitBool = input.parse()?;
                    optional = value.value;
                }
                "auth_bearer" => {
                    let value: LitStr = input.parse()?;
                    auth = Some(AuthMethod::Bearer(value.value()));
                }
                "auth_bearer_env" => {
                    let value: LitStr = input.parse()?;
                    auth_bearer_env = Some(value.value());
                }
                "env_file" => {
                    let value: LitStr = input.parse()?;
                    env_file = Some(value.value());
                }
                "auth_api_key" => {
                    let value: LitStr = input.parse()?;
                    let val_string = value.value();
                    let parts: Vec<&str> = val_string.splitn(2, ':').collect();
                    if parts.len() != 2 {
                        return Err(syn::Error::new(
                            value.span(),
                            "auth_api_key must be in format 'Header-Name:value' (e.g., 'X-API-Key:your_key')",
                        ));
                    }
                    auth = Some(AuthMethod::ApiKey {
                        header: parts[0].to_string(),
                        value: parts[1].to_string(),
                    });
                }
                "auth_basic" => {
                    let value: LitStr = input.parse()?;
                    let val_string = value.value();
                    let parts: Vec<&str> = val_string.splitn(2, ':').collect();
                    if parts.len() != 2 {
                        return Err(syn::Error::new(
                            value.span(),
                            "auth_basic must be in format 'username:password'",
                        ));
                    }
                    auth = Some(AuthMethod::Basic {
                        username: parts[0].to_string(),
                        password: parts[1].to_string(),
                    });
                }
                _ => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!("Unknown parameter: {}", key),
                    ));
                }
            }

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        if url.is_empty() {
            return Err(syn::Error::new(
                input.span(),
                "Missing required 'url' or 'url_api' parameter",
            ));
        }

        Ok(ExternalApiInput {
            struct_name,
            url,
            method,
            consumer,
            timeout,
            max_entity_count,
            max_depth,
            conflict_strategy,
            serde,
            default,
            optional,
            auth,
            auth_bearer_env,
            env_file,
        })
    }
}

fn read_env_value_from_dotenv(
    key: &str,
    env_file: Option<&str>,
    timeout_ms: u64,
) -> Result<String, String> {
    let timeout_ms = effective_timeout_ms(timeout_ms);
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .map_err(|_| "CARGO_MANIFEST_DIR not set".to_string())?;

    let (content, source_label) = match env_file {
        Some(path) if path.starts_with("http://") || path.starts_with("https://") => {
            let body = fetch_text_with_cache(path, "GET", timeout_ms, None)
                .map_err(|e| format!("Failed to fetch .env from {}: {}", path, e))?;
            (body, path.to_string())
        }
        Some(path) => {
            let env_path = if std::path::Path::new(path).is_absolute() {
                std::path::Path::new(path).to_path_buf()
            } else {
                std::path::Path::new(&manifest_dir).join(path)
            };
            let body = std::fs::read_to_string(&env_path)
                .map_err(|e| format!("Failed to read .env at {}: {}", env_path.display(), e))?;
            (body, env_path.display().to_string())
        }
        None => {
            let env_path = std::path::Path::new(&manifest_dir).join(".env");
            let body = std::fs::read_to_string(&env_path)
                .map_err(|e| format!("Failed to read .env at {}: {}", env_path.display(), e))?;
            (body, env_path.display().to_string())
        }
    };

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            let k = k.trim();
            let mut v = v.trim().to_string();

            if (v.starts_with('"') && v.ends_with('"')) || (v.starts_with('\'') && v.ends_with('\'')) {
                if v.len() >= 2 {
                    v = v[1..v.len() - 1].to_string();
                }
            }

            if k == key {
                return Ok(v);
            }
        }
    }

    Err(format!("Key '{}' not found in .env source {}", key, source_label))
}

/// Make HTTP request and fetch JSON
fn fetch_json_from_api(input: &ExternalApiInput) -> Result<String, String> {
    let timeout_ms = effective_timeout_ms(input.timeout);

    if input.method != "GET" {
        return fetch_json_no_cache(input, timeout_ms);
    }

    let mut auth = input.auth.clone();
    if auth.is_none() {
        if let Some(env_key) = &input.auth_bearer_env {
            let token = read_env_value_from_dotenv(
                env_key,
                input.env_file.as_deref(),
                timeout_ms,
            )?;
            auth = Some(AuthMethod::Bearer(token));
        }
    }

    fetch_text_with_cache(&input.url, "GET", timeout_ms, auth.as_ref())
}

fn fetch_json_no_cache(input: &ExternalApiInput, timeout_ms: u64) -> Result<String, String> {
    let agent = ureq::AgentBuilder::new()
        .timeout(std::time::Duration::from_millis(timeout_ms))
        .build();

    let mut request = match input.method.as_str() {
        "GET" => agent.get(&input.url),
        "POST" => agent.post(&input.url),
        "PUT" => agent.put(&input.url),
        "DELETE" => agent.delete(&input.url),
        _ => return Err(format!("Unsupported HTTP method: {}", input.method)),
    };

    // Apply authentication if provided
    let mut auth = input.auth.clone();
    if auth.is_none() {
        if let Some(env_key) = &input.auth_bearer_env {
            let token = read_env_value_from_dotenv(
                env_key,
                input.env_file.as_deref(),
                timeout_ms,
            )?;
            auth = Some(AuthMethod::Bearer(token));
        }
    }

    if let Some(auth) = &auth {
        request = match auth {
            AuthMethod::Bearer(token) => {
                request.set("Authorization", &format!("Bearer {}", token))
            }
            AuthMethod::ApiKey { header, value } => {
                request.set(header, value)
            }
            AuthMethod::Basic { username, password } => {
                // Encode username:password in base64 for Basic Auth
                let credentials = format!("{}:{}", username, password);
                let encoded = base64_encode(&credentials);
                request.set("Authorization", &format!("Basic {}", encoded))
            }
        };
    }

    let response = request
        .call()
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    let json_str = response
        .into_string()
        .map_err(|e| format!("Failed to read response body: {}", e))?;

    Ok(json_str)
}

/// Simple base64 encoding for Basic Auth
fn base64_encode(input: &str) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/=";
    let bytes = input.as_bytes();
    let mut result = String::new();

    for chunk in bytes.chunks(3) {
        let mut buf = [0u8; 3];
        for (i, &b) in chunk.iter().enumerate() {
            buf[i] = b;
        }

        let b1 = (buf[0] >> 2) as usize;
        let b2 = (((buf[0] & 0x03) << 4) | (buf[1] >> 4)) as usize;
        let b3 = (((buf[1] & 0x0F) << 2) | (buf[2] >> 6)) as usize;
        let b4 = (buf[2] & 0x3F) as usize;

        result.push(CHARSET[b1] as char);
        result.push(CHARSET[b2] as char);
        result.push(if chunk.len() > 1 { CHARSET[b3] as char } else { '=' });
        result.push(if chunk.len() > 2 { CHARSET[b4] as char } else { '=' });
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base64_encode() {
        assert_eq!(base64_encode("hello"), "aGVsbG8=");
    }
}

/// Limit nested depth in JSON
fn limit_json_depth(value: serde_json::Value, max_depth: usize, current_depth: usize) -> serde_json::Value {
    if current_depth >= max_depth {
        return serde_json::Value::Null;
    }

    match value {
        serde_json::Value::Object(map) => {
            let mut new_map = serde_json::Map::new();
            for (k, v) in map {
                new_map.insert(k, limit_json_depth(v, max_depth, current_depth + 1));
            }
            serde_json::Value::Object(new_map)
        }
        serde_json::Value::Array(arr) => {
            let limited: Vec<_> = arr
                .into_iter()
                .map(|v| limit_json_depth(v, max_depth, current_depth + 1))
                .collect();
            serde_json::Value::Array(limited)
        }
        other => other,
    }
}

/// Generate struct from external API call
#[proc_macro]
pub fn struct_from_external_api(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ExternalApiInput);

    // Fetch JSON from API
    let json_str = match fetch_json_from_api(&input) {
        Ok(json) => json,
        Err(e) => {
            return syn::Error::new(
                proc_macro2::Span::call_site(),
                format!("Failed to fetch from API: {}", e),
            )
            .to_compile_error()
            .into();
        }
    };

    // Parse JSON
    let mut json_value: serde_json::Value = match serde_json::from_str(&json_str) {
        Ok(v) => v,
        Err(e) => {
            return syn::Error::new(
                proc_macro2::Span::call_site(),
                format!("Failed to parse API response as JSON: {}", e),
            )
            .to_compile_error()
            .into();
        }
    };

    // If API returns an array, extract the first element to infer structure
    if let serde_json::Value::Array(ref arr) = json_value {
        if arr.is_empty() {
            return syn::Error::new(
                proc_macro2::Span::call_site(),
                "API returned an empty array. Cannot infer struct from empty array.",
            )
            .to_compile_error()
            .into();
        }
        // Extract first element as the basis for the struct
        json_value = arr[0].clone();
    }

    // Handle max_entity_count for arrays (for nested arrays within the object)
    if let Some(max_count) = input.max_entity_count {
        if let serde_json::Value::Array(ref mut arr) = json_value {
            arr.truncate(max_count);
        }
    }

    // Apply depth limit
    if let Some(max_depth) = input.max_depth {
        json_value = limit_json_depth(json_value, max_depth, 0);
    }

    // Convert back to string
    let processed_json = match serde_json::to_string_pretty(&json_value) {
        Ok(s) => s,
        Err(e) => {
            return syn::Error::new(
                proc_macro2::Span::call_site(),
                format!("Failed to serialize processed JSON: {}", e),
            )
            .to_compile_error()
            .into();
        }
    };

    // Parse JSON and generate IR
    let parser_options = ParserOptions {
        struct_name: input.struct_name.clone(),
        derive_serde: input.serde,
        derive_default: input.default,
        make_fields_optional: input.optional,
    };

    let mut parser = JsonParser::new(parser_options);
    let ir_module = match parser.parse(&processed_json) {
        Ok(module) => module,
        Err(e) => {
            return syn::Error::new(
                proc_macro2::Span::call_site(),
                format!("Failed to parse JSON from API: {}", e),
            )
            .to_compile_error()
            .into();
        }
    };

    // Generate Rust code
    let renderer = RustRenderer::new(RenderOptions {
        add_header: false,
        add_clippy_allows: false,
    });

    let generated_code = match renderer.render(&ir_module) {
        Ok(code) => code,
        Err(e) => {
            return syn::Error::new(
                proc_macro2::Span::call_site(),
                format!("Failed to generate code: {}", e),
            )
            .to_compile_error()
            .into();
        }
    };

    // Parse generated code back into TokenStream
    generated_code.parse().unwrap_or_else(|e| {
        syn::Error::new(
            proc_macro2::Span::call_site(),
            format!("Failed to parse generated code: {}", e),
        )
        .to_compile_error()
        .into()
    })
}

/// Macro input for openapi_to_rust
struct OpenApiInput {
    spec: Option<String>,
    url: Option<String>,
    file: Option<String>,
    generate_client: bool,
    generate_validation: bool,
    derive_serde: bool,
    derive_default: bool,
    timeout: u64,
    auth: Option<AuthMethod>,
}

impl Parse for OpenApiInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut spec = None;
        let mut url = None;
        let mut file = None;
        let mut generate_client = true;
        let mut generate_validation = true;
        let mut derive_serde = true;
        let mut derive_default = false;
        let mut timeout = 30000;
        let mut auth = None;

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            input.parse::<Token![=]>()?;

            match key.to_string().as_str() {
                "spec" => {
                    let value: LitStr = input.parse()?;
                    spec = Some(value.value());
                }
                "url" => {
                    let value: LitStr = input.parse()?;
                    url = Some(value.value());
                }
                "file" => {
                    let value: LitStr = input.parse()?;
                    file = Some(value.value());
                }
                "generate_client" => {
                    let value: LitBool = input.parse()?;
                    generate_client = value.value;
                }
                "generate_validation" => {
                    let value: LitBool = input.parse()?;
                    generate_validation = value.value;
                }
                "serde" => {
                    let value: LitBool = input.parse()?;
                    derive_serde = value.value;
                }
                "default" => {
                    let value: LitBool = input.parse()?;
                    derive_default = value.value;
                }
                "timeout" => {
                    let value: syn::LitInt = input.parse()?;
                    timeout = value.base10_parse()?;
                }
                "auth_bearer" => {
                    let value: LitStr = input.parse()?;
                    auth = Some(AuthMethod::Bearer(value.value()));
                }
                "auth_api_key" => {
                    let value: LitStr = input.parse()?;
                    let val_string = value.value();
                    let parts: Vec<&str> = val_string.splitn(2, ':').collect();
                    if parts.len() != 2 {
                        return Err(syn::Error::new(
                            value.span(),
                            "auth_api_key must be in format 'Header-Name:value'",
                        ));
                    }
                    auth = Some(AuthMethod::ApiKey {
                        header: parts[0].to_string(),
                        value: parts[1].to_string(),
                    });
                }
                "auth_basic" => {
                    let value: LitStr = input.parse()?;
                    let val_string = value.value();
                    let parts: Vec<&str> = val_string.splitn(2, ':').collect();
                    if parts.len() != 2 {
                        return Err(syn::Error::new(
                            value.span(),
                            "auth_basic must be in format 'username:password'",
                        ));
                    }
                    auth = Some(AuthMethod::Basic {
                        username: parts[0].to_string(),
                        password: parts[1].to_string(),
                    });
                }
                _ => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!("Unknown parameter: {}", key),
                    ));
                }
            }

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        // Validate that at least one source is provided
        if spec.is_none() && url.is_none() && file.is_none() {
            return Err(syn::Error::new(
                input.span(),
                "Must provide one of: 'spec', 'url', or 'file'",
            ));
        }

        Ok(OpenApiInput {
            spec,
            url,
            file,
            generate_client,
            generate_validation,
            derive_serde,
            derive_default,
            timeout,
            auth,
        })
    }
}

/// Generate Rust types from OpenAPI specification
#[proc_macro]
pub fn openapi_to_rust(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as OpenApiInput);

    // Get the OpenAPI specification content
    let spec_content = if let Some(spec) = input.spec {
        // Inline specification
        spec
    } else if let Some(url) = input.url {
        // Fetch from URL
        match fetch_openapi_from_url(&url, input.timeout, input.auth.as_ref()) {
            Ok(content) => content,
            Err(e) => {
                return syn::Error::new(
                    proc_macro2::Span::call_site(),
                    format!("Failed to fetch OpenAPI spec from URL: {}", e),
                )
                .to_compile_error()
                .into();
            }
        }
    } else if let Some(file) = input.file {
        // Read from file with CARGO_MANIFEST_DIR fallback
        let file_content = match std::fs::read_to_string(&file) {
            Ok(content) => Ok(content),
            Err(_) => {
                // Try relative to CARGO_MANIFEST_DIR
                match std::env::var("CARGO_MANIFEST_DIR") {
                    Ok(dir) => {
                        let path = std::path::Path::new(&dir).join(&file);
                        std::fs::read_to_string(&path).map_err(|e| {
                            format!("Failed to read OpenAPI spec from file '{}' (also tried '{}'): {}", file, path.display(), e)
                        })
                    }
                    Err(_) => Err(format!("Failed to read OpenAPI spec from file '{}' and CARGO_MANIFEST_DIR is not set", file)),
                }
            }
        };

        match file_content {
            Ok(content) => content,
            Err(e) => {
                return syn::Error::new(
                    proc_macro2::Span::call_site(),
                    e,
                )
                .to_compile_error()
                .into();
            }
        }
    } else {
        return syn::Error::new(
            proc_macro2::Span::call_site(),
            "No input source provided",
        )
        .to_compile_error()
        .into();
    };

    // Parse OpenAPI specification
    let options = OpenApiParserOptions::builder()
        .generate_client(input.generate_client)
        .generate_validation(input.generate_validation)
        .derive_serde(input.derive_serde)
        .derive_default(input.derive_default)
        .build();

    let mut parser = OpenApiParser::new(options);

    let ir_module = match parser.parse(&spec_content) {
        Ok(module) => module,
        Err(e) => {
            return syn::Error::new(
                proc_macro2::Span::call_site(),
                format!("Failed to parse OpenAPI specification: {}", e),
            )
            .to_compile_error()
            .into();
        }
    };

    // Generate Rust code
    let renderer = RustRenderer::new(RenderOptions {
        add_header: false,
        add_clippy_allows: false,
    });

    let generated_code = match renderer.render(&ir_module) {
        Ok(code) => code,
        Err(e) => {
            return syn::Error::new(
                proc_macro2::Span::call_site(),
                format!("Failed to generate Rust code: {}", e),
            )
            .to_compile_error()
            .into();
        }
    };

    // Parse generated code back into TokenStream
    generated_code.parse().unwrap_or_else(|e| {
        syn::Error::new(
            proc_macro2::Span::call_site(),
            format!("Failed to parse generated code: {}\n\nGenerated code:\n{}", e, generated_code),
        )
        .to_compile_error()
        .into()
    })
}

/// Fetch OpenAPI spec from URL with optional authentication
fn fetch_openapi_from_url(
    url: &str,
    timeout_ms: u64,
    auth: Option<&AuthMethod>,
) -> Result<String, String> {
    let timeout_ms = effective_timeout_ms(timeout_ms);
    let content = fetch_text_with_cache(url, "GET", timeout_ms, auth)?;

    if content.is_empty() {
        return Err("Empty response from URL".to_string());
    }

    Ok(content)
}

fn effective_timeout_ms(default_timeout: u64) -> u64 {
    std::env::var("UNISTRUCTGEN_FETCH_TIMEOUT_MS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .filter(|v| *v > 0)
        .unwrap_or(default_timeout)
}

fn fetch_text_with_cache(
    url: &str,
    method: &str,
    timeout_ms: u64,
    auth: Option<&AuthMethod>,
) -> Result<String, String> {
    if let Some(cache_dir) = fetch_cache_dir()? {
        let cache_path = cache_dir.join(cache_key(method, url));
        if is_offline_mode() {
            return read_cache(&cache_path).ok_or_else(|| {
                format!(
                    "Offline mode enabled and no cache found for {} {}",
                    method, url
                )
            });
        }
        if let Some(cached) = read_cache(&cache_path) {
            return Ok(cached);
        }
        let fetched = fetch_text_no_cache(url, timeout_ms, auth)?;
        write_cache(&cache_path, &fetched)?;
        return Ok(fetched);
    }

    if is_offline_mode() {
        return Err(format!(
            "Offline mode enabled but caching is disabled ({} {})",
            method, url
        ));
    }

    fetch_text_no_cache(url, timeout_ms, auth)
}

fn fetch_text_no_cache(
    url: &str,
    timeout_ms: u64,
    auth: Option<&AuthMethod>,
) -> Result<String, String> {
    let agent = ureq::AgentBuilder::new()
        .timeout(std::time::Duration::from_millis(timeout_ms))
        .build();

    let mut request = agent.get(url);

    if let Some(auth_method) = auth {
        request = match auth_method {
            AuthMethod::Bearer(token) => request.set("Authorization", &format!("Bearer {}", token)),
            AuthMethod::ApiKey { header, value } => request.set(header, value),
            AuthMethod::Basic { username, password } => {
                let credentials = format!("{}:{}", username, password);
                let encoded = base64_encode(&credentials);
                request.set("Authorization", &format!("Basic {}", encoded))
            }
        };
    }

    let response = request
        .call()
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    response
        .into_string()
        .map_err(|e| format!("Failed to read response body: {}", e))
}

fn fetch_cache_dir() -> Result<Option<std::path::PathBuf>, String> {
    if std::env::var("UNISTRUCTGEN_FETCH_CACHE")
        .ok()
        .map(|v| v == "0" || v.eq_ignore_ascii_case("false"))
        .unwrap_or(false)
    {
        return Ok(None);
    }

    if let Ok(dir) = std::env::var("UNISTRUCTGEN_FETCH_CACHE_DIR") {
        return Ok(Some(std::path::PathBuf::from(dir)));
    }

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .map_err(|_| "CARGO_MANIFEST_DIR not set".to_string())?;
    Ok(Some(
        std::path::Path::new(&manifest_dir)
            .join("target")
            .join("unistructgen_cache"),
    ))
}

fn is_offline_mode() -> bool {
    std::env::var("UNISTRUCTGEN_FETCH_OFFLINE")
        .ok()
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

fn cache_key(method: &str, url: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    method.hash(&mut hasher);
    url.hash(&mut hasher);
    format!("{:x}.cache", hasher.finish())
}

fn read_cache(path: &std::path::Path) -> Option<String> {
    std::fs::read_to_string(path).ok()
}

fn write_cache(path: &std::path::Path, content: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create cache dir {}: {}", parent.display(), e))?;
    }
    std::fs::write(path, content)
        .map_err(|e| format!("Failed to write cache file {}: {}", path.display(), e))
}

// ---------------------- NEW MACROS ----------------------

/// Macro input for generate_struct_from_env
struct EnvStructInput {
    name: String,
    env: String,
    serde: bool,
    default: bool,
    optional: bool,
}

impl Parse for EnvStructInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut name = String::from("Config");
        let mut env = String::new();
        let mut serde = false;
        let mut default = false;
        let mut optional = false;

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            input.parse::<Token![=]>()?;

            match key.to_string().as_str() {
                "name" => {
                    let value: LitStr = input.parse()?;
                    name = value.value();
                }
                "env" => {
                    let value: LitStr = input.parse()?;
                    env = value.value();
                }
                "serde" => {
                    let value: LitBool = input.parse()?;
                    serde = value.value;
                }
                "default" => {
                    let value: LitBool = input.parse()?;
                    default = value.value;
                }
                "optional" => {
                    let value: LitBool = input.parse()?;
                    optional = value.value;
                }
                _ => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!("Unknown parameter: {}", key),
                    ));
                }
            }

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        if env.is_empty() {
            return Err(syn::Error::new(
                input.span(),
                "Missing required 'env' parameter",
            ));
        }

        Ok(EnvStructInput {
            name,
            env,
            serde,
            default,
            optional,
        })
    }
}

/// Generate Rust struct from inline Env string
#[proc_macro]
pub fn generate_struct_from_env(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as EnvStructInput);

    let parser_options = EnvParserOptions {
        struct_name: input.name.clone(),
        derive_serde: input.serde,
        derive_default: input.default,
        make_fields_optional: input.optional,
    };

    let mut parser = EnvParser::new(parser_options);
    let ir_module = match parser.parse(&input.env) {
        Ok(module) => module,
        Err(e) => {
            return syn::Error::new(
                proc_macro2::Span::call_site(),
                format!("Failed to parse Env: {}", e),
            )
            .to_compile_error()
            .into();
        }
    };

    let renderer = RustRenderer::new(RenderOptions {
        add_header: false,
        add_clippy_allows: false,
    });

    let generated_code = match renderer.render(&ir_module) {
        Ok(code) => code,
        Err(e) => {
            return syn::Error::new(
                proc_macro2::Span::call_site(),
                format!("Failed to generate code: {}", e),
            )
            .to_compile_error()
            .into();
        }
    };

    generated_code.parse().unwrap_or_else(|e| {
        syn::Error::new(
            proc_macro2::Span::call_site(),
            format!("Failed to parse generated code: {}", e),
        )
        .to_compile_error()
        .into()
    })
}

/// Macro input for generate_struct_from_sql
struct SqlStructInput {
    sql: String,
    serde: bool,
    default: bool,
    optional: bool,
}

impl Parse for SqlStructInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut sql = String::new();
        let mut serde = true;
        let mut default = false;
        let mut optional = false;

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            input.parse::<Token![=]>()?;

            match key.to_string().as_str() {
                "sql" => {
                    let value: LitStr = input.parse()?;
                    sql = value.value();
                }
                "serde" => {
                    let value: LitBool = input.parse()?;
                    serde = value.value;
                }
                "default" => {
                    let value: LitBool = input.parse()?;
                    default = value.value;
                }
                "optional" => {
                    let value: LitBool = input.parse()?;
                    optional = value.value;
                }
                _ => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!("Unknown parameter: {}", key),
                    ));
                }
            }

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        if sql.is_empty() {
            return Err(syn::Error::new(
                input.span(),
                "Missing required 'sql' parameter",
            ));
        }

        Ok(SqlStructInput {
            sql,
            serde,
            default,
            optional,
        })
    }
}

/// Generate Rust struct from inline SQL DDL
#[proc_macro]
pub fn generate_struct_from_sql(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as SqlStructInput);

    let parser_options = SqlParserOptions {
        derive_serde: input.serde,
        derive_default: input.default,
        make_fields_optional: input.optional,
    };

    let mut parser = SqlParser::new(parser_options);
    let ir_module = match parser.parse(&input.sql) {
        Ok(module) => module,
        Err(e) => {
            return syn::Error::new(
                proc_macro2::Span::call_site(),
                format!("Failed to parse SQL: {}", e),
            )
            .to_compile_error()
            .into();
        }
    };

    let renderer = RustRenderer::new(RenderOptions {
        add_header: false,
        add_clippy_allows: false,
    });

    let generated_code = match renderer.render(&ir_module) {
        Ok(code) => code,
        Err(e) => {
            return syn::Error::new(
                proc_macro2::Span::call_site(),
                format!("Failed to generate code: {}", e),
            )
            .to_compile_error()
            .into();
        }
    };

    generated_code.parse().unwrap_or_else(|e| {
        syn::Error::new(
            proc_macro2::Span::call_site(),
            format!("Failed to parse generated code: {}", e),
        )
        .to_compile_error()
        .into()
    })
}

/// Macro input for generate_struct_from_graphql
struct GraphqlStructInput {
    schema: String,
    serde: bool,
    default: bool,
    optional: bool,
}

impl Parse for GraphqlStructInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut schema = String::new();
        let mut serde = true;
        let mut default = false;
        let mut optional = false;

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            input.parse::<Token![=]>()?;

            match key.to_string().as_str() {
                "schema" => {
                    let value: LitStr = input.parse()?;
                    schema = value.value();
                }
                "serde" => {
                    let value: LitBool = input.parse()?;
                    serde = value.value;
                }
                "default" => {
                    let value: LitBool = input.parse()?;
                    default = value.value;
                }
                "optional" => {
                    let value: LitBool = input.parse()?;
                    optional = value.value;
                }
                _ => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!("Unknown parameter: {}", key),
                    ));
                }
            }

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        if schema.is_empty() {
            return Err(syn::Error::new(
                input.span(),
                "Missing required 'schema' parameter",
            ));
        }

        Ok(GraphqlStructInput {
            schema,
            serde,
            default,
            optional,
        })
    }
}

/// Generate Rust struct from inline GraphQL schema
#[proc_macro]
pub fn generate_struct_from_graphql(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as GraphqlStructInput);

    let parser_options = GraphqlParserOptions {
        derive_serde: input.serde,
        derive_default: input.default,
        make_fields_optional: input.optional,
    };

    let mut parser = GraphqlParser::new(parser_options);
    let ir_module = match parser.parse(&input.schema) {
        Ok(module) => module,
        Err(e) => {
            return syn::Error::new(
                proc_macro2::Span::call_site(),
                format!("Failed to parse GraphQL: {}", e),
            )
            .to_compile_error()
            .into();
        }
    };

    let renderer = RustRenderer::new(RenderOptions {
        add_header: false,
        add_clippy_allows: false,
    });

    let generated_code = match renderer.render(&ir_module) {
        Ok(code) => code,
        Err(e) => {
            return syn::Error::new(
                proc_macro2::Span::call_site(),
                format!("Failed to generate code: {}", e),
            )
            .to_compile_error()
            .into();
        }
    };

    generated_code.parse().unwrap_or_else(|e| {
        syn::Error::new(
            proc_macro2::Span::call_site(),
            format!("Failed to parse generated code: {}", e),
        )
        .to_compile_error()
        .into()
    })
}
