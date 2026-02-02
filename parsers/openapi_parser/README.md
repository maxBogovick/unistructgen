# UniStructGen OpenAPI Parser

OpenAPI 3.0/3.1 parser for UniStructGen that generates type-safe Rust types from OpenAPI specifications.

> Status: parsing and type generation are production-grade; client generation is a scaffold and may require manual adjustments for edge cases.

## Features

✨ **Comprehensive OpenAPI Support**
- OpenAPI 3.0 and 3.1 specifications
- Both YAML and JSON formats
- Schema composition (allOf, oneOf, anyOf)
- Reference ($ref) resolution
- Nested object structures

🎯 **Smart Type Generation**
- Automatic type inference (UUID, DateTime, etc.)
- Enum generation from string enums
- Validation constraint extraction
- Array and map support
- Optional field detection

🔐 **API Client Generation (scaffold)**
- Generate client traits and types
- Basic parameter extraction
- Method signatures (best-effort)

✅ **Validation Support (best-effort)**
- Extracts validation metadata for downstream codegen
- Min/max length constraints
- Range validation
- Format validation (email, URL, etc.)
- Pattern matching

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
unistructgen-openapi-parser = "0.1"
unistructgen-core = "0.1"
unistructgen-codegen = "0.1"

# For special types
uuid = { version = "1.0", features = ["serde", "v4"] }
chrono = { version = "0.4", features = ["serde"] }
```

## Usage

### As a Library

```rust
use unistructgen_core::Parser;
use unistructgen_openapi_parser::{OpenApiParser, OpenApiParserOptions};
use unistructgen_codegen::RustRenderer;

// Configure the parser
let options = OpenApiParserOptions::builder()
    .generate_client(true)
    .generate_validation(true)
    .derive_serde(true)
    .build();

let mut parser = OpenApiParser::new(options);

// Parse OpenAPI spec
let spec = std::fs::read_to_string("openapi.yaml")?;
let ir_module = parser.parse(&spec)?;

// Generate Rust code
    let renderer = RustRenderer::new(Default::default());
let rust_code = renderer.generate(&ir_module)?;

println!("{}", rust_code);
```

### With Proc Macro

Add to your `Cargo.toml`:

```toml
[dependencies]
unistructgen-macro = "0.1"
```

Then in your code:

```rust
use unistructgen_macro::openapi_to_rust;

// From file
openapi_to_rust! {
    file = "openapi.yaml"
}

// From URL
openapi_to_rust! {
    url = "https://api.example.com/openapi.yaml",
    timeout = 30000
}

// From inline spec
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
      properties:
        id:
          type: integer
        name:
          type: string
    "#
}

// Now use the generated types
let user = User {
    id: 1,
    name: "Alice".to_string(),
};
```

## Configuration Options

### Parser Options

```rust
let options = OpenApiParserOptions::builder()
    // Generate API client traits (default: true)
    .generate_client(true)

    // Add validation derives (default: true)
    .generate_validation(true)

    // Add serde derives (default: true)
    .derive_serde(true)

    // Add Default derive (default: false)
    .derive_default(false)

    // Make all fields optional (default: false)
    .make_fields_optional(false)

    // Maximum depth for nested schemas (default: 10)
    .max_depth(10)

    // Filter by tags
    .include_tags(hashset!["users".to_string()])

    // Exclude tags
    .exclude_tags(hashset!["internal".to_string()])

    // Generate builder patterns (default: false)
    .generate_builders(false)

    // Generate documentation (default: true)
    .generate_docs(true)

    // Type name prefix/suffix
    .type_prefix("Api".to_string())
    .type_suffix("Dto".to_string())

    // Module name (default: "api")
    .module_name("api".to_string())

    .build();
```

## Examples

### Example 1: Pet Store API

```yaml
# petstore.yaml
openapi: 3.0.0
info:
  title: Pet Store
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
          type: string
          enum:
            - available
            - pending
            - sold
        age:
          type: integer
          minimum: 0
          maximum: 50
```

```rust
use unistructgen_macro::openapi_to_rust;

openapi_to_rust! {
    file = "petstore.yaml"
}

// Generated code includes:
// - Pet struct with validation
// - PetStatus enum
// - Serde derives for JSON serialization

fn main() {
    let pet = Pet {
        id: uuid::Uuid::new_v4(),
        name: "Fluffy".to_string(),
        status: Some(PetStatus::Available),
        age: Some(3),
    };

    // Validate
    pet.validate().expect("Invalid pet");

    // Serialize
    let json = serde_json::to_string(&pet).unwrap();
    println!("{}", json);
}
```

### Example 2: User Management API

```rust
openapi_to_rust! {
    spec = r#"
openapi: 3.0.0
info:
  title: User API
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
        profile:
          $ref: '#/components/schemas/Profile'

    Profile:
      type: object
      properties:
        firstName:
          type: string
        lastName:
          type: string
        age:
          type: integer
          minimum: 0
          maximum: 150
    "#
}

// Use the generated types
let user = User {
    id: 1,
    email: "user@example.com".to_string(),
    profile: Some(Profile {
        first_name: Some("John".to_string()),
        last_name: Some("Doe".to_string()),
        age: Some(30),
    }),
};
```

### Example 3: With Authentication

```rust
// Fetch from authenticated endpoint
openapi_to_rust! {
    url = "https://api.example.com/openapi.yaml",
    auth_bearer = env!("API_TOKEN"),
    timeout = 10000,
    generate_client = true,
    generate_validation = true
}
```

## Generated Code Examples

### Input Schema

```yaml
User:
  type: object
  required:
    - id
    - username
    - email
  properties:
    id:
      type: integer
      format: int64
    username:
      type: string
      minLength: 3
      maxLength: 20
    email:
      type: string
      format: email
    age:
      type: integer
      minimum: 0
      maximum: 150
    roles:
      type: array
      items:
        type: string
```

### Generated Rust Code

```rust
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
pub struct User {
    /// Unique identifier for the user
    pub id: i64,

    /// User's login name
    #[validate(length(min = 3, max = 20))]
    pub username: String,

    /// User's email address
    #[validate(email)]
    pub email: String,

    /// User's age
    #[validate(range(min = 0, max = 150))]
    pub age: Option<i64>,

    /// User's assigned roles
    pub roles: Option<Vec<String>>,
}
```

## Type Mappings

| OpenAPI Type | Format | Rust Type |
|--------------|--------|-----------|
| `string` | - | `String` |
| `string` | `date-time` | `chrono::DateTime<Utc>` |
| `string` | `uuid` | `uuid::Uuid` |
| `string` | `email` | `String` (with validation) |
| `string` | `uri`, `url` | `String` |
| `integer` | `int32` | `i32` |
| `integer` | `int64` | `i64` |
| `number` | `float` | `f32` |
| `number` | `double` | `f64` |
| `boolean` | - | `bool` |
| `array` | - | `Vec<T>` |
| `object` | - | Nested struct or `HashMap` |

## Advanced Features

### Schema Composition

```yaml
# allOf - merge schemas
User:
  allOf:
    - type: object
      properties:
        id:
          type: integer
    - type: object
      properties:
        name:
          type: string

# oneOf - enum variants
Response:
  oneOf:
    - $ref: '#/components/schemas/SuccessResponse'
    - $ref: '#/components/schemas/ErrorResponse'
```

### Reference Resolution

```yaml
Address:
  type: object
  properties:
    street:
      type: string

User:
  type: object
  properties:
    homeAddress:
      $ref: '#/components/schemas/Address'
    workAddress:
      $ref: '#/components/schemas/Address'
```

### Validation Constraints

```yaml
Username:
  type: string
  minLength: 3
  maxLength: 20
  pattern: '^[a-zA-Z0-9_]+$'

Age:
  type: integer
  minimum: 0
  maximum: 150

Tags:
  type: array
  minItems: 1
  maxItems: 10
```

## Error Handling

The parser provides detailed error messages:

```rust
use unistructgen_openapi_parser::OpenApiError;

match parser.parse(spec) {
    Ok(module) => { /* success */ }
    Err(OpenApiError::InvalidSpec(msg)) => {
        eprintln!("Invalid OpenAPI spec: {}", msg);
    }
    Err(OpenApiError::ReferenceResolution { reference, reason }) => {
        eprintln!("Failed to resolve {}: {}", reference, reason);
    }
    Err(OpenApiError::CircularReference(path)) => {
        eprintln!("Circular reference detected: {}", path);
    }
    Err(e) => {
        eprintln!("Parse error: {}", e);
    }
}
```

## Testing

Run the tests:

```bash
cd parsers/openapi_parser
cargo test
```

Run examples:

```bash
cargo run --example basic_usage
```

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](../../CONTRIBUTING.md) for guidelines.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](../../LICENSE-APACHE))
- MIT License ([LICENSE-MIT](../../LICENSE-MIT))

at your option.
