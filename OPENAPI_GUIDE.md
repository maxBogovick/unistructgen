# 🚀 OpenAPI Support Guide

Complete guide to using UniStructGen's OpenAPI/Swagger integration for enterprise-grade API type generation.

## Table of Contents

1. [Quick Start](#quick-start)
2. [Installation](#installation)
3. [Basic Usage](#basic-usage)
4. [Advanced Features](#advanced-features)
5. [Real-World Examples](#real-world-examples)
6. [Best Practices](#best-practices)
7. [Troubleshooting](#troubleshooting)

---

## Quick Start

### 5-Minute Example

```rust
use unistructgen_macro::openapi_to_rust;

// Generate types from your OpenAPI spec
openapi_to_rust! {
    file = "openapi.yaml"
}

fn main() {
    // Use the generated types immediately
    let user = User {
        id: 1,
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
        age: Some(30),
    };

    // Automatic validation
    user.validate().expect("Valid user");

    // JSON serialization
    let json = serde_json::to_string_pretty(&user).unwrap();
    println!("{}", json);
}
```

---

## Installation

### Step 1: Add Dependencies

```toml
[dependencies]
# Core UniStructGen with OpenAPI support
unistructgen-macro = "0.1"
unistructgen-openapi-parser = "0.1"

# Required for generated types
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# For validation (optional but recommended)
validator = { version = "0.16", features = ["derive"] }

# For special types
uuid = { version = "1.0", features = ["serde", "v4"] }
chrono = { version = "0.4", features = ["serde"] }
```

### Step 2: Create OpenAPI Spec

Create `openapi.yaml`:

```yaml
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
        - name
      properties:
        id:
          type: integer
        name:
          type: string
        email:
          type: string
          format: email
```

### Step 3: Generate Types

```rust
use unistructgen_macro::openapi_to_rust;

openapi_to_rust! {
    file = "openapi.yaml"
}

// Types are now available!
```

---

## Basic Usage

### From File

```rust
openapi_to_rust! {
    file = "openapi.yaml"
}
```

### From URL

```rust
openapi_to_rust! {
    url = "https://api.example.com/openapi.yaml",
    timeout = 30000  // 30 seconds
}
```

### From Inline Spec

```rust
openapi_to_rust! {
    spec = r#"
openapi: 3.0.0
info:
  title: Inline API
  version: 1.0.0
components:
  schemas:
    Product:
      type: object
      properties:
        id:
          type: integer
        name:
          type: string
    "#
}
```

---

## Advanced Features

### 1. Authentication

#### Bearer Token (OAuth2, JWT)

```rust
openapi_to_rust! {
    url = "https://api.example.com/openapi.yaml",
    auth_bearer = env!("API_TOKEN")
}
```

#### API Key

```rust
openapi_to_rust! {
    url = "https://api.example.com/openapi.yaml",
    auth_api_key = "X-API-Key:your_api_key_here"
}
```

#### Basic Authentication

```rust
openapi_to_rust! {
    url = "https://api.example.com/openapi.yaml",
    auth_basic = "username:password"
}
```

### 2. Configuration Options

```rust
openapi_to_rust! {
    file = "openapi.yaml",

    // API client generation (default: true)
    generate_client = true,

    // Validation derives (default: true)
    generate_validation = true,

    // Serde derives (default: true)
    serde = true,

    // Default derive (default: false)
    default = false
}
```

### 3. Using as Library

```rust
use unistructgen_core::Parser;
use unistructgen_openapi_parser::{OpenApiParser, OpenApiParserOptions};
use unistructgen_codegen::RustRenderer;

fn generate_types_from_spec(spec_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Configure parser
    let options = OpenApiParserOptions::builder()
        .generate_client(true)
        .generate_validation(true)
        .derive_serde(true)
        .build();

    let mut parser = OpenApiParser::new(options);

    // Parse spec
    let spec_content = std::fs::read_to_string(spec_path)?;
    let ir_module = parser.parse(&spec_content)?;

    // Generate code
    let renderer = RustRenderer::default();
    let code = renderer.generate(&ir_module)?;

    Ok(code)
}
```

---

## Real-World Examples

### Example 1: GitHub API

```rust
// Generate types from GitHub's OpenAPI spec
openapi_to_rust! {
    url = "https://raw.githubusercontent.com/github/rest-api-description/main/descriptions/api.github.com/api.github.com.yaml",
    timeout = 60000  // GitHub spec is large
}

fn main() {
    let repo = Repository {
        id: 1,
        name: "unistructgen".to_string(),
        full_name: "yourorg/unistructgen".to_string(),
        private: Some(false),
        // ... other fields
    };

    println!("Repository: {:#?}", repo);
}
```

### Example 2: Pet Store (Complete Example)

```yaml
# petstore.yaml
openapi: 3.0.0
info:
  title: Pet Store API
  version: 1.0.0
components:
  schemas:
    Pet:
      type: object
      required:
        - id
        - name
      properties:
        id:
          type: string
          format: uuid
        name:
          type: string
          minLength: 1
          maxLength: 100
        status:
          $ref: '#/components/schemas/PetStatus'
        age:
          type: integer
          minimum: 0
          maximum: 50

    PetStatus:
      type: string
      enum:
        - available
        - pending
        - sold

    NewPet:
      type: object
      required:
        - name
      properties:
        name:
          type: string
        breed:
          type: string
```

```rust
use unistructgen_macro::openapi_to_rust;
use uuid::Uuid;

openapi_to_rust! {
    file = "petstore.yaml"
}

fn main() {
    // Create a pet
    let pet = Pet {
        id: Uuid::new_v4(),
        name: "Fluffy".to_string(),
        status: Some(PetStatus::Available),
        age: Some(3),
    };

    // Validate
    pet.validate().expect("Pet is valid");

    // Serialize
    let json = serde_json::to_string_pretty(&pet).unwrap();
    println!("Pet JSON:\n{}", json);

    // Create new pet request
    let new_pet = NewPet {
        name: "Max".to_string(),
        breed: Some("Golden Retriever".to_string()),
    };

    println!("New pet: {:#?}", new_pet);
}
```

### Example 3: Authenticated Enterprise API

```rust
// Fetch from private API with authentication
openapi_to_rust! {
    url = "https://internal.company.com/api/v1/openapi.yaml",
    auth_bearer = env!("COMPANY_API_TOKEN"),
    timeout = 30000,
    generate_client = true,
    generate_validation = true
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    // Create user
    let new_user = CreateUserRequest {
        username: "john_doe".to_string(),
        email: "john@example.com".to_string(),
        role: UserRole::Admin,
    };

    // Validate before sending
    new_user.validate()?;

    // Send request
    let response = client
        .post("https://internal.company.com/api/v1/users")
        .bearer_auth(env!("COMPANY_API_TOKEN"))
        .json(&new_user)
        .send()
        .await?;

    let user: User = response.json().await?;
    println!("Created user: {:#?}", user);

    Ok(())
}
```

### Example 4: Multi-Service Architecture

```rust
// Service 1: User Service
openapi_to_rust! {
    file = "services/user-service/openapi.yaml",
    generate_client = true
}

// Service 2: Order Service
openapi_to_rust! {
    file = "services/order-service/openapi.yaml",
    generate_client = true
}

// Service 3: Payment Service
openapi_to_rust! {
    file = "services/payment-service/openapi.yaml",
    generate_client = true
}

async fn create_order_workflow() -> Result<(), Box<dyn std::error::Error>> {
    // Use types from multiple services
    let user = user_service::User { /* ... */ };
    let order = order_service::Order { /* ... */ };
    let payment = payment_service::Payment { /* ... */ };

    // All types are properly validated and type-safe
    Ok(())
}
```

---

## Best Practices

### 1. Use Environment Variables for Secrets

```rust
// ❌ Bad: Hardcoded tokens
openapi_to_rust! {
    url = "https://api.example.com/openapi.yaml",
    auth_bearer = "hardcoded_token_123"  // Don't do this!
}

// ✅ Good: Use environment variables
openapi_to_rust! {
    url = "https://api.example.com/openapi.yaml",
    auth_bearer = env!("API_TOKEN")
}
```

### 2. Cache Downloaded Specs

For CI/CD or frequent builds:

```bash
# Download spec once
curl -H "Authorization: Bearer $API_TOKEN" \
  https://api.example.com/openapi.yaml \
  -o openapi.yaml

# Use cached file
```

```rust
openapi_to_rust! {
    file = "openapi.yaml"  // Use cached file
}
```

### 3. Organize Generated Types

```rust
// src/api/mod.rs
pub mod types {
    use unistructgen_macro::openapi_to_rust;

    openapi_to_rust! {
        file = "../openapi.yaml"
    }
}

pub use types::*;
```

### 4. Validation in Production

```rust
use validator::Validate;

fn create_user(data: CreateUserRequest) -> Result<User, ValidationError> {
    // Always validate input
    data.validate()?;

    // Process validated data
    let user = save_to_database(data)?;

    // Validate output
    user.validate()?;

    Ok(user)
}
```

### 5. Version Your Specs

```
specs/
├── v1/
│   └── openapi.yaml
├── v2/
│   └── openapi.yaml
└── v3/
    └── openapi.yaml
```

```rust
#[cfg(feature = "api-v1")]
openapi_to_rust! {
    file = "specs/v1/openapi.yaml"
}

#[cfg(feature = "api-v2")]
openapi_to_rust! {
    file = "specs/v2/openapi.yaml"
}
```

---

## Troubleshooting

### Problem: "Failed to parse OpenAPI specification"

**Solution:** Validate your OpenAPI spec:

```bash
# Using Swagger CLI
swagger-cli validate openapi.yaml

# Using online validator
# https://editor.swagger.io/
```

### Problem: "Failed to fetch from URL"

**Possible causes:**
1. Network connectivity
2. Authentication required
3. Timeout too short

**Solutions:**

```rust
// Increase timeout
openapi_to_rust! {
    url = "https://api.example.com/openapi.yaml",
    timeout = 60000  // 60 seconds
}

// Add authentication
openapi_to_rust! {
    url = "https://api.example.com/openapi.yaml",
    auth_bearer = env!("API_TOKEN")
}
```

### Problem: "Circular reference detected"

**Cause:** Your OpenAPI spec has circular $ref references.

**Solution:** Review your spec for circular dependencies:

```yaml
# ❌ Circular reference
User:
  properties:
    friend:
      $ref: '#/components/schemas/User'  # Points to itself

# ✅ Better design
User:
  properties:
    friendId:
      type: string
      format: uuid
```

### Problem: "Type not found"

**Cause:** Generated types are not in scope.

**Solution:**

```rust
// Make sure the macro is in the right scope
mod api {
    use unistructgen_macro::openapi_to_rust;

    openapi_to_rust! {
        file = "openapi.yaml"
    }
}

// Use with full path
use api::User;

// Or re-export
pub use api::*;
```

### Problem: "Validation errors"

**Solution:** Check constraints in OpenAPI spec:

```yaml
username:
  type: string
  minLength: 3  # Make sure values meet constraints
  maxLength: 20
  pattern: '^[a-zA-Z0-9_]+$'
```

```rust
// Validate before use
let user = User { /* ... */ };
match user.validate() {
    Ok(_) => println!("Valid!"),
    Err(e) => println!("Validation errors: {}", e),
}
```

---

## Support

- **Documentation:** [docs.rs/unistructgen](https://docs.rs/unistructgen)
- **Issues:** [GitHub Issues](https://github.com/maxBogovick/unistructgen/issues)
- **Examples:** [examples/openapi/](./examples/openapi/)

---

## Next Steps

1. ✅ [Quick Start](#quick-start) - Get up and running
2. 📚 [Examples](#real-world-examples) - Learn from real code
3. 🔧 [Best Practices](#best-practices) - Write production code
4. 🚀 [Advanced Features](#advanced-features) - Master the tool

**Happy coding! 🎉**
