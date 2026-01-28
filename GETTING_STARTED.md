# Getting Started with UniStructGen

UniStructGen makes it easy to generate type-safe Rust structs from various sources. This guide will walk you through the most common use cases.

## Table of Contents

1. [Quick Start with Proc Macros](#quick-start-with-proc-macros)
2. [External API Integration](#external-api-integration)
3. [CLI Usage](#cli-usage)
4. [Advanced Features](#advanced-features)

## Quick Start with Proc Macros

The fastest way to get started is using the procedural macros for compile-time code generation.

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
unistructgen-macro = "0.1"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

### Basic Usage

```rust
use unistructgen_macro::generate_struct_from_json;

// Generate a struct from inline JSON
generate_struct_from_json! {
    name = "User",
    json = r#"{
        "id": 42,
        "name": "Alice",
        "email": "alice@example.com",
        "is_active": true
    }"#
}

fn main() {
    // Now you can use the generated struct!
    let user = User {
        id: 1,
        name: "Bob".to_string(),
        email: "bob@example.com".to_string(),
        is_active: true,
    };

    println!("User: {} ({})", user.name, user.email);
}
```

### With Nested Objects

UniStructGen automatically generates nested structs:

```rust
generate_struct_from_json! {
    name = "Product",
    json = r#"{
        "id": 1,
        "name": "Laptop",
        "price": 999.99,
        "manufacturer": {
            "name": "TechCorp",
            "country": "USA"
        }
    }"#
}

// This generates both Product and Manufacturer structs!
```

## External API Integration

Generate structs directly from API responses at compile time:

```rust
use unistructgen_macro::struct_from_external_api;

// Fetch and generate from a real API during compilation
struct_from_external_api! {
    struct_name = "GithubUser",
    url_api = "https://api.github.com/users/octocat"
}

fn main() {
    // The GithubUser struct is now available with all fields from the API
    let user = GithubUser {
        login: "octocat".to_string(),
        id: 1,
        // ... all other fields from the GitHub API response
    };
}
```

### Advanced API Options

```rust
struct_from_external_api! {
    struct_name = "Post",
    url_api = "https://jsonplaceholder.typicode.com/posts/1",

    // Optional: Limit nested depth
    max_depth = 3,

    // Optional: Custom timeout in milliseconds
    request_timeout = 5000,

    // Optional: Make all fields optional
    optional = true,

    // Optional: Add Default derive
    default = true
}
```

## CLI Usage

For build scripts or CI/CD pipelines, use the CLI tool:

### Installation

```bash
cargo install unistructgen
```

### Basic Usage

```bash
# Generate from a JSON file
unistructgen generate --input user.json --name User --output src/models/user.rs

# With optional fields
unistructgen generate --input user.json --name User --optional

# With additional derives
unistructgen generate --input user.json --name User --serde true --default true
```

### Example: Build Script Integration

Create `build.rs`:

```rust
use std::process::Command;

fn main() {
    // Generate structs during build
    Command::new("unistructgen")
        .args(&[
            "generate",
            "--input", "schemas/api_response.json",
            "--name", "ApiResponse",
            "--output", "src/generated/api.rs"
        ])
        .status()
        .expect("Failed to generate structs");

    println!("cargo:rerun-if-changed=schemas/api_response.json");
}
```

## Advanced Features

### Smart Type Inference

UniStructGen automatically detects and converts special types:

```json
{
  "id": 12345,
  "uuid": "550e8400-e29b-41d4-a716-446655440000",
  "created_at": "2023-01-15T10:30:00Z",
  "price": 19.99,
  "count": 42,
  "is_active": true,
  "tags": ["rust", "codegen"],
  "metadata": {
    "key": "value"
  }
}
```

Generates:

```rust
pub struct Generated {
    pub id: i64,
    pub uuid: uuid::Uuid,                    // Detected UUID
    pub created_at: chrono::DateTime<Utc>,  // Detected DateTime
    pub price: f64,
    pub count: i64,
    pub is_active: bool,
    pub tags: Vec<String>,                   // Detected array
    pub metadata: Metadata,                  // Nested struct
}
```

### Customizing Field Types

You can customize the generated code by editing after generation or using transformers (advanced).

### Working with Arrays

When your JSON contains arrays, UniStructGen analyzes the first element:

```rust
generate_struct_from_json! {
    name = "UserList",
    json = r#"{
        "users": [
            {
                "id": 1,
                "name": "Alice"
            },
            {
                "id": 2,
                "name": "Bob"
            }
        ]
    }"#
}

// Generates:
// struct UserList { users: Vec<User> }
// struct User { id: i64, name: String }
```

### Field Name Conversion

UniStructGen automatically converts field names to Rust conventions:

- `camelCase` → `snake_case`
- `PascalCase` → `snake_case`
- Preserves `snake_case`

Using `#[serde(rename = "...")]` to maintain original names.

## Best Practices

### 1. Use Proc Macros for Static Data

When your schema is known at compile time, use proc macros:

```rust
// ✅ Good - compile-time validation
generate_struct_from_json! {
    name = "Config",
    json = r#"{"host": "localhost", "port": 8080}"#
}
```

### 2. Use External API Macros Cautiously

External API calls happen at compile time. Use them for:
- Schema exploration
- Development/prototyping
- When the API schema is stable

```rust
// ⚠️ Runs during every compilation
struct_from_external_api! {
    struct_name = "Schema",
    url_api = "https://api.example.com/schema"
}
```

### 3. Use CLI for Build Pipelines

For production builds, generate code once and commit:

```bash
# Generate and commit the code
unistructgen generate --input schema.json --output src/models.rs
git add src/models.rs
```

### 4. Handle Optional Fields

If your API may omit fields, use the `optional` flag:

```rust
struct_from_external_api! {
    struct_name = "ApiResponse",
    url_api = "https://api.example.com/data",
    optional = true  // Makes all fields Option<T>
}
```

## Troubleshooting

### Compilation Errors

If you get compilation errors about missing types:

1. Add required dependencies to `Cargo.toml`:
```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = { version = "0.4", features = ["serde"] }  # For DateTime
uuid = { version = "1.0", features = ["serde", "v4"] }  # For UUID
```

### API Timeout

If external API calls timeout:

```rust
struct_from_external_api! {
    struct_name = "Data",
    url_api = "https://slow-api.example.com/data",
    request_timeout = 30000  // 30 seconds
}
```

## Next Steps

- Check out the [examples](examples/) directory for more use cases
- Read the [API documentation](https://docs.rs/unistructgen)
- Explore advanced features like custom transformers
- Join our community discussions

## Need Help?

- 📖 [Full Documentation](https://docs.rs/unistructgen)
- 💬 [GitHub Discussions](https://github.com/maxBogovick/unistructgen/discussions)
- 🐛 [Report Issues](https://github.com/maxBogovick/unistructgen/issues)
