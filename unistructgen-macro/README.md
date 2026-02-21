# 🔮 UniStructGen Proc Macros

**Compile-time generation of Rust structures with zero runtime overhead**

[![Crate](https://img.shields.io/crates/v/unistructgen-macro.svg)](https://crates.io/crates/unistructgen-macro)
[![Docs](https://docs.rs/unistructgen-macro/badge.svg)](https://docs.rs/unistructgen-macro)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](../LICENSE-MIT)

---

## 📋 Table of Contents

- [Overview](#-overview)
- [Installation](#-installation)
- [Macros](#-macros)
- [generate_struct_from_json!](#-generate_struct_from_json)
- [#[json_struct]](#-json_struct)
- [struct_from_external_api!](#-struct_from_external_api)
- [openapi_to_rust!](#-openapi_to_rust)
- [Authentication](#-authentication)
- [Compile-time Fetch Controls](#-compile-time-fetch-controls)
- [Type Inference](#-type-inference)
- [Examples](#-examples)

---

## 🎯 Overview

`unistructgen-macro` provides procedural macros for generating Rust code at compile time:

### Compile-time Generation Benefits

| Aspect | Compile-time | Runtime |
|--------|--------------|---------|
| **Performance** | Zero overhead | Parsing cost |
| **Type Safety** | Full verification | Partial |
| **IDE Support** | Full autocomplete | Limited |
| **Error Detection** | At compilation | At execution |

### Available Macros

| Macro | Source | Description |
|-------|--------|-------------|
| `generate_struct_from_json!` | Inline JSON | Generates from JSON string |
| `#[json_struct]` | Const string | Attribute for const |
| `struct_from_external_api!` | HTTP API | Loads JSON from API |
| `openapi_to_rust!` | OpenAPI spec | Full generation from OpenAPI |

---

## 📦 Installation

```toml
[dependencies]
unistructgen-macro = "0.2"

# For working with generated code
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# For special types (optional)
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1.0", features = ["serde", "v4"] }
```

---

## 🔧 generate_struct_from_json!

Generates Rust structures from inline JSON.

### Syntax

```rust
generate_struct_from_json! {
    name = "StructName",
    json = r#"{ "field": "value" }"#,
    // Optional parameters:
    serde = true,      // Add serde derives (default: true)
    default = false,   // Add Default derive (default: false)
    optional = false,  // All fields as Option<T> (default: false)
}
```

### Examples

**Basic Usage:**

```rust
use unistructgen_macro::generate_struct_from_json;

generate_struct_from_json! {
    name = "User",
    json = r#"{
        "id": 1,
        "name": "Alice",
        "email": "alice@example.com"
    }"#
}

fn main() {
    let user = User {
        id: 42,
        name: "Bob".to_string(),
        email: "bob@example.com".to_string(),
    };

    let json = serde_json::to_string(&user).unwrap();
    println!("{}", json);
}
```

**With Nested Objects:**

```rust
generate_struct_from_json! {
    name = "Order",
    json = r#"{
        "id": "ORD-001",
        "customer": {
            "name": "Alice",
            "email": "alice@example.com"
        },
        "items": [
            { "product": "Widget", "quantity": 2, "price": 29.99 }
        ],
        "total": 59.98
    }"#
}

// Generates:
// - Customer { name: String, email: String }
// - ItemsItem { product: String, quantity: i64, price: f64 }
// - Order { id: String, customer: Customer, items: Vec<ItemsItem>, total: f64 }
```

**With Default:**

```rust
generate_struct_from_json! {
    name = "Config",
    json = r#"{
        "host": "localhost",
        "port": 8080,
        "debug": false
    }"#,
    default = true
}

let config = Config::default();
```

---

## 🏷️ #[json_struct]

Attribute macro for generation from a const string.

### Syntax

```rust
#[json_struct(name = "StructName", serde = true, default = false)]
const SAMPLE: &str = r#"{ "json": "data" }"#;
```

### Example

```rust
use unistructgen_macro::json_struct;

#[json_struct(name = "Settings", default = true)]
const SETTINGS_SAMPLE: &str = r#"{
    "theme": "dark",
    "font_size": 14,
    "notifications": true
}"#;

fn main() {
    let settings = Settings::default();
    println!("Theme: {}", settings.theme);
}
```

---

## 🌐 struct_from_external_api!

Loads JSON from an external API during compilation.

### Syntax

```rust
struct_from_external_api! {
    struct_name = "StructName",
    url_api = "https://api.example.com/data",

    // HTTP settings:
    method = "GET",           // HTTP method (default: "GET")
    timeout = 30000,          // Timeout in ms (default: 30000)

    // Limits:
    max_depth = 5,            // Maximum nesting depth
    max_entity_count = 100,   // Maximum array elements

    // Generation:
    serde = true,
    default = false,
    optional = false,

    // Authentication (one of):
    auth_bearer = "token",
    auth_api_key = "Header-Name:value",
    auth_basic = "username:password",
}
```

### Examples

**Public API:**

```rust
use unistructgen_macro::struct_from_external_api;

struct_from_external_api! {
    struct_name = "GitHubUser",
    url_api = "https://api.github.com/users/octocat"
}

// GitHubUser generated with all fields from response
```

**API returning an array:**

```rust
struct_from_external_api! {
    struct_name = "Todo",
    url_api = "https://jsonplaceholder.typicode.com/todos"
}

// Macro automatically extracts the first element
// to define the structure
```

**With Authentication:**

```rust
// Bearer Token
struct_from_external_api! {
    struct_name = "UserProfile",
    url_api = "https://api.myservice.com/me",
    auth_bearer = env!("API_TOKEN")
}

// API Key
struct_from_external_api! {
    struct_name = "WeatherData",
    url_api = "https://api.weather.com/current",
    auth_api_key = "X-API-Key:your_key"
}

// Basic Auth
struct_from_external_api! {
    struct_name = "PrivateData",
    url_api = "https://internal.api.com/data",
    auth_basic = "admin:secret"
}
```

---

## 📜 openapi_to_rust!

Generates types and client from an OpenAPI specification.

### Sources

**From file:**

```rust
openapi_to_rust! {
    file = "openapi.yaml"
}
```

**From URL:**

```rust
openapi_to_rust! {
    url = "https://api.example.com/openapi.yaml",
    timeout = 30000,
    auth_bearer = env!("TOKEN")
}
```

**Inline specification:**

```rust
openapi_to_rust! {
    spec = r#"
openapi: 3.0.0
info:
  title: My API
  version: 1.0.0
components:
  schemas:
    User:
      type: object
      required:
        - id
        - email
      properties:
        id:
          type: integer
          format: int64
        email:
          type: string
          format: email
    "#
}
```

### Parameters

```rust
openapi_to_rust! {
    // Source (one of):
    file = "spec.yaml",
    url = "https://...",
    spec = r#"..."#,

    // Generation:
    generate_client = true,      // API client trait
    generate_validation = true,  // #[validate(...)]
    serde = true,
    default = false,

    // For URL:
    timeout = 30000,
    auth_bearer = "token",
    auth_api_key = "Header:value",
    auth_basic = "user:pass",
}
```

### Example

```rust
use unistructgen_macro::openapi_to_rust;

openapi_to_rust! {
    file = "petstore.yaml",
    generate_client = true,
    generate_validation = true
}

fn main() {
    let pet = Pet {
        id: Some(1),
        name: "Fluffy".to_string(),
        status: Some(PetStatus::Available),
    };

    // Validation
    pet.validate().expect("Invalid pet");

    // Serialization
    let json = serde_json::to_string(&pet).unwrap();
}
```

---

## 🔐 Authentication

### Bearer Token

```rust
struct_from_external_api! {
    struct_name = "Data",
    url_api = "https://api.example.com/protected",
    auth_bearer = "eyJhbGciOiJIUzI1NiIs..."
}
// Header: Authorization: Bearer eyJhbGciOiJIUzI1NiIs...
```

### API Key

```rust
struct_from_external_api! {
    struct_name = "Data",
    url_api = "https://api.example.com/data",
    auth_api_key = "X-API-Key:sk_live_abc123"
}
// Header: X-API-Key: sk_live_abc123
```

### Basic Auth

```rust
struct_from_external_api! {
    struct_name = "Data",
    url_api = "https://api.example.com/secure",
    auth_basic = "admin:password123"
}
// Header: Authorization: Basic YWRtaW46cGFzc3dvcmQxMjM=
```

### Environment Variables

```rust
struct_from_external_api! {
    struct_name = "Data",
    url_api = "https://api.example.com/data",
    auth_bearer = env!("MY_API_TOKEN")
}

// Before compilation:
// export MY_API_TOKEN="your_secret_token"
```

---

## 🌐 Compile-time Fetch Controls

For macros that access the network during compilation (`struct_from_external_api!`, `openapi_to_rust!` with `url`, `env_file` over HTTP), the following environment variables are available:

- `UNISTRUCTGEN_FETCH_OFFLINE=1` — disable network, use cache only
- `UNISTRUCTGEN_FETCH_CACHE=0` — disable cache
- `UNISTRUCTGEN_FETCH_CACHE_DIR=/path` — path to cache directory
- `UNISTRUCTGEN_FETCH_TIMEOUT_MS=60000` — timeout (ms)

---

## 🔍 Type Inference

### Automatic Type Detection

| JSON | Pattern | Rust Type |
|------|---------|-----------|
| Number (int) | - | `i64` / `u64` |
| Number (float) | - | `f64` |
| String | ISO 8601 datetime | `chrono::DateTime<Utc>` |
| String | UUID format | `uuid::Uuid` |
| String | other | `String` |
| Boolean | - | `bool` |
| Array | - | `Vec<T>` |
| Object | - | Nested struct |
| Null | - | `Option<serde_json::Value>` |

### Field Name Sanitization

| Original | Result |
|----------|-----------|
| `camelCase` | `camel_case` + `#[serde(rename = "camelCase")]` |
| `PascalCase` | `pascal_case` + rename |
| `kebab-case` | `kebab_case` + rename |
| `type` (keyword) | `type_` |

---

## 📝 Examples

### E-commerce

```rust
generate_struct_from_json! {
    name = "Product",
    json = r#"{
        "id": "prod_123",
        "name": "Wireless Headphones",
        "price": 79.99,
        "in_stock": true,
        "categories": ["electronics", "audio"]
    }"#
}
```

### Configuration

```rust
#[json_struct(name = "AppConfig", default = true)]
const CONFIG: &str = r#"{
    "server": { "host": "0.0.0.0", "port": 8080 },
    "database": { "url": "postgres://localhost/db" }
}"#;
```

### External API

```rust
struct_from_external_api! {
    struct_name = "WeatherForecast",
    url_api = "https://api.weather.com/current?city=NYC",
    auth_api_key = "X-API-Key:your_key"
}
```

### OpenAPI Full

```rust
openapi_to_rust! {
    file = "api-spec.yaml",
    generate_client = true,
    generate_validation = true
}

async fn example() -> Result<(), ApiError> {
    let client = ApiClient::new("https://api.example.com");
    let users = client.list_users(Some(10)).await?;
    Ok(())
}
```

---

## ⚠️ Limitations

1. **Compile-time network** — `struct_from_external_api!` performs HTTP requests during compilation
2. **Static data** — types are fixed at compile time
3. **Secrets** — use `env!()` for environment variables

---

## 🔗 Related Modules

- [unistructgen (Core)](../README.md) — IR, traits, pipeline
- [unistructgen (Codegen)](../README.md) — Rust generator
- [unistructgen (Parsers)](../README.md) — JSON/OpenAPI parsers
- [unistructgen-cli](../unistructgen-cli/README.md) — CLI tool

---

## 📜 License

MIT or Apache-2.0 — at your option.
