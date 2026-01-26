# UniStructGen
## Author
[Maxim Bogovic](https://bogovick.com)

**Rust code generation toolkit for type-safe structs, AI tool calling, structured LLM outputs, and compiler-driven AI agents.**

Parse any schema. Generate idiomatic Rust. Build AI agents with type safety from end to end.

```rust
use unistructgen_proc_macro::ai_tool;
use unistructgen_core::ToolRegistry;

/// Calculate shipping cost based on weight and destination
#[ai_tool]
fn calculate_shipping(weight_kg: f64, destination: String) -> f64 {
    weight_kg * 2.5 + if destination == "international" { 15.0 } else { 5.0 }
}

let mut registry = ToolRegistry::new();
registry.register(CalculateShippingTool);

// JSON Schema auto-generated, OpenAI-compatible, ready to send to any LLM
let tool_definitions = registry.get_definitions();
let result = registry.execute("calculate_shipping", r#"{"weight_kg": 3.0, "destination": "domestic"}"#)?;
```

[![Crates.io](https://img.shields.io/crates/v/unistructgen.svg)](https://crates.io/crates/unistructgen)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)]()
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)]()

---

## Killer Features

### `#[ai_tool]` -- Turn Any Function into an LLM Tool

One attribute. That's all it takes to make any Rust function callable by GPT-4, Claude, Llama, or any LLM that supports function calling.

```rust
use unistructgen_proc_macro::ai_tool;

/// Get current weather for a city
#[ai_tool]
fn get_weather(city: String, units: String) -> String {
    format!("Weather in {}: 22{}", city, if units == "celsius" { "C" } else { "F" })
}
```

The `#[ai_tool]` macro automatically:

1. **Extracts the function signature** and parses argument names + types
2. **Generates a JSON Schema** (Draft 2020-12) from the IR type system -- not hand-written, not a string, derived from actual Rust types
3. **Creates a tool struct** (`GetWeatherTool`) implementing the `AiTool` trait
4. **Handles JSON deserialization** of arguments and execution dispatch

The generated tool struct works with the `ToolRegistry`:

```rust
use unistructgen_core::ToolRegistry;

let mut registry = ToolRegistry::new();
registry.register(CalculateShippingTool);
registry.register(GetWeatherTool);

// Export OpenAI-compatible tool definitions for any LLM
let definitions = registry.get_definitions();
// [
//   {
//     "type": "function",
//     "function": {
//       "name": "calculate_shipping",
//       "description": "Calculate shipping cost based on weight and destination",
//       "parameters": {
//         "type": "object",
//         "properties": {
//           "weight_kg": { "type": "number" },
//           "destination": { "type": "string" }
//         },
//         "required": ["weight_kg", "destination"]
//       }
//     }
//   },
//   ...
// ]

// Execute when LLM calls a tool
let result = registry.execute("get_weather", r#"{"city": "Tokyo", "units": "celsius"}"#)?;
```

No boilerplate. No manual JSON Schema. No serialization glue. One `#[ai_tool]` and your function is LLM-ready.

---

### JSON Schema Generator -- Structured Outputs for LLMs

Generate Draft 2020-12 JSON Schema from any struct definition and use it as a contract for LLM structured outputs:

```rust
use unistructgen_core::{StructGen, FieldType};
use unistructgen_codegen::JsonSchemaRenderer;
use unistructgen_core::CodeGenerator;

// Define the response shape
let module = StructGen::new()
    .name("AgentResponse")
    .field("answer", FieldType::String)
    .field("confidence", FieldType::F64)
    .field("sources", FieldType::vec(FieldType::String))
    .field("requires_action", FieldType::Bool)
    .build_ir_module();

// Generate JSON Schema
let renderer = JsonSchemaRenderer::new().fragment();
let schema = renderer.generate(&module)?;
```

Output:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "answer": { "type": "string" },
    "confidence": { "type": "number" },
    "sources": { "type": "array", "items": { "type": "string" } },
    "requires_action": { "type": "boolean" }
  },
  "required": ["answer", "confidence", "sources", "requires_action"],
  "additionalProperties": false
}
```

Send this schema directly to OpenAI's `response_format.json_schema` or inject it as a system prompt for Ollama -- the LLM module handles both:

```rust
use llm_utl::{openai::OpenAiClient, CompletionRequest, Message, LlmClient};

let client = OpenAiClient::new("your-key", "gpt-4o");
let response = client.complete(CompletionRequest {
    messages: vec![Message::user("Analyze this codebase")],
    response_schema: Some(schema_value),  // JSON Schema from above
    ..Default::default()
}).await?;
// Response is guaranteed to match your schema
```

Features of the schema generator:
- **`$defs` with `$ref`** for nested types and cross-references
- **Recursive type support** (types referencing each other)
- **Strict mode** (`additionalProperties: false`) for OpenAI compatibility
- **Fragment mode** (`.fragment()`) omits `$schema` for embedding in larger payloads
- **All IR types mapped**: primitives, `Option<T>`, `Vec<T>`, `HashMap<K,V>`, named references, enums as string unions

---

### AI Validation Loop -- Self-Healing LLM Responses

LLMs produce malformed JSON. UniStructGen catches errors and generates correction prompts that the LLM can understand:

```rust
use unistructgen_core::{ValidationReport, AiValidationError, map_serde_error};

// Try to parse LLM response
let response_json = ai_client.complete(request).await?;

for attempt in 0..3 {
    match serde_json::from_str::<AgentResponse>(&response_json) {
        Ok(valid) => {
            println!("Valid response: {:?}", valid);
            break;
        }
        Err(e) => {
            // Convert serde error to AI-friendly format
            let ai_error = map_serde_error(&e);
            // ai_error.path = "confidence"
            // ai_error.message = "invalid type: string \"high\", expected f64"

            let mut report = ValidationReport::new();
            report.add_error(ai_error);

            // Generate correction prompt for the LLM
            let correction = report.to_correction_prompt();
            // "The generated JSON response was invalid. Please fix the following errors:
            //  1. Field `confidence`: invalid type: string "high", expected f64
            //     Hint: Ensure the field name and type matches the schema exactly.
            //  Return the corrected JSON only."

            // Send correction back to LLM
            response_json = ai_client.complete(CompletionRequest {
                messages: vec![
                    Message::system("Fix the JSON"),
                    Message::user(&correction),
                ],
                response_schema: Some(schema.clone()),
                ..Default::default()
            }).await?;
        }
    }
}
```

The validation system provides:
- **`AiValidationError`** with `path`, `message`, `invalid_value`, `correction_hint`
- **`ValidationReport`** that aggregates errors and generates correction prompts
- **`map_serde_error()`** that converts serde errors to structured AI-readable errors with field path extraction
- **`AiValidatable` trait** for types that can self-validate

---

### Compiler-Driven Development Loop -- AI That Fixes Its Own Code

Build AI agents that write Rust code, compile it in a sandbox, and iterate on compiler errors:

```rust
// 1. Create ephemeral Rust project
let sandbox = RustSandbox::new()?;

// 2. AI generates code
let code = ai_generate("Write validate_email(email: &str) -> bool");

for attempt in 0..5 {
    // 3. Write to sandbox
    sandbox.write_code(&code)?;

    // 4. Compile and get structured errors
    let errors = Compiler::check(sandbox.path())?;
    // Each CompilerError has: message, location (file:line:col), rendered output

    if errors.is_empty() {
        println!("Code compiles!");
        break;
    }

    // 5. Feed errors back to AI
    let feedback = errors.iter()
        .map(|e| format!("Error at {}: {}", e.location.as_deref().unwrap_or("?"), e.message))
        .collect::<Vec<_>>()
        .join("\n");

    code = ai_generate(&format!("Fix these errors:\n{}\n\nOriginal code:\n{}", feedback, code));
}
```

The `code-agent` example demonstrates this full loop with:
- **`RustSandbox`** -- ephemeral `cargo` project with pre-configured dependencies
- **`Compiler::check()`** -- runs `cargo check --message-format=json` and parses structured diagnostics
- **`extract_rust_code()`** -- extracts Rust code from markdown-formatted AI responses
- Automatic iteration until compilation succeeds

---

### Compile-Time API Fetching -- Structs from Live APIs

Fetch a JSON API at compile time and generate type-safe structs. No manual type definitions. No code generation scripts. Just point to a URL:

```rust
use unistructgen_proc_macro::struct_from_external_api;

struct_from_external_api! {
    struct_name = "GithubRepo",
    url_api = "https://api.github.com/repos/rust-lang/rust",
    method = "GET",
    auth_bearer = "ghp_your_token",
    serde = true,
    optional = true,
    max_depth = 3,           // Limit nested object depth
    max_entity_count = 10    // Limit array items for inference
}

// GithubRepo struct is now available with all fields from the API response
let repo: GithubRepo = reqwest::get("https://api.github.com/repos/rust-lang/rust")
    .await?.json().await?;
println!("{}", repo.full_name);
```

Authentication methods:
- `auth_bearer = "token"` -- Bearer/OAuth2/JWT
- `auth_api_key = "X-API-Key:value"` -- Custom header
- `auth_basic = "user:password"` -- HTTP Basic

The macro:
1. Makes an HTTP request during compilation
2. Infers types from the JSON response (arrays use the first element)
3. Applies depth/count limits to prevent deeply nested or huge types
4. Generates the struct with all fields, nested structs, and serde attributes

---

### LLM Client Abstraction -- OpenAI + Ollama with Structured Outputs

Unified async trait for any LLM provider, with built-in structured output support:

```rust
use llm_utl::{LlmClient, CompletionRequest, Message};

// OpenAI with structured outputs
use llm_utl::openai::OpenAiClient;
let openai = OpenAiClient::new("sk-...", "gpt-4o");

// Ollama (local LLMs) with JSON mode
use llm_utl::ollama::OllamaClient;
let ollama = OllamaClient::new("llama3");

// Same interface for both
async fn ask(client: &dyn LlmClient, schema: Option<Value>) -> String {
    client.complete(CompletionRequest {
        messages: vec![
            Message::system("You are a helpful assistant"),
            Message::user("Analyze this data"),
        ],
        response_schema: schema,  // JSON Schema for structured output
        temperature: Some(0.7),
        max_tokens: Some(1000),
        ..Default::default()
    }).await.unwrap()
}
```

How structured outputs work per provider:
- **OpenAI**: Uses `response_format.json_schema` with `strict: true` (native support)
- **Ollama**: Enables `format: "json"` + injects schema into system prompt (works with Llama 3, Mistral, etc.)

---

### 6 Parsers -- Any Schema to Rust

Generate Rust structs from any of these formats at compile time (proc macros) or runtime (pipeline):

```rust
// JSON
generate_struct_from_json! {
    name = "User",
    json = r#"{"id": 1, "name": "Alice", "tags": ["admin"]}"#,
    serde = true
}

// OpenAPI / Swagger
openapi_to_rust! {
    file = "api/openapi.yaml",
    generate_client = true,
    generate_validation = true
}

// SQL DDL
generate_struct_from_sql! {
    sql = r#"CREATE TABLE users (
        id SERIAL PRIMARY KEY,
        name VARCHAR(100) NOT NULL,
        email VARCHAR(255) UNIQUE,
        created_at TIMESTAMP DEFAULT NOW()
    );"#,
    serde = true
}

// GraphQL Schema
generate_struct_from_graphql! {
    schema = r#"
        type User { id: ID!, name: String!, email: String, posts: [Post!]! }
        type Post { id: ID!, title: String!, body: String! }
    "#,
    serde = true
}

// .env files
generate_struct_from_env! {
    name = "AppConfig",
    env = r#"
        DATABASE_URL=postgres://localhost/mydb
        PORT=8080
        DEBUG=true
    "#
}

// Markdown tables
// Use MarkdownParser at runtime or CLI: unistructgen generate --input schema.md
```

---

### RAG Semantic Chunker -- Split Docs for Retrieval

Built-in semantic chunking for markdown documents, designed for RAG pipelines:

```rust
use unistructgen_markdown_parser::chunker::SemanticChunker;

let markdown = std::fs::read_to_string("docs/README.md")?;
let chunks = SemanticChunker::chunk(&markdown);

for chunk in &chunks {
    // Each chunk preserves heading hierarchy and semantic boundaries
    // Ready to embed and store in a vector database
    vector_db.insert(embed(&chunk.content), chunk.path.clone());
}
```

---

## Full Feature List

| Category | Feature | Description |
|---|---|---|
| **AI Tools** | `#[ai_tool]` macro | Auto-generate JSON Schema + tool struct from any function |
| **AI Tools** | `ToolRegistry` | Register, discover, and execute tools by name with JSON args |
| **AI Tools** | OpenAI-compatible definitions | `get_definitions()` returns tool specs ready for any LLM API |
| **Structured Output** | `JsonSchemaRenderer` | Generate Draft 2020-12 JSON Schema from IR types |
| **Structured Output** | Structured output support | OpenAI native `json_schema` + Ollama JSON mode |
| **Structured Output** | Schema features | `$defs`/`$ref`, recursive types, strict mode, fragment mode |
| **Validation** | `ValidationReport` | Aggregate errors from LLM responses |
| **Validation** | `to_correction_prompt()` | Auto-generate LLM-readable fix instructions |
| **Validation** | `map_serde_error()` | Convert serde errors to AI-friendly format with field paths |
| **Agent Infra** | Compiler sandbox | Ephemeral Rust projects for AI-generated code |
| **Agent Infra** | Compiler diagnostics | Structured error extraction from `cargo check` |
| **Agent Infra** | Code extraction | Parse Rust code from markdown AI responses |
| **Agent Infra** | Semantic chunker | Split markdown docs into RAG-ready chunks |
| **LLM Clients** | OpenAI client | Async client with structured outputs support |
| **LLM Clients** | Ollama client | Local LLM support with JSON mode |
| **LLM Clients** | `LlmClient` trait | Unified async interface for any provider |
| **Code Gen** | 6 parsers | JSON, OpenAPI, SQL, GraphQL, Markdown, .env |
| **Code Gen** | Compile-time macros | Zero runtime cost struct generation |
| **Code Gen** | Compile-time API fetch | `struct_from_external_api!` with auth |
| **Code Gen** | Fluent builder API | `StructGen`, `EnumGen`, `ModuleGen`, `FieldBuilder` |
| **Code Gen** | Pipeline | Pluggable parser + transformer + generator chain |
| **Code Gen** | Field constraints | Length, range, pattern, format validation attributes |
| **Code Gen** | Plugin system | Before/after hooks for parsing and generation |
| **Code Gen** | Visitor pattern | IR traversal, analysis, and validation |
| **Code Gen** | Transformers | `FieldOptionalizer`, `DocCommentAdder`, `TypeDeduplicator`, `FieldRenamer` |

---

## 🎓 Learning Resources

- 📖 **[Quick Start Guide](QUICKSTART.md)** - Get started in 5 minutes
- 📚 **[Complete Examples](EXAMPLES.md)** - Real-world usage patterns
- 🔧 **[API Documentation](https://docs.rs/unistructgen)** - Full API reference
- 🌐 **[External API Guide](docs/EXTERNAL_API_GUIDE.md)** - Advanced API integration
- 🎯 **[Best Practices](docs/BEST_PRACTICES.md)** - Tips and tricks

---

## Quick Start

```toml
[dependencies]
unistructgen-core = "0.1"       # IR, traits, ToolRegistry, validation
unistructgen-codegen = "0.1"    # Rust renderer + JSON Schema generator
unistructgen-proc-macro = "0.1" # All proc macros + #[ai_tool]
unistructgen-llm = "0.1"        # LLM clients (OpenAI, Ollama)

# Individual parsers (pick what you need)
unistructgen-json-parser = "0.1"
unistructgen-openapi-parser = "0.1"
unistructgen-sql-parser = "0.1"
unistructgen-graphql-parser = "0.1"
unistructgen-markdown-parser = "0.1"
unistructgen-env-parser = "0.1"
```

---

## Builder API

Build structs and enums programmatically with full control:

```rust
use unistructgen_core::{StructGen, EnumGen, ModuleGen, FieldType, FieldBuilder};

let code = StructGen::new()
    .name("User")
    .doc("Represents a user in the system")
    .field("id", FieldType::I64)
    .field("name", FieldType::String)
    .field_optional("email", FieldType::String)
    .field_with(|f| f.doc("User's age").range(0.0, 150.0), "age", FieldType::I32)
    .with_serde()
    .with_default()
    .generate()?;

let code = EnumGen::new()
    .name("OrderStatus")
    .variant("Pending")
    .variant_with_rename("InTransit", "in_transit")
    .variant("Delivered")
    .with_serde()
    .generate()?;

let code = ModuleGen::new("models")
    .add_struct(StructGen::new().name("User").field("id", FieldType::I64))
    .add_enum(EnumGen::new().name("Status").variant("Active"))
    .generate()?;
```

Field constraints generate `#[validate(...)]` attributes:

```rust
FieldBuilder::new("email", FieldType::String)
    .optional()
    .doc("User email")
    .rename("user_email")       // #[serde(rename = "user_email")]
    .length(5, 255)             // #[validate(length(min = 5, max = 255))]
    .pattern(r"^[\w@.]+$")     // #[validate(regex = "...")]
    .format("email")           // #[validate(email)]
    .build();
```

---

## Pipeline API

Chain parsers, transformers, and generators:

```rust
use unistructgen_core::{Pipeline, transformer::FieldOptionalizer};
use unistructgen_json_parser::{JsonParser, ParserOptions};
use unistructgen_codegen::{RustRenderer, RenderOptions};

let mut pipeline = Pipeline::new(
    JsonParser::new(ParserOptions {
        struct_name: "User".into(),
        derive_serde: true,
        ..Default::default()
    }),
    RustRenderer::new(RenderOptions::default()),
)
.add_transformer(Box::new(FieldOptionalizer::new()));

let rust_code = pipeline.execute(r#"{"id": 1, "name": "Alice"}"#)?;
```

---

## CLI

```bash
cargo install unistructgen-cli

# Generate from JSON
unistructgen generate --input data.json --name MyStruct --serde

# Generate from Markdown table
unistructgen generate --input schema.md --name Config

# Generate HTTP client from OpenAPI
unistructgen client --spec api.yaml --name GitHub --output ./generated
```

---

## Architecture

```
                    ┌──────────────────────────────────────────────────────────────┐
                    │                        UniStructGen                          │
                    │                                                              │
  ┌──────────┐     │  ┌────────┐    ┌────┐    ┌─────────────┐    ┌──────────┐    │
  │  JSON    │──┐  │  │        │    │    │    │             │    │  Rust    │    │
  │  OpenAPI │──┤  │  │        │    │    │    │             │    │  Code    │    │
  │  SQL     │──┤  │  │ Parser │──▶│ IR │──▶│ Transformer │──▶│  JSON    │    │
  │  GraphQL │──┼─▶│  │        │    │    │    │             │    │  Schema  │    │
  │  .env    │──┤  │  │        │    │    │    │             │    │          │    │
  │  Markdown│──┘  │  └────────┘    └────┘    └─────────────┘    └──────────┘    │
  └──────────┘     │  [Plugins]                [Plugins]                         │
                   └──────────────────────────────────────────────────────────────┘
                                        │
                            ┌───────────┼───────────┐
                            ▼           ▼           ▼
                     ┌────────────┐ ┌────────┐ ┌──────────┐
                     │ #[ai_tool] │ │  LLM   │ │Validation│
                     │ ToolRegist │ │ Client │ │  Loop    │
                     │ JSON Schema│ │OpenAI  │ │ Reports  │
                     └────────────┘ │Ollama  │ │ Prompts  │
                                    └────────┘ └──────────┘
```

### Extensible via Traits

| Trait | Purpose | Implementations |
|---|---|---|
| `Parser` | Input format to IR | `JsonParser`, `OpenApiParser`, `SqlParser`, `GraphqlParser`, `MarkdownParser`, `EnvParser` |
| `CodeGenerator` | IR to output | `RustRenderer`, `JsonSchemaRenderer` |
| `IRTransformer` | Transform IR | `FieldOptionalizer`, `DocCommentAdder`, `TypeDeduplicator`, `FieldRenamer` |
| `Plugin` | Pipeline hooks | `LoggingPlugin`, `HeaderPlugin`, custom |
| `AiTool` | LLM tool interface | Auto-generated by `#[ai_tool]` |
| `LlmClient` | LLM provider | `OpenAiClient`, `OllamaClient` |
| `AiValidatable` | Self-validation | Custom types |
| `IRVisitor` | IR traversal | `StructNameCollector`, `FieldCounter`, `IRValidator` |

---

## Examples

| Example | What It Demonstrates |
|---|---|
| **`tools-agent`** | Register functions as AI tools, export OpenAI definitions, execute tool calls |
| **`docu-agent`** | RAG ingestion with semantic chunking, JSON Schema contract, AI validation loop with auto-correction |
| **`code-agent`** | Compiler-driven development: AI writes code, sandbox compiles, errors fed back, AI fixes iteratively |
| **`github-client`** | Full GitHub API client generated from OpenAPI spec |
| **`blog-api`** | Blog API types from OpenAPI |
| **`api-example`** | Struct generation from live API responses |
| **`proc-macro-example`** | All proc macros: JSON, OpenAPI, SQL, GraphQL, .env |

---

## Workspace Structure

```
unistructgen/
├── core/                    # IR, traits, pipeline, plugins, ToolRegistry, validation
├── codegen/                 # RustRenderer + JsonSchemaRenderer
├── parsers/
│   ├── json_parser/         # JSON → IR (smart type inference)
│   ├── openapi_parser/      # OpenAPI/Swagger → IR + HTTP client gen
│   ├── sql_parser/          # SQL DDL → IR
│   ├── graphql_parser/      # GraphQL schema → IR
│   ├── markdown_parser/     # Markdown tables → IR + SemanticChunker
│   └── env_parser/          # .env → IR
├── proc-macro/              # All proc macros + #[ai_tool]
├── llm/                     # LLM clients (OpenAI, Ollama)
├── cli/                     # CLI binary
└── examples/
    ├── tools-agent/         # AI tool registry demo
    ├── docu-agent/          # RAG + validation loop demo
    ├── code-agent/          # Compiler-driven AI coding demo
    ├── github-client/       # Generated GitHub API client
    └── ...
```

---

## Type Mapping

| IR Type | Rust | JSON Schema | Source Examples |
|---|---|---|---|
| `String` | `String` | `"string"` | JSON string, `VARCHAR`, GraphQL `String` |
| `I32` | `i32` | `"integer"` | JSON int, `INT`, GraphQL `Int` |
| `I64` | `i64` | `"integer"` | `BIGINT`, large numbers |
| `F64` | `f64` | `"number"` | JSON float, `DOUBLE`, GraphQL `Float` |
| `Bool` | `bool` | `"boolean"` | `BOOLEAN`, GraphQL `Boolean` |
| `DateTime` | `chrono::DateTime<Utc>` | `"string" format:"date-time"` | `TIMESTAMP` |
| `Uuid` | `uuid::Uuid` | `"string" format:"uuid"` | `UUID` columns |
| `Decimal` | `rust_decimal::Decimal` | `"number"` | `DECIMAL`, `NUMERIC` |
| `Option(T)` | `Option<T>` | omitted from `required` | nullable, optional fields |
| `Vec(T)` | `Vec<T>` | `"array"` | JSON arrays, `[Type]` |
| `Map(K,V)` | `HashMap<K,V>` | `"object" + additionalProperties` | dynamic key objects |
| `Named(S)` | `S` | `"$ref": "#/$defs/S"` | nested objects, type refs |

---

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
