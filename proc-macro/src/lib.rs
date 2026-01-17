use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, parse::{Parse, ParseStream}, Token, LitStr, LitBool, Ident};
use unistructgen_core::Parser;
use unistructgen_codegen::{RenderOptions, RustRenderer};
use unistructgen_json_parser::{JsonParser, ParserOptions};
use unistructgen_openapi_parser::{OpenApiParser, OpenApiParserOptions};

/// Macro input for generate_struct_from_json
struct JsonStructInput {
    name: String,
    json: String,
    serde: bool,
    default: bool,
    optional: bool,
}

impl Parse for JsonStructInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut name = String::from("Generated");
        let mut json = String::new();
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
        })
    }
}

/// Generate Rust struct from inline JSON
///
/// # Examples
///
/// ```ignore
/// use unistructgen_macro::generate_struct_from_json;
///
/// generate_struct_from_json! {
///     name = "User",
///     json = r#"{
///         "id": 1,
///         "name": "Alice",
///         "email": "alice@example.com"
///     }"#
/// }
/// ```
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
    let ir_module = match parser.parse(&input.json) {
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
///
/// # Examples
///
/// ```ignore
/// use unistructgen_macro::json_struct;
///
/// #[json_struct(name = "User")]
/// const SAMPLE: &str = r#"{
///     "id": 1,
///     "name": "Alice"
/// }"#;
/// ```
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
        })
    }
}

/// Make HTTP request and fetch JSON
fn fetch_json_from_api(input: &ExternalApiInput) -> Result<String, String> {
    let agent = ureq::AgentBuilder::new()
        .timeout(std::time::Duration::from_millis(input.timeout))
        .build();

    let mut request = match input.method.as_str() {
        "GET" => agent.get(&input.url),
        "POST" => agent.post(&input.url),
        "PUT" => agent.put(&input.url),
        "DELETE" => agent.delete(&input.url),
        _ => return Err(format!("Unsupported HTTP method: {}", input.method)),
    };

    // Apply authentication if provided
    if let Some(auth) = &input.auth {
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
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
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
        // Test basic encoding
        assert_eq!(base64_encode("hello"), "aGVsbG8=");
        assert_eq!(base64_encode("hello world"), "aGVsbG8gd29ybGQ=");

        // Test username:password format (typical for Basic Auth)
        assert_eq!(base64_encode("user:pass"), "dXNlcjpwYXNz");
        assert_eq!(base64_encode("admin:secret123"), "YWRtaW46c2VjcmV0MTIz");

        // Test empty string
        assert_eq!(base64_encode(""), "");

        // Test single character
        assert_eq!(base64_encode("a"), "YQ==");

        // Test two characters
        assert_eq!(base64_encode("ab"), "YWI=");

        // Test three characters (no padding)
        assert_eq!(base64_encode("abc"), "YWJj");
    }

    #[test]
    fn test_auth_method_bearer() {
        let auth = AuthMethod::Bearer("my_token_123".to_string());
        match auth {
            AuthMethod::Bearer(token) => assert_eq!(token, "my_token_123"),
            _ => panic!("Expected Bearer variant"),
        }
    }

    #[test]
    fn test_auth_method_api_key() {
        let auth = AuthMethod::ApiKey {
            header: "X-API-Key".to_string(),
            value: "my_key_456".to_string(),
        };
        match auth {
            AuthMethod::ApiKey { header, value } => {
                assert_eq!(header, "X-API-Key");
                assert_eq!(value, "my_key_456");
            }
            _ => panic!("Expected ApiKey variant"),
        }
    }

    #[test]
    fn test_auth_method_basic() {
        let auth = AuthMethod::Basic {
            username: "user".to_string(),
            password: "pass".to_string(),
        };
        match auth {
            AuthMethod::Basic { username, password } => {
                assert_eq!(username, "user");
                assert_eq!(password, "pass");
            }
            _ => panic!("Expected Basic variant"),
        }
    }
}

/// Merge multiple JSON samples to detect optional fields
#[allow(dead_code)]
fn merge_json_samples(samples: Vec<serde_json::Value>) -> serde_json::Value {
    if samples.is_empty() {
        return serde_json::Value::Null;
    }

    if samples.len() == 1 {
        return samples[0].clone();
    }

    // For now, just return the first sample
    // TODO: Implement proper field merging
    samples[0].clone()
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
///
/// This macro fetches JSON from an external API at compile time and generates
/// a Rust struct based on the response structure.
///
/// # Array Handling
///
/// If the API returns an array of objects, the macro automatically extracts
/// the first element to infer the struct definition. This allows you to work
/// with list endpoints without needing to manually extract a single item.
///
/// # Authentication
///
/// The macro supports three authentication methods:
///
/// - **Bearer Token** - OAuth2, JWT, etc.
/// - **API Key** - Custom header-based authentication
/// - **Basic Auth** - Username/password authentication
///
/// # Examples
///
/// ## API returning a single object (no auth)
///
/// ```ignore
/// use unistructgen_macro::struct_from_external_api;
///
/// struct_from_external_api! {
///     struct_name = "User",
///     url_api = "https://jsonplaceholder.typicode.com/users/1"
/// }
/// ```
///
/// ## API with Bearer token authentication
///
/// ```ignore
/// use unistructgen_macro::struct_from_external_api;
///
/// struct_from_external_api! {
///     struct_name = "User",
///     url_api = "https://api.example.com/user",
///     auth_bearer = "your_token_here"
/// }
/// ```
///
/// ## API with API Key authentication
///
/// ```ignore
/// use unistructgen_macro::struct_from_external_api;
///
/// struct_from_external_api! {
///     struct_name = "Data",
///     url_api = "https://api.example.com/data",
///     auth_api_key = "X-API-Key:your_api_key_here"
/// }
/// ```
///
/// ## API with Basic authentication
///
/// ```ignore
/// use unistructgen_macro::struct_from_external_api;
///
/// struct_from_external_api! {
///     struct_name = "Resource",
///     url_api = "https://api.example.com/resource",
///     auth_basic = "username:password"
/// }
/// ```
///
/// ## API returning an array (automatically extracts first element)
///
/// ```ignore
/// use unistructgen_macro::struct_from_external_api;
///
/// struct_from_external_api! {
///     struct_name = "Todo",
///     url_api = "https://jsonplaceholder.typicode.com/todos"
/// }
/// // Generates struct from the first element of the array
/// ```
///
/// ## Complete example with all options
///
/// ```ignore
/// use unistructgen_macro::struct_from_external_api;
///
/// struct_from_external_api! {
///     struct_name = "ApiResponse",
///     url_api = "https://api.example.com/data",
///     method = "GET",
///     timeout = 30000,
///     auth_bearer = "your_token",
///     max_depth = 5,
///     serde = true,
///     default = false
/// }
/// ```
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
///
/// This macro parses OpenAPI 3.0/3.1 specifications and generates:
/// - Rust structs for all schemas
/// - API client traits (optional)
/// - Validation derives (optional)
///
/// # Input Sources
///
/// The macro supports three input methods:
///
/// ## 1. Inline Specification
///
/// ```ignore
/// openapi_to_rust! {
///     spec = r#"
/// openapi: 3.0.0
/// info:
///   title: My API
///   version: 1.0.0
/// components:
///   schemas:
///     User:
///       type: object
///       properties:
///         id:
///           type: integer
///         name:
///           type: string
///     "#
/// }
/// ```
///
/// ## 2. From URL (fetched at compile time)
///
/// ```ignore
/// openapi_to_rust! {
///     url = "https://api.example.com/openapi.yaml",
///     timeout = 30000  // optional, milliseconds
/// }
/// ```
///
/// ## 3. From File
///
/// ```ignore
/// openapi_to_rust! {
///     file = "openapi.yaml"
/// }
/// ```
///
/// # Authentication
///
/// When fetching from a URL, you can provide authentication:
///
/// ```ignore
/// // Bearer token
/// openapi_to_rust! {
///     url = "https://api.example.com/openapi.yaml",
///     auth_bearer = "your_token_here"
/// }
///
/// // API key
/// openapi_to_rust! {
///     url = "https://api.example.com/openapi.yaml",
///     auth_api_key = "X-API-Key:your_key"
/// }
///
/// // Basic auth
/// openapi_to_rust! {
///     url = "https://api.example.com/openapi.yaml",
///     auth_basic = "username:password"
/// }
/// ```
///
/// # Customization Options
///
/// ```ignore
/// openapi_to_rust! {
///     file = "openapi.yaml",
///     generate_client = true,      // Generate API client traits (default: true)
///     generate_validation = true,  // Add validation derives (default: true)
///     serde = true,                // Add serde derives (default: true)
///     default = false              // Add Default derive (default: false)
/// }
/// ```
///
/// # Examples
///
/// ## Simple usage
///
/// ```ignore
/// use unistructgen_macro::openapi_to_rust;
///
/// openapi_to_rust! {
///     file = "petstore.yaml"
/// }
///
/// // Now you can use the generated types:
/// let pet = Pet {
///     id: 1,
///     name: "Fluffy".to_string(),
///     status: PetStatus::Available,
/// };
/// ```
///
/// ## With authentication
///
/// ```ignore
/// openapi_to_rust! {
///     url = "https://api.github.com/openapi.yaml",
///     auth_bearer = env!("GITHUB_TOKEN"),
///     timeout = 10000
/// }
/// ```
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
    let agent = ureq::AgentBuilder::new()
        .timeout(std::time::Duration::from_millis(timeout_ms))
        .build();

    let mut request = agent.get(url);

    // Apply authentication if provided
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

    let content = response
        .into_string()
        .map_err(|e| format!("Failed to read response body: {}", e))?;

    if content.is_empty() {
        return Err("Empty response from URL".to_string());
    }

    Ok(content)
}
