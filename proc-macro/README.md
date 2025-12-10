# UniStructGen Proc-Macro

Procedural macros for generating Rust structs from JSON at compile time.

## Features

- **Compile-time code generation**: Generate structs during compilation
- **Two macro styles**: Function-like and attribute macros
- **Type inference**: Automatic detection of DateTime, UUID, and other types
- **Nested structures**: Full support for nested objects and arrays
- **Serde integration**: Optional serde derives

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
unistructgen-macro = "0.1"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1.0", features = ["serde", "v4"] }
```

## Usage

### Function-like Macro: `generate_struct_from_json!`

Generate structs directly in your code:

```rust
use unistructgen_macro::generate_struct_from_json;

generate_struct_from_json! {
    name = "User",
    json = r#"{
        "id": 1,
        "name": "Alice",
        "email": "alice@example.com",
        "created_at": "2024-01-15T10:30:00Z"
    }"#
}

// Now you can use the User struct
fn main() {
    let json = r#"{"id": 42, "name": "Bob", "email": "bob@example.com", "created_at": "2024-12-09T10:00:00Z"}"#;
    let user: User = serde_json::from_str(json).unwrap();
    println!("{:?}", user);
}
```

### Attribute Macro: `#[json_struct]`

Attach to a const string:

```rust
use unistructgen_macro::json_struct;

#[json_struct(name = "Product")]
const PRODUCT_SCHEMA: &str = r#"{
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "Laptop",
    "price": 999.99,
    "in_stock": true
}"#;

fn main() {
    let product: Product = serde_json::from_str(PRODUCT_SCHEMA).unwrap();
    println!("{:?}", product);
}
```

## Parameters

Both macros support the following parameters:

### `name` (required for `generate_struct_from_json!`, optional for `#[json_struct]`)

The name of the root struct to generate.

```rust
generate_struct_from_json! {
    name = "MyStruct",
    json = r#"{"field": "value"}"#
}
```

### `json` (required for `generate_struct_from_json!`)

The JSON sample as a string literal.

### `serde` (optional, default: `true`)

Whether to add Serialize and Deserialize derives.

```rust
generate_struct_from_json! {
    name = "Plain",
    serde = false,
    json = r#"{"id": 1}"#
}
```

### `default` (optional, default: `false`)

Whether to add the Default derive.

```rust
generate_struct_from_json! {
    name = "WithDefault",
    default = true,
    json = r#"{"count": 0}"#
}
```

### `optional` (optional, default: `false`)

Make all fields optional (wrapped in `Option<T>`).

```rust
generate_struct_from_json! {
    name = "Config",
    optional = true,
    json = r#"{"host": "localhost", "port": 8080}"#
}

// Generated struct will have:
// pub struct Config {
//     pub host: Option<String>,
//     pub port: Option<i64>,
// }
```

## Examples

### Basic Usage

```rust
use unistructgen_macro::generate_struct_from_json;

generate_struct_from_json! {
    name = "Person",
    json = r#"{
        "name": "Alice",
        "age": 30,
        "active": true
    }"#
}

let person = Person {
    name: "Bob".to_string(),
    age: 25,
    active: true,
};
```

### Nested Structures

```rust
generate_struct_from_json! {
    name = "Company",
    json = r#"{
        "name": "Acme Corp",
        "address": {
            "street": "123 Main St",
            "city": "New York"
        }
    }"#
}

// Generates both Company and Address structs
```

### Arrays

```rust
generate_struct_from_json! {
    name = "Blog",
    json = r#"{
        "title": "My Post",
        "tags": ["rust", "programming"],
        "comments": [
            {"user": "Alice", "text": "Great!"}
        ]
    }"#
}

// Generates Blog and CommentsItem structs
```

### Special Type Detection

```rust
generate_struct_from_json! {
    name = "Event",
    json = r#"{
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "timestamp": "2024-12-09T14:30:00Z",
        "name": "Meeting"
    }"#
}

// Generates:
// pub struct Event {
//     pub id: uuid::Uuid,
//     pub timestamp: chrono::DateTime<chrono::Utc>,
//     pub name: String,
// }
```

## Type Inference

The macro automatically infers appropriate Rust types:

| JSON Type | Detected Pattern | Rust Type |
|-----------|------------------|-----------|
| Number (integer) | - | `i64` or `u64` |
| Number (float) | - | `f64` |
| String | ISO 8601 datetime | `chrono::DateTime<Utc>` |
| String | UUID format | `uuid::Uuid` |
| String | other | `String` |
| Boolean | - | `bool` |
| Array | - | `Vec<T>` |
| Object | - | Nested struct |
| Null | - | `Option<T>` |

## Field Name Sanitization

Field names are automatically converted to Rust conventions:

- `camelCase` → `snake_case` with `#[serde(rename = "camelCase")]`
- `PascalCase` → `snake_case` with rename attribute
- `kebab-case` → `snake_case` with rename attribute
- Rust keywords get `_` suffix

## Limitations

- JSON must be valid and parseable at compile time
- Empty arrays default to `Vec<serde_json::Value>`
- Heterogeneous arrays are not yet supported
- Only JSON input is supported (no Markdown or SQL in proc-macros)

## Comparison with CLI

| Feature | Proc-Macro | CLI |
|---------|-----------|-----|
| When to use | Compile-time | Pre-generation |
| Input sources | Inline JSON | Files (JSON, MD, SQL) |
| Watch mode | ❌ | ✅ |
| Multiple samples | ❌ | ✅ (planned) |
| IDE integration | ✅ | Limited |

## Error Messages

The macro provides helpful error messages:

```rust
generate_struct_from_json! {
    name = "Test",
    json = r#"invalid json"#
}
// Error: Failed to parse JSON: expected value at line 1 column 1
```

## See Also

- [CLI Documentation](../cli/README.md)
- [Examples](../examples/)
- [Core Library](../core/)

## License

MIT OR Apache-2.0
