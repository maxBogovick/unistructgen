# 🚀 Quick Start Guide

Get up and running with UniStructGen in **5 minutes**.

## 📋 Table of Contents

1. [Installation](#-installation)
2. [Your First Struct](#-your-first-struct)
3. [External API Integration](#-external-api-integration)
4. [Advanced Features](#-advanced-features)
5. [CLI Usage](#-cli-usage)
6. [Common Patterns](#-common-patterns)
7. [Troubleshooting](#-troubleshooting)
8. [Next Steps](#-next-steps)

---

## 📦 Installation

### Option 1: Proc-Macro (Recommended)

Add to your `Cargo.toml`:

```toml
[dependencies]
unistructgen-macro = "0.1"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Optional but recommended
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1.0", features = ["serde", "v4"] }
```

### Option 2: CLI Tool

```bash
cargo install unistructgen
```

### Option 3: Both

```bash
# Install CLI
cargo install unistructgen

# Add macro to Cargo.toml
# [dependencies]
# unistructgen-macro = "0.1"
```

---

## ⚡ Killer Example (60 Seconds)

```bash
cargo run -p killer-example
```

This single example shows:
- Compile-time type generation from JSON
- LLM tool schemas from Rust functions
- Safe, structured tool execution

See `examples/killer-example/README.md` for details.

---

## 🎯 Your First Struct

Let's start with the simplest example and build from there.

### Step 1: Basic JSON to Struct

Create a new file `src/main.rs`:

```rust
use unistructgen_macro::generate_struct_from_json;

// Define your struct from JSON
generate_struct_from_json! {
    name = "User",
    json = r#"{
        "id": 1,
        "name": "Alice",
        "email": "alice@example.com"
    }"#
}

fn main() {
    // Use the generated struct
    let user = User {
        id: 42,
        name: "Bob".to_string(),
        email: "bob@example.com".to_string(),
    };

    println!("User: {} <{}>", user.name, user.email);

    // Serialize to JSON
    let json = serde_json::to_string_pretty(&user).unwrap();
    println!("{}", json);
}
```

**Run it:**
```bash
cargo run
```

**Output:**
```
User: Bob <bob@example.com>
{
  "id": 42,
  "name": "Bob",
  "email": "bob@example.com"
}
```

### Step 2: Nested Objects

Let's add complexity with nested structures:

```rust
use unistructgen_macro::generate_struct_from_json;

generate_struct_from_json! {
    name = "Company",
    json = r#"{
        "name": "Acme Corp",
        "founded": 2020,
        "address": {
            "street": "123 Main St",
            "city": "New York",
            "zipCode": "10001"
        },
        "employees": [
            {
                "name": "John Doe",
                "position": "Engineer",
                "salary": 100000
            }
        ]
    }"#
}

fn main() {
    let company = Company {
        name: "TechCorp".to_string(),
        founded: 2024,
        address: Address {
            street: "456 Tech Ave".to_string(),
            city: "San Francisco".to_string(),
            zip_code: "94102".to_string(),
        },
        employees: vec![
            Employees {
                name: "Jane Smith".to_string(),
                position: "CTO".to_string(),
                salary: 200000,
            }
        ],
    };

    println!("Company: {}", company.name);
    println!("Location: {}, {}", company.address.city, company.address.zip_code);
    println!("Employees: {}", company.employees.len());
}
```

**What happened?**
- UniStructGen automatically created **three structs**: `Company`, `Address`, and `Employees`
- Field names converted: `zipCode` → `zip_code`
- Arrays inferred as `Vec<T>`
- All with serde derives by default

### Step 3: Smart Type Detection

UniStructGen detects special types automatically:

```rust
use unistructgen_macro::generate_struct_from_json;

generate_struct_from_json! {
    name = "Event",
    json = r#"{
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "title": "Conference",
        "startTime": "2024-06-15T09:00:00Z",
        "endTime": "2024-06-15T17:00:00Z",
        "attendees": 150
    }"#
}

fn main() {
    let event = Event {
        id: uuid::Uuid::new_v4(),  // ✨ Auto-detected as UUID
        title: "Rust Meetup".to_string(),
        start_time: chrono::Utc::now(),  // ✨ Auto-detected as DateTime
        end_time: chrono::Utc::now() + chrono::Duration::hours(2),
        attendees: 50,
    };

    println!("Event: {} (ID: {})", event.title, event.id);
    println!("Start: {}", event.start_time.format("%Y-%m-%d %H:%M"));
}
```

**Detected types:**
- ✅ UUID strings → `uuid::Uuid`
- ✅ ISO 8601 dates → `chrono::DateTime<Utc>`
- ✅ Integers → `i64` or `u64`
- ✅ Floats → `f64`
- ✅ Booleans → `bool`

---

## 🌐 External API Integration

The real power: generate structs from **live APIs** at compile time.

### Step 1: Basic API Call

```rust
use unistructgen_macro::struct_from_external_api;

// Fetch and generate at compile time
struct_from_external_api! {
    struct_name = "User",
    url_api = "https://jsonplaceholder.typicode.com/users/1"
}

fn main() {
    // Struct is ready to use!
    println!("User struct generated from live API");
}
```

**What happens:**
1. During compilation, UniStructGen makes an HTTP request
2. Receives the JSON response
3. Generates a fully-typed struct
4. No runtime overhead!

### Step 2: Array Responses

APIs often return arrays. We handle it automatically:

```rust
struct_from_external_api! {
    struct_name = "Todo",
    url_api = "https://jsonplaceholder.typicode.com/todos"
    // Returns: [{"userId": 1, "id": 1, "title": "...", "completed": false}, ...]
    // We automatically extract the first element!
}

fn main() {
    let todo = Todo {
        user_id: 1,
        id: 1,
        title: "Learn UniStructGen".to_string(),
        completed: true,
    };

    println!("Todo: {} ({})", todo.title,
             if todo.completed { "✓" } else { "○" });
}
```

### Step 3: Multiple APIs

Generate multiple structs from different endpoints:

```rust
use unistructgen_macro::struct_from_external_api;

// GitHub API
struct_from_external_api! {
    struct_name = "Repository",
    url_api = "https://api.github.com/repos/rust-lang/rust"
}

// JSONPlaceholder API
struct_from_external_api! {
    struct_name = "Post",
    url_api = "https://jsonplaceholder.typicode.com/posts/1"
}

// Another endpoint
struct_from_external_api! {
    struct_name = "Comment",
    url_api = "https://jsonplaceholder.typicode.com/comments/1"
}

fn main() {
    println!("Generated 3 structs from 3 different APIs!");
}
```

---

## ⚙️ Advanced Features

### Optional Fields

Make all fields `Option<T>`:

```rust
use unistructgen_macro::generate_struct_from_json;

generate_struct_from_json! {
    name = "User",
    json = r#"{"id": 1, "name": "Alice"}"#,
    optional = true  // All fields become Option<T>
}

fn main() {
    let user = User {
        id: Some(42),
        name: Some("Bob".to_string()),
    };

    // Or partial data
    let partial = User {
        id: Some(1),
        name: None,
    };

    println!("Full: {:?}", user);
    println!("Partial: {:?}", partial);
}
```

**Generated:**
```rust
pub struct User {
    pub id: Option<i64>,
    pub name: Option<String>,
}
```

### Default Derive

Add `Default` trait:

```rust
generate_struct_from_json! {
    name = "Config",
    json = r#"{"debug": false, "port": 8080}"#,
    default = true  // Adds #[derive(Default)]
}

fn main() {
    // Use default values
    let config = Config::default();
    println!("Default port: {}", config.port);
}
```

### Disable Serde

If you don't need serialization:

```rust
generate_struct_from_json! {
    name = "Data",
    json = r#"{"value": 42}"#,
    serde = false  // No serde derives
}
```

### Limit Nesting Depth

For deeply nested APIs:

```rust
struct_from_external_api! {
    struct_name = "DeepData",
    url_api = "https://api.example.com/deep",
    max_depth = 3  // Stop at 3 levels deep
}
```

### Request Timeout

Control API request timeout:

```rust
struct_from_external_api! {
    struct_name = "SlowApi",
    url_api = "https://slow-api.example.com/data",
    timeout = 10000  // 10 seconds (default: 30000ms)
}
```

### Combine Options

```rust
struct_from_external_api! {
    struct_name = "FullFeatured",
    url_api = "https://api.example.com/data",

    // All options
    timeout = 10000,
    max_depth = 5,
    optional = true,
    default = true,
    serde = true,
}
```

---

## 💻 CLI Usage

For build scripts, CI/CD, and pre-generation.

### Basic Usage

```bash
# Generate from file
unistructgen generate \
    --input examples/user.json \
    --name User \
    --output src/models/user.rs

# Generate to stdout
unistructgen generate -i schema.json -n Schema

# With options
unistructgen generate \
    -i data.json \
    -n Data \
    --serde true \
    --optional \
    --default
```

### Real-World Example

```bash
# Create a schema file
cat > product.json << EOF
{
  "id": "123e4567-e89b-12d3-a456-426614174000",
  "name": "Laptop",
  "price": 999.99,
  "inStock": true,
  "createdAt": "2024-01-15T10:30:00Z"
}
EOF

# Generate Rust code
unistructgen generate \
    --input product.json \
    --name Product \
    --output src/product.rs \
    --serde true

# Use in your code
cat > src/main.rs << 'EOF'
mod product;
use product::Product;

fn main() {
    let p = Product {
        id: uuid::Uuid::new_v4(),
        name: "Mouse".to_string(),
        price: 29.99,
        in_stock: true,
        created_at: chrono::Utc::now(),
    };
    println!("{:?}", p);
}
EOF

cargo run
```

### Pipeline Integration

```bash
# Fetch from API and generate
curl -o schema.json https://api.example.com/schema.json
unistructgen generate --input schema.json --name ApiSchema --output src/api_schema.rs

# In CI/CD (GitHub Actions example)
- name: Generate structs
  run: |
    cargo install unistructgen
    unistructgen generate -i schema.json -n Schema -o src/schema.rs

- name: Commit generated code
  run: |
    git add src/schema.rs
    git commit -m "Update generated schemas"
```

---

## 📚 Common Patterns

### Compile-Time Fetch Controls

For macros that fetch remote content at compile time (`struct_from_external_api!`, `openapi_to_rust!` with `url`, `env_file` over HTTP), you can control networking behavior:

- `UNISTRUCTGEN_FETCH_OFFLINE=1` — disable network access (cache only)
- `UNISTRUCTGEN_FETCH_CACHE=0` — disable caching
- `UNISTRUCTGEN_FETCH_CACHE_DIR=/path` — custom cache directory
- `UNISTRUCTGEN_FETCH_TIMEOUT_MS=60000` — override timeout (ms)

### Pattern 1: Configuration Files

```rust
use unistructgen_macro::json_struct;
use std::fs;

#[json_struct(name = "AppConfig")]
const CONFIG_SCHEMA: &str = r#"{
    "server": {
        "host": "localhost",
        "port": 8080,
        "tls": false
    },
    "database": {
        "url": "postgres://localhost/mydb",
        "poolSize": 10
    },
    "logging": {
        "level": "info",
        "format": "json"
    }
}"#;

fn main() {
    // Read config file
    let config_str = fs::read_to_string("config.json").unwrap();
    let config: AppConfig = serde_json::from_str(&config_str).unwrap();

    println!("Server: {}:{}", config.server.host, config.server.port);
    println!("Database pool: {}", config.database.pool_size);
    println!("Log level: {}", config.logging.level);
}
```

### Pattern 2: API Response Handler

```rust
use unistructgen_macro::struct_from_external_api;

struct_from_external_api! {
    struct_name = "GithubRepo",
    url_api = "https://api.github.com/repos/rust-lang/rust"
}

async fn fetch_repo(owner: &str, repo: &str) -> Result<GithubRepo, Box<dyn std::error::Error>> {
    let url = format!("https://api.github.com/repos/{}/{}", owner, repo);
    let response = reqwest::get(&url).await?;
    let repo: GithubRepo = response.json().await?;
    Ok(repo)
}

#[tokio::main]
async fn main() {
    let repo = fetch_repo("rust-lang", "rust").await.unwrap();
    println!("⭐ Stars: {}", repo.stargazers_count);
    println!("🍴 Forks: {}", repo.forks_count);
}
```

### Pattern 3: Multi-File Organization

```rust
// src/models/mod.rs
pub mod user;
pub mod post;
pub mod comment;

// src/models/user.rs
use unistructgen_macro::struct_from_external_api;

struct_from_external_api! {
    struct_name = "User",
    url_api = "https://jsonplaceholder.typicode.com/users/1"
}

// src/models/post.rs
use unistructgen_macro::struct_from_external_api;

struct_from_external_api! {
    struct_name = "Post",
    url_api = "https://jsonplaceholder.typicode.com/posts/1"
}

// src/main.rs
mod models;
use models::{user::User, post::Post};

fn main() {
    // Use both types
}
```

### Pattern 4: Test Fixtures

```rust
#[cfg(test)]
mod tests {
    use unistructgen_macro::generate_struct_from_json;

    generate_struct_from_json! {
        name = "TestUser",
        json = r#"{
            "id": 1,
            "name": "Test User",
            "email": "test@example.com"
        }"#
    }

    #[test]
    fn test_user_creation() {
        let user = TestUser {
            id: 1,
            name: "Alice".to_string(),
            email: "alice@test.com".to_string(),
        };

        assert_eq!(user.id, 1);
        assert_eq!(user.name, "Alice");
    }
}
```

---

## 🔧 Troubleshooting

### Issue: "Failed to fetch from API"

**Problem:** Network error during compilation

**Solution:**
```rust
// Check your internet connection
// Increase timeout
struct_from_external_api! {
    struct_name = "Data",
    url_api = "https://api.example.com/data",
    timeout = 60000  // 60 seconds
}
```

### Issue: "Invalid JSON structure: expected object, found array"

**Problem:** API returns an array but you're treating it as object

**Solution:** This should work automatically now, but if not:
```rust
// Arrays are auto-detected and first element is extracted
struct_from_external_api! {
    struct_name = "Item",
    url_api = "https://api.example.com/items"  // Returns [...]
}
// This works! We extract the first element
```

### Issue: Compilation is slow

**Problem:** Multiple API calls during compilation

**Solution:**
```rust
// Option 1: Use CLI to pre-generate
// unistructgen generate -i schema.json -o models.rs

// Option 2: Cache responses
// Option 3: Use inline JSON in dev, API in release
#[cfg(debug_assertions)]
generate_struct_from_json! {
    name = "User",
    json = r#"{"id": 1, "name": "Dev"}"#
}

#[cfg(not(debug_assertions))]
struct_from_external_api! {
    struct_name = "User",
    url_api = "https://api.example.com/user"
}
```

### Issue: Field names conflict with Rust keywords

**Problem:** JSON has field named `type`, `impl`, etc.

**Solution:** Automatic!
```rust
// JSON: {"type": "user"}
// Generated:
pub struct Data {
    pub type_: String,  // Automatically appends _
}
```

### Issue: Need custom derives

**Problem:** Want `PartialOrd`, `Eq`, etc.

**Solution:**
```rust
use unistructgen_macro::generate_struct_from_json;

generate_struct_from_json! {
    name = "User",
    json = r#"{"id": 1, "name": "Alice"}"#
}

// Add custom derives after generation
#[derive(PartialOrd, Ord, Eq)]
struct UserWrapper(User);
```

---

## 🎯 Next Steps

Congratulations! You now know the essentials of UniStructGen.

### Learn More

1. **[Complete Examples](EXAMPLES.md)** - See real-world usage patterns
2. **[External API Guide](docs/EXTERNAL_API_GUIDE.md)** - Advanced API integration
3. **[API Documentation](https://docs.rs/unistructgen)** - Full reference
4. **[Architecture](docs/ARCHITECTURE.md)** - How it works internally

### Build Something

Try these projects:

1. **Weather App**
   ```rust
   struct_from_external_api! {
       struct_name = "Weather",
       url_api = "https://api.openweathermap.org/data/2.5/weather?q=London&appid=YOUR_KEY"
   }
   ```

2. **GitHub Stats**
   ```rust
   struct_from_external_api! {
       struct_name = "GitHubUser",
       url_api = "https://api.github.com/users/yourusername"
   }
   ```

3. **REST API Client**
   - Use UniStructGen to generate all your API models
   - Build a fully-typed client
   - Never worry about API changes

### Join the Community

- ⭐ [Star us on GitHub](https://github.com/maxBogovick/unistructgen)
- 💬 [Join Discussions](https://github.com/maxBogovick/unistructgen/discussions)
- 🐛 [Report Issues](https://github.com/maxBogovick/unistructgen/issues)
- 📝 [Contribute](CONTRIBUTING.md)

---

## 🤔 FAQ

**Q: Can I use this in production?**
A: Yes! All code generation happens at compile time. Zero runtime cost.

**Q: What if the API changes?**
A: Just recompile. The struct will be regenerated from the latest API response.

**Q: Does it work with private APIs?**
A: Yes, but the API must be accessible during compilation. For private APIs with auth, use the CLI with environment variables.

**Q: Can I modify generated code?**
A: Yes! Either edit the generated output (if using CLI) or create wrapper types.

**Q: What about GraphQL?**
A: Coming soon! For now, you can use the JSON response from a GraphQL query.

**Q: Does it work offline?**
A: `generate_struct_from_json!` works offline. `struct_from_external_api!` needs network during compilation.

---

<div align="center">

**Ready to build?** [Back to README](README.md) • [See Examples](EXAMPLES.md) • [Read Docs](https://docs.rs/unistructgen)

Made with 🦀 and ❤️ for Rust developers.

</div>
