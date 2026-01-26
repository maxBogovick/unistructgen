# 🚀 UniStructGen

<div align="center">

**Transform JSON into Type-Safe Rust Structs — At Compile Time**

[![Crates.io](https://img.shields.io/crates/v/unistructgen?style=flat-square)](https://crates.io/crates/unistructgen)
[![Documentation](https://img.shields.io/docsrs/unistructgen?style=flat-square)](https://docs.rs/unistructgen)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue?style=flat-square)](LICENSE)
[![Build Status](https://img.shields.io/github/workflow/status/yourusername/unistructgen/CI?style=flat-square)](https://github.com/yourusername/unistructgen/actions)

[Quick Start](#-quick-start) • [Examples](#-real-world-examples) • [Documentation](QUICKSTART.md) • [API Docs](https://docs.rs/unistructgen)

</div>

---

## 💡 Why UniStructGen?

Stop writing boilerplate structs by hand. Stop wrestling with `serde_json::Value`. Stop maintaining types that drift out of sync with your APIs.

**UniStructGen generates perfectly typed Rust structs from JSON — automatically, at compile time, with zero runtime overhead.**

### The Problem

```rust
// ❌ Before: Manual struct definition, error-prone, tedious
#[derive(Deserialize)]
struct User {
    pub id: i64,                    // Is this i64 or u64?
    pub name: String,
    pub email: String,
    pub created_at: String,         // Should this be DateTime?
    // Did the API add new fields? Who knows! 🤷
}

// Parsing untyped JSON
let data: serde_json::Value = serde_json::from_str(json)?;
let id = data["user"]["id"].as_i64().unwrap();  // 💥 Runtime panic waiting to happen
```

### The Solution

```rust
// ✅ After: One line. Compile-time safe. Always in sync.
use unistructgen_macro::struct_from_external_api;

struct_from_external_api! {
    struct_name = "User",
    url_api = "https://api.example.com/users/1"
}

// That's it! Fully typed struct generated at compile time:
// - Smart type detection (DateTime, UUID, etc.)
// - Automatic serde derives
// - Field name conversion (camelCase → snake_case)
// - Nested object support
// - Array handling
```

---

## ✨ Key Features

<table>
<tr>
<td width="50%">

### 🎯 **Compile-Time Magic**
Generate structs during compilation. Zero runtime overhead, maximum type safety.

### 🌐 **Live API Integration**
Fetch schemas from external APIs at build time. Always stay in sync.

### 🔒 **Authentication Support**
Bearer tokens, API keys, and Basic Auth — secure API access built-in.

### 🧠 **Smart Type Inference**
Automatically detects UUIDs, DateTimes, emails, URLs, and more.

</td>
<td width="50%">

### 🔄 **Array Auto-Detection**
Returns an array? We automatically extract the item type.

### 🎨 **Beautiful Code Gen**
Clean, idiomatic Rust with proper formatting and documentation.

### 🛠️ **CLI + Macros**
Use as proc-macros or a standalone CLI tool — your choice.

</td>
</tr>
</table>

---

## ⚡ Quick Start

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
unistructgen-macro = "0.1"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Optional: for advanced types
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1.0", features = ["serde", "v4"] }
```

### 30-Second Demo

```rust
use unistructgen_macro::generate_struct_from_json;

// Define once, use everywhere
generate_struct_from_json! {
    name = "Product",
    json = r#"{
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "name": "Laptop",
        "price": 999.99,
        "inStock": true,
        "createdAt": "2024-01-15T10:30:00Z"
    }"#
}

fn main() {
    let product = Product {
        id: uuid::Uuid::new_v4(),
        name: "Gaming Mouse".to_string(),
        price: 49.99,
        in_stock: true,  // Auto-converted from 'inStock'
        created_at: chrono::Utc::now(),
    };

    println!("{}", serde_json::to_string_pretty(&product).unwrap());
}
```

**Generated code:**
```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Product {
    pub id: uuid::Uuid,
    pub name: String,
    pub price: f64,
    #[serde(rename = "inStock")]
    pub in_stock: bool,
    #[serde(rename = "createdAt")]
    pub created_at: chrono::DateTime<chrono::Utc>,
}
```

---

## 🎯 Real-World Examples

### Example 1: API Client Development

```rust
use unistructgen_macro::struct_from_external_api;

// Generate from GitHub API
struct_from_external_api! {
    struct_name = "Repository",
    url_api = "https://api.github.com/repos/rust-lang/rust"
}

// Generate from JSONPlaceholder
struct_from_external_api! {
    struct_name = "Post",
    url_api = "https://jsonplaceholder.typicode.com/posts/1"
}

fn main() {
    // Use your perfectly typed structs!
    let repo = Repository { /* ... */ };
    let post = Post { /* ... */ };
}
```

### Example 2: Array Responses (Auto-Detected!)

```rust
// API returns an array? No problem!
struct_from_external_api! {
    struct_name = "Todo",
    url_api = "https://jsonplaceholder.typicode.com/todos"
    // Automatically extracts first element to infer structure
}

// Generated:
// pub struct Todo {
//     pub user_id: i64,
//     pub id: i64,
//     pub title: String,
//     pub completed: bool,
// }
```

### Example 3: Configuration Files

```rust
#[json_struct(name = "Config")]
const SCHEMA: &str = r#"{
    "database": {
        "host": "localhost",
        "port": 5432,
        "ssl": true
    },
    "api": {
        "baseUrl": "https://api.example.com",
        "timeout": 30000
    }
}"#;

fn main() {
    let config = Config {
        database: Database {
            host: "prod.example.com".to_string(),
            port: 5432,
            ssl: true,
        },
        api: Api {
            base_url: "https://api.example.com".to_string(),
            timeout: 30000,
        },
    };
}
```

### Example 4: API Authentication 🔒

```rust
// Bearer Token (OAuth2, JWT)
struct_from_external_api! {
    struct_name = "User",
    url_api = "https://api.example.com/user",
    auth_bearer = "your_bearer_token_here"
}

// API Key in Custom Header
struct_from_external_api! {
    struct_name = "Data",
    url_api = "https://api.example.com/data",
    auth_api_key = "X-API-Key:your_api_key_here"
}

// HTTP Basic Authentication
struct_from_external_api! {
    struct_name = "Resource",
    url_api = "https://api.example.com/resource",
    auth_basic = "username:password"
}
```

### Example 5: Advanced Options

```rust
struct_from_external_api! {
    struct_name = "ApiResponse",
    url_api = "https://api.example.com/data",

    // Authentication
    auth_bearer = "your_token",   // Bearer token auth

    // Customization
    timeout = 10000,              // Request timeout (ms)
    max_depth = 5,                // Limit nesting depth
    optional = true,              // Make fields Option<T>
    default = true,               // Add Default derive
    serde = true,                 // Serde derives (default)
}
```

---

## 🎪 Use Cases

<table>
<tr>
<td>

### 🌐 **API Clients**
Generate types from REST APIs, GraphQL schemas, or any JSON endpoint.

### ⚙️ **Microservices**
Keep service contracts in sync by generating from shared schemas.

### 📊 **Data Pipelines**
Type-safe ETL processes with validated data structures.

</td>
<td>

### 🧪 **Testing**
Generate mock data structures from API fixtures.

### 📝 **Documentation**
Auto-generate types from OpenAPI/Swagger specs.

### 🔄 **Schema Evolution**
Stay in sync with evolving external APIs automatically.

</td>
</tr>
</table>

---

## 📦 Three Ways to Use

### 1️⃣ Proc Macro (Recommended)

Perfect for schemas known at compile time:

```rust
use unistructgen_macro::generate_struct_from_json;

generate_struct_from_json! {
    name = "User",
    json = r#"{"id": 1, "name": "Alice"}"#
}
```

**Pros:**
- ✅ Zero runtime overhead
- ✅ Compile-time validation
- ✅ IDE autocomplete
- ✅ Type checking

### 2️⃣ External API Macro

Fetch schemas from live endpoints:

```rust
struct_from_external_api! {
    struct_name = "User",
    url_api = "https://api.example.com/schema"
}
```

**Pros:**
- ✅ Always in sync with API
- ✅ One-line integration
- ✅ Compile-time fetching
- ✅ No build scripts needed

### 3️⃣ CLI Tool

For build pipelines and pre-generation:

```bash
# Install
cargo install unistructgen

# Generate from file
unistructgen generate -i schema.json -o models.rs -n User

# Generate from URL
curl https://api.example.com/schema | unistructgen generate -n User

# Watch mode (coming soon)
unistructgen watch -i schema.json -o models.rs
```

**Pros:**
- ✅ Language-agnostic
- ✅ CI/CD integration
- ✅ Commit generated code
- ✅ Review changes in PRs

---

## 🔥 What Makes It Special?

### Smart Type Detection

UniStructGen doesn't just map JSON types to Rust primitives. It **understands** your data:

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "email": "user@example.com",
  "website": "https://example.com",
  "created": "2024-01-15T10:30:00Z",
  "tags": ["rust", "codegen"]
}
```

**Generates:**
```rust
pub struct Item {
    pub id: uuid::Uuid,              // ✨ Detected as UUID
    pub email: String,               // Could add email validation
    pub website: url::Url,           // ✨ Detected as URL
    pub created: chrono::DateTime<chrono::Utc>, // ✨ Detected as DateTime
    pub tags: Vec<String>,
}
```

### Automatic Array Handling

```rust
// API returns: [{"id": 1, "name": "Todo 1"}, {"id": 2, "name": "Todo 2"}]
struct_from_external_api! {
    struct_name = "Todo",
    url_api = "https://api.example.com/todos"
}
// Automatically extracts item structure from array!
```

### Nested Object Support

```json
{
  "user": {
    "profile": {
      "address": {
        "city": "New York"
      }
    }
  }
}
```

**Generates:**
```rust
pub struct Root {
    pub user: User,
}

pub struct User {
    pub profile: Profile,
}

pub struct Profile {
    pub address: Address,
}

pub struct Address {
    pub city: String,
}
```

### Field Name Sanitization

Automatically converts JSON naming to Rust conventions:

| JSON Field | Rust Field | Attribute |
|------------|------------|-----------|
| `userName` | `user_name` | `#[serde(rename = "userName")]` |
| `user-id` | `user_id` | `#[serde(rename = "user-id")]` |
| `123field` | `_123field` | - |
| `type` | `type_` | - (keyword) |

---

## 📊 Comparison

| Feature | UniStructGen | serde_json::Value | quicktype | json2rust |
|---------|--------------|-------------------|-----------|-----------|
| **Compile-time generation** | ✅ | ❌ | ❌ | ❌ |
| **Live API fetching** | ✅ | ❌ | ✅ | ❌ |
| **Zero runtime cost** | ✅ | ❌ | ✅ | ✅ |
| **Smart type detection** | ✅ | ❌ | ✅ | ⚠️ |
| **Proc macro support** | ✅ | ❌ | ❌ | ❌ |
| **Array auto-detection** | ✅ | ❌ | ✅ | ❌ |
| **Rust-specific** | ✅ | ✅ | ❌ (multi-lang) | ✅ |
| **Nested objects** | ✅ | ⚠️ | ✅ | ✅ |
| **CLI + Library** | ✅ | ❌ | ✅ (CLI only) | ✅ (web) |

---

## 🎓 Learning Resources

- 📖 **[Quick Start Guide](QUICKSTART.md)** - Get started in 5 minutes
- 📚 **[Complete Examples](EXAMPLES.md)** - Real-world usage patterns
- 🔧 **[API Documentation](https://docs.rs/unistructgen)** - Full API reference
- 🌐 **[External API Guide](docs/EXTERNAL_API_GUIDE.md)** - Advanced API integration
- 🎯 **[Best Practices](docs/BEST_PRACTICES.md)** - Tips and tricks

---

## 🏗️ Architecture

UniStructGen follows a clean, modular architecture:

```
┌─────────────────────────────────────────────────────┐
│                   Your Code                         │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────┐  │
│  │ Proc Macros  │  │     CLI      │  │   API    │  │
│  └──────┬───────┘  └──────┬───────┘  └────┬─────┘  │
└─────────┼──────────────────┼───────────────┼────────┘
          │                  │               │
┌─────────▼──────────────────▼───────────────▼────────┐
│              Core Pipeline                           │
│  ┌───────┐   ┌──────┐   ┌─────────┐   ┌─────────┐  │
│  │Parser │──▶│  IR  │──▶│ Codegen │──▶│  Output │  │
│  └───────┘   └──────┘   └─────────┘   └─────────┘  │
└──────────────────────────────────────────────────────┘
```

**Modules:**
- `core` - Intermediate Representation (IR) and traits
- `parsers/json_parser` - JSON → IR conversion
- `codegen` - IR → Rust code generation
- `proc-macro` - Procedural macros
- `cli` - Command-line interface

---

## 🚀 Quick Start Examples

### Example A: From Inline JSON

```rust
use unistructgen_macro::generate_struct_from_json;

generate_struct_from_json! {
    name = "Person",
    json = r#"{"name": "Alice", "age": 30}"#
}

let person = Person {
    name: "Bob".to_string(),
    age: 25,
};
```

### Example B: From External API

```rust
use unistructgen_macro::struct_from_external_api;

struct_from_external_api! {
    struct_name = "GithubUser",
    url_api = "https://api.github.com/users/octocat"
}

// Fully typed struct ready to use!
```

### Example C: From File (CLI)

```bash
# Create schema.json
echo '{"id": 1, "title": "Hello"}' > schema.json

# Generate
unistructgen generate -i schema.json -n Post -o post.rs

# Use in your code
# mod post;
# use post::Post;
```

### Example D: Attribute Macro

```rust
use unistructgen_macro::json_struct;

#[json_struct(name = "Settings")]
const CONFIG_SCHEMA: &str = r#"{
    "debug": true,
    "maxConnections": 100
}"#;

// Settings struct automatically generated
```

---

## 💼 Production Ready

UniStructGen is designed for production use:

### ✅ Type Safety
All generated code is fully typed and checked at compile time.

### ✅ Performance
Zero runtime overhead - all generation happens at compile time.

### ✅ Reliability
Comprehensive test suite with 100+ tests covering edge cases.

### ✅ Maintainability
Clean, idiomatic Rust output that's easy to read and modify.

### ✅ Flexibility
Customize derives, field types, and naming conventions.

---

## 📈 Roadmap

### ✅ v0.1 - **Current**
- ✅ JSON parsing and struct generation
- ✅ Proc macro support (function-like & attribute)
- ✅ External API integration
- ✅ Authentication support (Bearer, API Key, Basic)
- ✅ Smart type inference
- ✅ Array auto-detection
- ✅ CLI tool

### 🎯 v0.2 - **Next**
- [ ] Merge multiple JSON samples
- [ ] OpenAPI/Swagger support
- [ ] GraphQL schema support
- [ ] Watch mode for file changes
- [ ] Builder pattern generation
- [ ] Validation derives

### 🔮 v1.0 - **Future**
- [ ] Markdown table parsing
- [ ] SQL DDL parsing
- [ ] TypeScript definitions export
- [ ] VSCode extension
- [ ] Web playground
- [ ] Plugin system

---

## 🤝 Contributing

We welcome contributions! Here's how you can help:

1. 🐛 **Report Bugs** - Open an issue with reproduction steps
2. 💡 **Suggest Features** - Share your ideas in discussions
3. 📝 **Improve Docs** - Help make docs clearer
4. 🔧 **Submit PRs** - Check [CONTRIBUTING.md](CONTRIBUTING.md)

### Development Setup

```bash
# Clone repository
git clone https://github.com/yourusername/unistructgen
cd unistructgen

# Build
cargo build --workspace

# Test
cargo test --workspace

# Run examples
cargo run --example api-example
```

---

## 📜 License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.

---

## 🙏 Acknowledgments

Built with ❤️ by the Rust community.

Special thanks to:
- The [serde](https://serde.rs/) team for JSON serialization inspiration
- [quicktype](https://quicktype.io/) for schema generation ideas
- All contributors and users of UniStructGen

---

<div align="center">

**[⭐ Star us on GitHub](https://github.com/yourusername/unistructgen)** • **[📦 View on crates.io](https://crates.io/crates/unistructgen)** • **[💬 Join Discussions](https://github.com/yourusername/unistructgen/discussions)**

Made with 🦀 by Rust developers, for Rust developers.

</div>
