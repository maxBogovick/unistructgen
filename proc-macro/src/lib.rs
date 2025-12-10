use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, parse::{Parse, ParseStream}, Token, LitStr, LitBool, Ident};
use unistructgen_codegen::{RenderOptions, RustRenderer};
use unistructgen_json_parser::{JsonParser, ParserOptions};

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

/// Macro input for struct_from_external_api
struct ExternalApiInput {
    struct_name: String,
    url: String,
    method: String,
    consumer: String,
    timeout: u64,
    max_entity_count: Option<usize>,
    max_depth: Option<usize>,
    conflict_strategy: ConflictStrategy,
    serde: bool,
    default: bool,
    optional: bool,
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
        })
    }
}

/// Make HTTP request and fetch JSON
fn fetch_json_from_api(input: &ExternalApiInput) -> Result<String, String> {
    let agent = ureq::AgentBuilder::new()
        .timeout(std::time::Duration::from_millis(input.timeout))
        .build();

    let request = match input.method.as_str() {
        "GET" => agent.get(&input.url),
        "POST" => agent.post(&input.url),
        "PUT" => agent.put(&input.url),
        "DELETE" => agent.delete(&input.url),
        _ => return Err(format!("Unsupported HTTP method: {}", input.method)),
    };

    let response = request
        .call()
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    let json_str = response
        .into_string()
        .map_err(|e| format!("Failed to read response body: {}", e))?;

    Ok(json_str)
}

/// Merge multiple JSON samples to detect optional fields
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
/// # Examples
///
/// ```ignore
/// use unistructgen_macro::struct_from_external_api;
///
/// struct_from_external_api! {
///     struct_name = "User",
///     url_api = "https://jsonplaceholder.typicode.com/users/1"
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

    // Handle max_entity_count for arrays
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
