# ✨ Features

Comprehensive guide to UniStructGen's capabilities and features.

## 📋 Table of Contents

1. [Core Features](#-core-features)
2. [Smart Type Inference](#-smart-type-inference)
3. [Code Generation](#-code-generation)
4. [API Integration](#-api-integration)
5. [Customization Options](#-customization-options)
6. [Advanced Capabilities](#-advanced-capabilities)
7. [Performance](#-performance)
8. [Comparison Matrix](#-comparison-matrix)

---

## 🎯 Core Features

### Compile-Time Generation

Generate Rust structs during compilation with zero runtime overhead.

```rust
use unistructgen_macro::generate_struct_from_json;

// This runs at compile time!
generate_struct_from_json! {
    name = "User",
    json = r#"{"id": 1, "name": "Alice"}"#
}

// Generated struct is available immediately
let user = User {
    id: 42,
    name: "Bob".to_string(),
};
```

**Benefits:**
- ✅ No runtime parsing overhead
- ✅ Compile-time type checking
- ✅ IDE autocomplete support
- ✅ Zero-cost abstractions

---

### Multiple Input Methods

Three ways to provide input:

#### 1. Inline JSON

```rust
generate_struct_from_json! {
    name = "Product",
    json = r#"{"id": 1, "name": "Widget"}"#
}
```

**Use when:**
- Schema is known and static
- Quick prototyping
- Testing

#### 2. External API

```rust
struct_from_external_api! {
    struct_name = "User",
    url_api = "https://api.example.com/users/1"
}
```

**Use when:**
- Syncing with live APIs
- Dynamic schemas
- External data sources

#### 3. Attribute Macro

```rust
#[json_struct(name = "Config")]
const SCHEMA: &str = r#"{"debug": true}"#;
```

**Use when:**
- Organizing schemas
- Keeping schemas with constants
- Cleaner code separation

---

## 🧠 Smart Type Inference

UniStructGen doesn't just map JSON primitives to Rust types. It **understands** your data.

### Automatic Type Detection

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "email": "user@example.com",
  "website": "https://example.com",
  "created": "2024-01-15T10:30:00Z",
  "port": 8080,
  "price": 99.99,
  "active": true,
  "tags": ["rust", "codegen"]
}
```

**Generates:**

```rust
pub struct Data {
    pub id: uuid::Uuid,                          // ✨ UUID detected
    pub email: String,                           // Could be validated
    pub website: url::Url,                       // ✨ URL detected
    pub created: chrono::DateTime<chrono::Utc>,  // ✨ DateTime detected
    pub port: i64,                               // Integer
    pub price: f64,                              // Float
    pub active: bool,                            // Boolean
    pub tags: Vec<String>,                       // Array → Vec
}
```

### Type Detection Rules

| JSON Value | Rust Type | Detection |
|------------|-----------|-----------|
| `"550e8400-..."` | `uuid::Uuid` | UUID pattern |
| `"2024-01-15T10:30:00Z"` | `chrono::DateTime<Utc>` | ISO 8601 |
| `"user@example.com"` | `String` | Email pattern (future) |
| `"https://example.com"` | `url::Url` | URL pattern |
| `42` | `i64` | Integer |
| `3.14` | `f64` | Float |
| `true` | `bool` | Boolean |
| `[...]` | `Vec<T>` | Array |
| `{...}` | Nested struct | Object |
| `null` | `Option<T>` | Null value |

---

## 🎨 Code Generation

### Clean, Idiomatic Output

Generated code follows Rust best practices:

```rust
// Input JSON
{
    "userId": 123,
    "firstName": "John",
    "last-name": "Doe",
    "123field": "value"
}
```

```rust
// Generated Rust (formatted and clean)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct User {
    #[serde(rename = "userId")]
    pub user_id: i64,

    #[serde(rename = "firstName")]
    pub first_name: String,

    #[serde(rename = "last-name")]
    pub last_name: String,

    pub _123field: String,
}
```

**Features:**
- ✅ snake_case field names
- ✅ Proper serde rename attributes
- ✅ Keyword handling (type_ for "type")
- ✅ Invalid character sanitization
- ✅ Proper formatting and indentation

---

### Nested Object Generation

Automatically creates separate structs for nested objects:

```json
{
  "user": {
    "profile": {
      "address": {
        "city": "New York",
        "country": "USA"
      }
    }
  }
}
```

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
    pub country: String,
}
```

**Benefits:**
- Clean separation of concerns
- Type safety at all levels
- Reusable nested types
- Better code organization

---

### Array Handling

Intelligent array type inference:

```json
{
  "tags": ["rust", "codegen"],
  "scores": [1, 2, 3],
  "items": [
    {"id": 1, "name": "First"},
    {"id": 2, "name": "Second"}
  ]
}
```

```rust
pub struct Data {
    pub tags: Vec<String>,
    pub scores: Vec<i64>,
    pub items: Vec<Items>,
}

pub struct Items {
    pub id: i64,
    pub name: String,
}
```

**Features:**
- ✅ Infers element type from first element
- ✅ Creates nested struct for object arrays
- ✅ Handles empty arrays gracefully
- ✅ Supports multi-dimensional arrays

---

## 🌐 API Integration

### Compile-Time API Fetching

Fetch schemas from live APIs during compilation:

```rust
struct_from_external_api! {
    struct_name = "GithubRepo",
    url_api = "https://api.github.com/repos/rust-lang/rust"
}

// Struct generated from real API response
// No runtime cost!
```

**How it works:**
1. HTTP request made during macro expansion
2. JSON response parsed
3. Struct generated
4. All at compile time!

---

### Array Auto-Detection

APIs often return arrays. We handle it automatically:

```rust
// API returns: [{"id": 1, "name": "Todo 1"}, ...]
struct_from_external_api! {
    struct_name = "Todo",
    url_api = "https://jsonplaceholder.typicode.com/todos"
}

// Automatically extracts structure from first array element!
```

**No more:**
```rust
// ❌ Manual extraction needed
let response: Vec<Todo> = fetch_api()?;
let first = response.first().unwrap();
```

**Just works:**
```rust
// ✅ Automatic
struct_from_external_api! { /* ... */ }
```

---

### Error Handling

Clear error messages for API issues:

```rust
// Network error
struct_from_external_api! {
    struct_name = "Data",
    url_api = "https://unreachable-api.com/data"
}
// Error: Failed to fetch from API: Connection timeout
```

```rust
// Empty array
struct_from_external_api! {
    struct_name = "Data",
    url_api = "https://api.example.com/empty"
}
// Error: API returned an empty array. Cannot infer struct from empty array.
```

---

## ⚙️ Customization Options

### Optional Fields

Make all fields `Option<T>`:

```rust
generate_struct_from_json! {
    name = "User",
    json = r#"{"id": 1, "name": "Alice"}"#,
    optional = true
}

// Generated:
pub struct User {
    pub id: Option<i64>,
    pub name: Option<String>,
}
```

**Use cases:**
- Partial data from APIs
- Optional configuration fields
- Nullable database columns

---

### Default Derive

Add `Default` trait implementation:

```rust
generate_struct_from_json! {
    name = "Config",
    json = r#"{"debug": false, "port": 8080}"#,
    default = true
}

// Now you can:
let config = Config::default();
```

---

### Serde Configuration

Control serde derives:

```rust
// With serde (default)
generate_struct_from_json! {
    name = "User",
    json = r#"{"id": 1}"#,
    serde = true
}

// Without serde
generate_struct_from_json! {
    name = "PlainStruct",
    json = r#"{"id": 1}"#,
    serde = false
}
```

---

### Depth Limiting

Limit nested object depth:

```rust
struct_from_external_api! {
    struct_name = "DeepData",
    url_api = "https://api.example.com/deep-nesting",
    max_depth = 3  // Stop at 3 levels
}
```

**Prevents:**
- Excessive struct generation
- Compilation slowdown
- Memory issues with deeply nested APIs

---

### Request Timeout

Configure API request timeout:

```rust
struct_from_external_api! {
    struct_name = "SlowApi",
    url_api = "https://slow-api.example.com/data",
    timeout = 10000  // 10 seconds (default: 30000ms)
}
```

---

### Combined Options

Use multiple options together:

```rust
struct_from_external_api! {
    struct_name = "CompleteExample",
    url_api = "https://api.example.com/data",

    // All customization options
    timeout = 10000,
    max_depth = 5,
    optional = true,
    default = true,
    serde = true,
}
```

---

## 🚀 Advanced Capabilities

### Multi-API Support

Generate from multiple APIs in one project:

```rust
// GitHub
struct_from_external_api! {
    struct_name = "GithubRepo",
    url_api = "https://api.github.com/repos/rust-lang/rust"
}

// JSONPlaceholder
struct_from_external_api! {
    struct_name = "Post",
    url_api = "https://jsonplaceholder.typicode.com/posts/1"
}

// Your own API
struct_from_external_api! {
    struct_name = "CustomData",
    url_api = "https://your-api.com/data"
}
```

---

### Conditional Compilation

Different schemas for dev/prod:

```rust
#[cfg(debug_assertions)]
generate_struct_from_json! {
    name = "Config",
    json = r#"{"env": "development"}"#
}

#[cfg(not(debug_assertions))]
struct_from_external_api! {
    struct_name = "Config",
    url_api = "https://prod-api.com/config"
}
```

---

### Module Organization

Organize generated types:

```rust
// src/models/user.rs
use unistructgen_macro::struct_from_external_api;

struct_from_external_api! {
    struct_name = "User",
    url_api = "https://api.example.com/users/1"
}

// src/models/mod.rs
pub mod user;
pub mod post;

// src/main.rs
use models::user::User;
```

---

## ⚡ Performance

### Zero Runtime Overhead

All generation happens at compile time:

```rust
// Compile time: Generate struct from API
struct_from_external_api! { /* ... */ }

// Runtime: Just use the struct
let user = User { /* ... */ };  // ← Zero overhead!
```

**No runtime costs for:**
- ❌ JSON parsing
- ❌ Type reflection
- ❌ Schema validation
- ❌ Dynamic dispatch

**Just:**
- ✅ Direct struct access
- ✅ Static dispatch
- ✅ Compile-time guarantees

---

### Compilation Performance

**Fast compilation:**
- Efficient macro expansion
- Minimal dependencies
- Parallel compilation support

**Tips for faster builds:**
```rust
// Cache API responses in CI/CD
// Use inline JSON in development
// Switch to API fetching in release builds
```

---

## 📊 Comparison Matrix

### vs. Manual Struct Writing

| Aspect | UniStructGen | Manual |
|--------|--------------|--------|
| Time to create | Seconds | Minutes to hours |
| Accuracy | 100% | Prone to errors |
| Maintenance | Auto-sync | Manual updates |
| Type safety | Guaranteed | Manual verification |
| Documentation | Auto-generated | Manual |

### vs. serde_json::Value

| Aspect | UniStructGen | serde_json::Value |
|--------|--------------|-------------------|
| Type safety | Full | None |
| Runtime overhead | Zero | Parsing cost |
| IDE support | Full autocomplete | Limited |
| Refactoring | Safe | Risky |
| Performance | Native | Slower |

### vs. quicktype

| Feature | UniStructGen | quicktype |
|---------|--------------|-----------|
| Compile-time | ✅ | ❌ |
| Proc macros | ✅ | ❌ |
| API fetching | ✅ | ✅ |
| Rust-specific | ✅ | ❌ (multi-lang) |
| Live sync | ✅ | Manual |
| Zero runtime cost | ✅ | ✅ |

---

## 🎯 Feature Roadmap

### ✅ Current (v0.1)

- ✅ JSON parsing
- ✅ Smart type inference
- ✅ Proc macros
- ✅ External API support
- ✅ Array auto-detection
- ✅ CLI tool
- ✅ Nested objects
- ✅ Field sanitization

### 🔜 Coming Soon (v0.2)

- [ ] OpenAPI/Swagger support
- [ ] GraphQL schema parsing
- [ ] Multi-sample merging (detect optional fields)
- [ ] Watch mode
- [ ] Builder pattern generation
- [ ] Validation derives

### 🔮 Future (v1.0+)

- [ ] Markdown table parsing
- [ ] SQL DDL parsing
- [ ] TypeScript definitions export
- [ ] Custom type mappings
- [ ] VSCode extension
- [ ] Web playground
- [ ] Plugin system

---

## 💎 Best Practices

### When to Use Each Feature

**Inline JSON:**
- ✅ Static schemas
- ✅ Testing
- ✅ Examples
- ❌ Dynamic APIs

**External API:**
- ✅ Live APIs
- ✅ Auto-sync needed
- ✅ Production use
- ❌ Offline builds

**CLI:**
- ✅ Build scripts
- ✅ CI/CD
- ✅ Committed schemas
- ✅ Code review

### Optimization Tips

1. **Cache API responses** in CI/CD
2. **Use inline JSON** in development
3. **Limit max_depth** for deep APIs
4. **Set appropriate timeouts**
5. **Organize into modules**

---

## 🎓 Learning Path

1. **Start Simple** - Use inline JSON
2. **Add Complexity** - Try nested objects
3. **Go Live** - Connect to real APIs
4. **Customize** - Use optional fields, derives
5. **Scale** - Organize into modules
6. **Optimize** - Tune performance settings

---

<div align="center">

**[⬅️ Back to README](README.md)** • **[📖 Quick Start](QUICKSTART.md)** • **[📚 Examples](EXAMPLES.md)**

Discover the full power of UniStructGen

</div>
