# 💼 Practical Applications & Implementation Guide

Real-world applications you can build **TODAY** with UniStructGen, and what's needed to unlock even more.

## 📋 Table of Contents

1. [What You Can Build Today](#-what-you-can-build-today)
2. [What's Blocked & How to Fix](#-whats-blocked--how-to-fix)
3. [Step-by-Step Implementation Plans](#-step-by-step-implementation-plans)
4. [Monetization Opportunities](#-monetization-opportunities)
5. [Community Projects](#-community-projects)

---

## ✅ What You Can Build Today

### 1. API Client Generator (Ready Now)

**Problem:** Manually writing API client types

**Solution:**

```rust
// File: api_client_generator/src/main.rs
use unistructgen_macro::struct_from_external_api;

// GitHub API
struct_from_external_api! {
    struct_name = "Repository",
    url_api = "https://api.github.com/repos/rust-lang/rust"
}

struct_from_external_api! {
    struct_name = "Issue",
    url_api = "https://api.github.com/repos/rust-lang/rust/issues/1"
}

struct_from_external_api! {
    struct_name = "User",
    url_api = "https://api.github.com/users/torvalds"
}

// Now build API client
pub struct GithubClient {
    client: reqwest::Client,
    token: String,
}

impl GithubClient {
    pub async fn get_repo(&self, owner: &str, name: &str) -> Result<Repository> {
        let url = format!("https://api.github.com/repos/{}/{}", owner, name);
        let repo = self.client.get(&url)
            .bearer_auth(&self.token)
            .send()
            .await?
            .json::<Repository>()
            .await?;
        Ok(repo)
    }

    pub async fn get_issues(&self, owner: &str, repo: &str) -> Result<Vec<Issue>> {
        // Similar implementation
    }
}
```

**Value:** Save 10+ hours per API integration

**Market:** Every company using APIs (99% of companies)

---

### 2. Configuration Management System (Ready Now)

**Problem:** Runtime config parsing errors

**Solution:**

```rust
// File: config_generator/src/main.rs
use unistructgen_macro::generate_struct_from_json;

// Database config
generate_struct_from_json! {
    name = "DatabaseConfig",
    json = r#"{
        "host": "localhost",
        "port": 5432,
        "username": "admin",
        "password": "secret",
        "database": "myapp",
        "pool_size": 10,
        "ssl": true
    }"#
}

// Redis config
generate_struct_from_json! {
    name = "RedisConfig",
    json = r#"{
        "url": "redis://localhost:6379",
        "max_connections": 20,
        "timeout": 5000
    }"#
}

// Application config
generate_struct_from_json! {
    name = "AppConfig",
    json = r#"{
        "database": {
            "host": "localhost",
            "port": 5432
        },
        "redis": {
            "url": "redis://localhost"
        },
        "server": {
            "host": "0.0.0.0",
            "port": 8080
        }
    }"#
}

// Usage
fn main() {
    let config_str = std::fs::read_to_string("config.json").unwrap();
    let config: AppConfig = serde_json::from_str(&config_str).unwrap();

    println!("Starting server on {}:{}", config.server.host, config.server.port);
}
```

**Value:** Type-safe config, caught at compile time

**Market:** Every Rust application

---

### 3. Mock Data Generator for Testing (Ready Now)

**Problem:** Creating realistic test data

**Solution:**

```rust
// File: mock_generator/src/lib.rs
use unistructgen_macro::generate_struct_from_json;
use fake::{Fake, Faker};

generate_struct_from_json! {
    name = "User",
    json = r#"{
        "id": 1,
        "name": "John Doe",
        "email": "john@example.com",
        "age": 30
    }"#
}

// Extension trait for mocking
pub trait MockData: Sized {
    fn mock() -> Self;
    fn mock_many(n: usize) -> Vec<Self>;
}

impl MockData for User {
    fn mock() -> Self {
        User {
            id: Faker.fake(),
            name: Faker.fake(),
            email: Faker.fake(),
            age: (18..80).fake(),
        }
    }

    fn mock_many(n: usize) -> Vec<Self> {
        (0..n).map(|_| Self::mock()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_with_mock_data() {
        let users = User::mock_many(100);
        assert_eq!(users.len(), 100);

        // Use in tests
        for user in users {
            assert!(user.age >= 18 && user.age < 80);
        }
    }
}
```

**Value:** Faster testing with realistic data

**Market:** Every developer writing tests

---

### 4. Schema Documentation Generator (Ready Now)

**Problem:** Documenting API schemas

**Solution:**

```rust
// File: schema_docs/src/main.rs
use unistructgen_core::*;
use unistructgen_json_parser::{JsonParser, ParserOptions};

fn generate_markdown_docs(module: &IRModule) -> String {
    let mut output = String::new();

    output.push_str(&format!("# {} Documentation\n\n", module.name));

    for ty in &module.types {
        if let IRType::Struct(s) = ty {
            output.push_str(&format!("## {}\n\n", s.name));

            // Table header
            output.push_str("| Field | Type | Description |\n");
            output.push_str("|-------|------|-------------|\n");

            for field in &s.fields {
                let field_type = format_type(&field.ty);
                let desc = field.documentation.as_deref().unwrap_or("-");
                output.push_str(&format!("| `{}` | `{}` | {} |\n",
                    field.name, field_type, desc));
            }

            output.push_str("\n");
        }
    }

    output
}

fn format_type(ty: &IRTypeRef) -> String {
    match ty {
        IRTypeRef::Primitive(p) => format!("{:?}", p),
        IRTypeRef::Named(n) => n.clone(),
        IRTypeRef::Vec(inner) => format!("Vec<{}>", format_type(inner)),
        IRTypeRef::Option(inner) => format!("Option<{}>", format_type(inner)),
        _ => "Unknown".to_string(),
    }
}

fn main() {
    let json = std::fs::read_to_string("schema.json").unwrap();

    let mut parser = JsonParser::new(ParserOptions {
        struct_name: "User".to_string(),
        derive_serde: true,
        derive_default: false,
        make_fields_optional: false,
    });

    let module = parser.parse(&json).unwrap();
    let docs = generate_markdown_docs(&module);

    std::fs::write("SCHEMA.md", docs).unwrap();
    println!("Documentation generated!");
}
```

**Output:**

```markdown
# User Documentation

## User

| Field | Type | Description |
|-------|------|-------------|
| `id` | `I64` | - |
| `name` | `String` | - |
| `email` | `String` | - |
```

**Value:** Auto-generated, always in sync

**Market:** API documentation teams

---

### 5. Multi-API Aggregator (Ready Now)

**Problem:** Working with multiple APIs

**Solution:**

```rust
// File: api_aggregator/src/lib.rs
use unistructgen_macro::struct_from_external_api;

// Multiple APIs
struct_from_external_api! {
    struct_name = "GithubUser",
    url_api = "https://api.github.com/users/octocat"
}

struct_from_external_api! {
    struct_name = "JsonPlaceholderUser",
    url_api = "https://jsonplaceholder.typicode.com/users/1"
}

// Aggregate data
pub struct AggregatedUser {
    pub github: Option<GithubUser>,
    pub placeholder: Option<JsonPlaceholderUser>,
}

impl AggregatedUser {
    pub async fn fetch(username: &str) -> Result<Self> {
        let github = fetch_github_user(username).await.ok();
        let placeholder = fetch_placeholder_user(1).await.ok();

        Ok(Self { github, placeholder })
    }
}
```

**Value:** Unified interface to multiple data sources

**Market:** Data aggregation services

---

## 🚧 What's Blocked & How to Fix

### 1. Authenticated APIs ❌ (Critical)

**What's Blocked:**

```rust
// Can't access authenticated endpoints
struct_from_external_api! {
    struct_name = "PrivateData",
    url_api = "https://api.example.com/private"  // ❌ 401 Unauthorized
}
```

**What's Needed:**

```rust
struct_from_external_api! {
    struct_name = "PrivateData",
    url_api = "https://api.example.com/private",
    auth = Bearer(env!("API_TOKEN"))  // ✅ Fix
}
```

**Implementation Plan (1 week):**

```rust
// 1. Extend ExternalApiInput in proc-macro/src/lib.rs
struct ExternalApiInput {
    // ... existing fields ...
    auth_type: Option<AuthType>,
    auth_value: Option<String>,
}

enum AuthType {
    Bearer,
    ApiKey,
    Basic,
}

// 2. Update fetch_json_from_api
fn fetch_json_from_api(input: &ExternalApiInput) -> Result<String> {
    let mut request = agent.get(&input.url);

    // Add authentication
    if let Some(auth_type) = &input.auth_type {
        request = match auth_type {
            AuthType::Bearer => {
                let token = input.auth_value.as_ref().unwrap();
                request.set("Authorization", &format!("Bearer {}", token))
            }
            AuthType::ApiKey => {
                let key = input.auth_value.as_ref().unwrap();
                request.set("X-API-Key", key)
            }
            AuthType::Basic => {
                // Implement basic auth
                request
            }
        };
    }

    // ... rest of implementation
}
```

**Impact:** Unlocks 90% of real-world APIs

---

### 2. OpenAPI/Swagger ❌ (High Priority)

**What's Blocked:**

```yaml
# openapi.yaml
openapi: 3.0.0
paths:
  /users:
    get:
      responses:
        '200':
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/User'
```

Can't generate types from this!

**What's Needed:**

```rust
openapi_to_rust! {
    spec = "openapi.yaml",
    output = "src/api/"
}
```

**Implementation Plan (4-6 weeks):**

**Phase 1: Parser (2 weeks)**
```rust
// New crate: parsers/openapi_parser/src/lib.rs
use openapiv3::OpenAPI;
use unistructgen_core::*;

pub struct OpenAPIParser {
    spec: OpenAPI,
}

impl Parser for OpenAPIParser {
    type Error = OpenAPIError;

    fn parse(&mut self, input: &str) -> Result<IRModule> {
        // 1. Parse OpenAPI YAML/JSON
        let spec: OpenAPI = serde_yaml::from_str(input)?;

        // 2. Extract schemas
        let schemas = spec.components
            .as_ref()
            .and_then(|c| c.schemas.as_ref())
            .ok_or(OpenAPIError::NoSchemas)?;

        // 3. Convert to IR
        let mut module = IRModule::new("OpenAPI".to_string());

        for (name, schema) in schemas {
            let ir_struct = self.schema_to_struct(name, schema)?;
            module.add_type(IRType::Struct(ir_struct));
        }

        Ok(module)
    }
}

impl OpenAPIParser {
    fn schema_to_struct(&self, name: &str, schema: &Schema) -> Result<IRStruct> {
        // Convert OpenAPI schema to IRStruct
        // Handle $ref, allOf, oneOf, anyOf
        // Map OpenAPI types to Rust types
    }
}
```

**Phase 2: Validation (1 week)**
```rust
// Add validation support
use validator::Validate;

pub struct User {
    #[validate(email)]
    pub email: String,

    #[validate(length(min = 1, max = 100))]
    pub name: String,

    #[validate(range(min = 0, max = 150))]
    pub age: i32,
}
```

**Phase 3: API Client Generation (2 weeks)**
```rust
// Generate API client trait
#[async_trait]
pub trait UserApi {
    async fn get_user(&self, id: i64) -> Result<User>;
    async fn create_user(&self, user: CreateUserRequest) -> Result<User>;
    async fn update_user(&self, id: i64, user: UpdateUserRequest) -> Result<User>;
    async fn delete_user(&self, id: i64) -> Result<()>;
}

// Implementation
pub struct UserApiClient {
    client: reqwest::Client,
    base_url: String,
}

#[async_trait]
impl UserApi for UserApiClient {
    async fn get_user(&self, id: i64) -> Result<User> {
        let url = format!("{}/users/{}", self.base_url, id);
        let user = self.client.get(&url)
            .send()
            .await?
            .json::<User>()
            .await?;
        Ok(user)
    }
    // ... other methods
}
```

**Phase 4: Proc Macro (1 week)**
```rust
openapi_to_rust! {
    spec = "openapi.yaml",
    output = "src/api/",
    generate_client = true,
    generate_validation = true
}
```

**Impact:**
- Opens enterprise market
- Competitive with quicktype
- Standard for Rust API clients

---

### 3. Schema Validation ❌ (Medium Priority)

**What's Blocked:**

```rust
// No runtime validation
let user = User {
    email: "invalid-email",  // Should fail but doesn't
    age: -5,                 // Invalid age but no check
};
```

**What's Needed:**

```rust
use validator::Validate;

#[derive(Validate)]
pub struct User {
    #[validate(email)]
    pub email: String,

    #[validate(range(min = 0, max = 150))]
    pub age: i32,
}

let user = User { /* ... */ };
user.validate()?;  // ✅ Validates
```

**Implementation Plan (2 weeks):**

```rust
// In json_parser, detect validation rules
impl JsonParser {
    fn infer_validation(&self, field_name: &str, value: &Value) -> Vec<String> {
        let mut validations = Vec::new();

        // Email detection
        if field_name.contains("email") {
            if let Value::String(s) = value {
                if s.contains('@') {
                    validations.push("email".to_string());
                }
            }
        }

        // URL detection
        if field_name.contains("url") || field_name.contains("link") {
            validations.push("url".to_string());
        }

        // Range detection for age
        if field_name == "age" {
            validations.push("range(min = 0, max = 150)".to_string());
        }

        validations
    }
}

// Add to IRField
pub struct IRField {
    // ... existing fields ...
    pub validations: Vec<String>,
}

// In RustRenderer, generate validation attributes
if !field.validations.is_empty() {
    for validation in &field.validations {
        writeln!(w, "    #[validate({})]", validation)?;
    }
}
```

**Impact:** Production-ready validation

---

### 4. Multi-Sample Merging ❌ (High Value)

**What's Blocked:**

```json
// Sample 1
{"name": "Alice", "email": "alice@example.com", "age": 30}

// Sample 2
{"name": "Bob", "age": 25}  // Missing email!

// Today: Generates email: String (WRONG!)
// Should: email: Option<String>
```

**What's Needed:**

```rust
generate_from_samples! {
    name = "User",
    samples = ["sample1.json", "sample2.json", "sample3.json"]
}

// Generates:
pub struct User {
    pub name: String,           // Present in all
    pub email: Option<String>,  // Missing in some
    pub age: i64,               // Present in all
}
```

**Implementation Plan (2 weeks):**

```rust
// New function in json_parser
pub fn merge_samples(samples: Vec<&str>, struct_name: &str) -> Result<IRModule> {
    let mut field_presence: HashMap<String, usize> = HashMap::new();
    let mut field_types: HashMap<String, IRTypeRef> = HashMap::new();

    // Parse all samples
    for sample in &samples {
        let value: Value = serde_json::from_str(sample)?;

        if let Value::Object(obj) = value {
            for (key, val) in obj {
                // Track presence
                *field_presence.entry(key.clone()).or_insert(0) += 1;

                // Infer type
                let ty = infer_type(&val, &key)?;

                // Check for type conflicts
                if let Some(existing_ty) = field_types.get(&key) {
                    if existing_ty != &ty {
                        // Type mismatch - make it Option<serde_json::Value>
                        field_types.insert(key.clone(),
                            IRTypeRef::Option(Box::new(IRTypeRef::Primitive(PrimitiveKind::Json)))
                        );
                    }
                } else {
                    field_types.insert(key.clone(), ty);
                }
            }
        }
    }

    // Build struct
    let mut ir_struct = IRStruct::new(struct_name.to_string());

    for (field_name, presence_count) in field_presence {
        let mut ty = field_types.get(&field_name).unwrap().clone();

        // If not present in all samples, make optional
        if presence_count < samples.len() {
            ty = IRTypeRef::Option(Box::new(ty));
        }

        ir_struct.add_field(IRField::new(field_name, ty));
    }

    let mut module = IRModule::new(struct_name.to_string());
    module.add_type(IRType::Struct(ir_struct));

    Ok(module)
}
```

**Impact:** Accurate type inference from real data

---

### 5. Database Schema Sync ❌ (High Value)

**What's Blocked:**

```sql
CREATE TABLE users (
    id UUID PRIMARY KEY,
    email VARCHAR(255) NOT NULL,
    created_at TIMESTAMP NOT NULL
);
```

Can't generate Rust types from this!

**What's Needed:**

```rust
sql_to_rust! {
    connection = "postgres://localhost/mydb",
    table = "users"
}
```

**Implementation Plan (6-8 weeks):**

**Phase 1: SQL Parser (3 weeks)**
```rust
// New crate: parsers/sql_parser/src/lib.rs
use sqlparser::ast::Statement;
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::parser::Parser as SqlParser;

pub struct SqlSchemaParser {
    dialect: Box<dyn Dialect>,
}

impl Parser for SqlSchemaParser {
    fn parse(&mut self, input: &str) -> Result<IRModule> {
        let statements = SqlParser::parse_sql(&*self.dialect, input)?;

        let mut module = IRModule::new("Database".to_string());

        for statement in statements {
            if let Statement::CreateTable { name, columns, .. } = statement {
                let struct_name = name.to_string();
                let ir_struct = self.table_to_struct(&struct_name, &columns)?;
                module.add_type(IRType::Struct(ir_struct));
            }
        }

        Ok(module)
    }
}

impl SqlSchemaParser {
    fn sql_type_to_ir(&self, sql_type: &DataType) -> IRTypeRef {
        match sql_type {
            DataType::Uuid => IRTypeRef::Primitive(PrimitiveKind::Uuid),
            DataType::Varchar(_) | DataType::Text => IRTypeRef::Primitive(PrimitiveKind::String),
            DataType::Integer => IRTypeRef::Primitive(PrimitiveKind::I32),
            DataType::BigInt => IRTypeRef::Primitive(PrimitiveKind::I64),
            DataType::Timestamp => IRTypeRef::Primitive(PrimitiveKind::DateTime),
            DataType::Boolean => IRTypeRef::Primitive(PrimitiveKind::Bool),
            // ... more types
        }
    }
}
```

**Phase 2: Database Introspection (2 weeks)**
```rust
// Query database for schema
pub async fn introspect_database(conn_str: &str) -> Result<IRModule> {
    let pool = PgPoolOptions::new().connect(conn_str).await?;

    let tables = sqlx::query!(
        "SELECT table_name FROM information_schema.tables
         WHERE table_schema = 'public'"
    )
    .fetch_all(&pool)
    .await?;

    let mut module = IRModule::new("Database".to_string());

    for table in tables {
        let columns = sqlx::query!(
            "SELECT column_name, data_type, is_nullable
             FROM information_schema.columns
             WHERE table_name = $1",
            table.table_name
        )
        .fetch_all(&pool)
        .await?;

        // Convert to IRStruct
        let ir_struct = columns_to_struct(&table.table_name, &columns)?;
        module.add_type(IRType::Struct(ir_struct));
    }

    Ok(module)
}
```

**Phase 3: ORM Integration (3 weeks)**
```rust
// Diesel integration
diesel_schema_to_rust! {
    migrations = "migrations/"
}

// SeaORM integration
sea_orm_to_rust! {
    entities = "entities/"
}

// SQLx integration
sqlx_to_rust! {
    database_url = env!("DATABASE_URL")
}
```

**Impact:**
- Keep DB and code in sync
- Eliminate schema drift
- Type-safe queries

---

## 📦 Step-by-Step: First Three Features

### Feature 1: Authentication Support

**Priority:** Critical
**Effort:** 1 week
**Impact:** Unlocks 90% of real APIs

**Week 1:**
```rust
// Day 1-2: Design API
struct_from_external_api! {
    struct_name = "User",
    url_api = "https://api.example.com/user",
    auth = Bearer(env!("API_TOKEN"))  // New parameter
}

// Day 3-4: Implement
// - Extend ExternalApiInput
// - Add auth_type, auth_value fields
// - Update fetch_json_from_api
// - Add header support

// Day 5: Test & Document
// - Test with GitHub API
// - Test with private APIs
// - Write examples
// - Update docs
```

### Feature 2: Multi-Sample Merging

**Priority:** High
**Effort:** 2 weeks
**Impact:** Accurate optional field detection

**Week 1:**
```rust
// Day 1-3: Implement merging logic
fn merge_samples(samples: Vec<&str>) -> Result<IRModule>

// Day 4-5: Handle edge cases
// - Type conflicts
// - Missing fields
// - Nested objects
```

**Week 2:**
```rust
// Day 1-2: Add proc macro
merge_from_samples! {
    name = "User",
    samples = ["sample1.json", "sample2.json"]
}

// Day 3-4: Testing
// - Unit tests
// - Integration tests
// - Real-world samples

// Day 5: Documentation
```

### Feature 3: YAML/TOML Support

**Priority:** Medium
**Effort:** 1 week
**Impact:** Config file use case

**Week 1:**
```rust
// Day 1-2: YAML parser
use serde_yaml;
// Reuse JSON infrastructure

// Day 3: TOML parser
use toml;
// Similar to YAML

// Day 4-5: Proc macros
yaml_to_rust! {
    file = "config.yaml",
    name = "Config"
}

toml_to_rust! {
    file = "config.toml",
    name = "Config"
}
```

---

## 💰 Monetization Opportunities

### 1. UniStructGen Cloud (SaaS)

**Service:** Hosted API type generation

```
Features:
- Web UI for generating types
- API endpoint for automation
- Schema versioning
- Team collaboration
- Private schema storage

Pricing:
- Free: 100 generations/month
- Pro: $29/month - 10,000 generations
- Team: $99/month - Unlimited + collaboration
- Enterprise: Custom pricing
```

**Revenue Potential:** $10K-$100K MRR

### 2. VSCode Extension (Freemium)

**Free:**
- Basic code generation
- Community support

**Pro ($5/month):**
- Live preview
- Auto-generate on save
- Advanced validation
- Priority support

**Revenue Potential:** $5K-$50K MRR

### 3. Enterprise Support

**Offering:**
- Dedicated support
- Custom features
- Training
- SLA guarantees

**Pricing:** $10K-$100K/year

### 4. Training & Certification

**Courses:**
- "Mastering UniStructGen" - $199
- "API Development with Rust" - $299
- Enterprise training - $5K/day

**Revenue Potential:** $50K-$200K/year

---

## 🌍 Community Projects

### Ideas for Contributors

1. **unistructgen-examples** - Curated examples repo
2. **awesome-unistructgen** - List of projects using it
3. **unistructgen-templates** - Common patterns
4. **unistructgen-benchmarks** - Performance comparisons
5. **unistructgen-plugins** - Community extensions

### Hackathon Ideas

1. **Best API Client** - Most complete API client generated
2. **Most Creative Use** - Novel application
3. **Best Documentation** - Auto-generated docs
4. **Performance Champion** - Fastest generation

---

## 🎯 Summary: What to Build First

### Month 1: Quick Wins
1. ✅ Authentication support (1 week)
2. ✅ Multi-sample merging (2 weeks)
3. ✅ YAML/TOML support (1 week)

### Month 2: High Impact
4. ✅ Schema validation (2 weeks)
5. ✅ Watch mode (1 week)
6. ✅ Better error messages (1 week)

### Month 3-4: Game Changers
7. ✅ OpenAPI support (6 weeks)
8. ✅ Documentation generator (2 weeks)

### Month 5-6: Ecosystem
9. ✅ VSCode extension (6 weeks)
10. ✅ Database sync (6 weeks)

**After 6 months:** Production-ready, enterprise-grade, ecosystem leader.

---

<div align="center">

**From good library to indispensable tool in 6 months.**

The roadmap is clear. Time to execute. 🚀

</div>
