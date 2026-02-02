# UniStructGen

**Rust toolkit for type-safe code generation, AI tool calling, structured LLM outputs, and compiler-driven agents.**

Parse JSON, OpenAPI, SQL, GraphQL, Markdown, or .env schemas into a language-agnostic intermediate representation (IR), then generate idiomatic Rust structs, JSON Schema for LLM structured outputs, or wire up AI tool calling -- all with compile-time safety.

**Author**: [Maxim Bogovic](https://bogovick.com)
**Version**: 0.1.0
**License**: MIT / Apache-2.0
**Rust**: 1.70+

[![Crates.io](https://img.shields.io/crates/v/unistructgen.svg)](https://crates.io/crates/unistructgen)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)]()
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)]()

---

**Why developers use UniStructGen**
- **Ship types fast** — generate Rust structs from real JSON and schemas at compile time.
- **Keep LLM tools correct** — auto‑generate JSON Schemas and tool definitions from Rust functions.
- **Reduce boilerplate** — one source of truth for types, validation, and docs.

Try the killer example:
```bash
cargo run -p killer-example
```

---

## What Problem Does This Solve

You have data schemas -- JSON payloads, database DDL, OpenAPI specs, GraphQL types, environment variables. You need Rust structs that match. You also need JSON Schema to tell an LLM exactly what shape of response you expect. And you need to turn plain Rust functions into tools the LLM can call.

UniStructGen gives you one pipeline for all of this:

```
Schema (JSON/SQL/OpenAPI/GraphQL/.env/Markdown)
    |
    v
  Parser  -->  IR (Intermediate Representation)  -->  Generator
    |                                                      |
    v                                                      v
  Rust structs         or         JSON Schema (Draft 2020-12)
```

Instead of hand-writing struct definitions, JSON Schema, serde attributes, and tool boilerplate, you describe the shape once and generate everything.

---

## Project Status

**Stable core:** `core/`, `codegen/`, `parsers/*`, `proc-macro/`, `cli/` are the primary developer-facing surface and should remain backward compatible within minor versions.

**Experimental/optional:** `llm/`, `mcp/`, `agent/`, and `schema-registry/` are evolving and may change more frequently.

**Compile-time fetch controls:** set `UNISTRUCTGEN_FETCH_OFFLINE=1` to disable network, `UNISTRUCTGEN_FETCH_CACHE=0` to disable caching, `UNISTRUCTGEN_FETCH_CACHE_DIR=/path` to override cache location, and `UNISTRUCTGEN_FETCH_TIMEOUT_MS=...` to override timeouts.

---

## Table of Contents

- [Quick Start](#quick-start)
- [Killer Example (60 Seconds)](#killer-example-60-seconds)
- [Core Feature: `#[ai_tool]` Macro](#core-feature-ai_tool-macro)
- [Core Feature: JSON Schema for Structured LLM Outputs](#core-feature-json-schema-for-structured-llm-outputs)
- [Core Feature: AI Validation Loop](#core-feature-ai-validation-loop)
- [Core Feature: Compiler Diagnostics for AI Agents](#core-feature-compiler-diagnostics-for-ai-agents)
- [Core Feature: Compile-Time API Fetching](#core-feature-compile-time-api-fetching)
- [Core Feature: LLM Client Abstraction](#core-feature-llm-client-abstraction)
- [All 6 Parsers](#all-6-parsers)
- [Builder API](#builder-api)
- [Pipeline API](#pipeline-api)
- [CLI](#cli)
- [Architecture](#architecture)
- [Crate Map](#crate-map)
- [Type Mapping Reference](#type-mapping-reference)
- [Examples](#examples)
- [Blog](#blog)
- [License](#license)

---

## Quick Start

Add the crates you need to `Cargo.toml`:

```toml
[dependencies]
# Core IR types, traits, ToolRegistry, validation, Context
unistructgen-core = "0.1"

# Rust code renderer + JSON Schema generator
unistructgen-codegen = "0.1"

# Proc macros: generate_struct_from_json!, #[ai_tool], openapi_to_rust!, etc.
unistructgen-macro = "0.1"

# LLM clients (OpenAI, Ollama) with structured output support
unistructgen-llm = "0.1"

# Parsers -- pick what you need
unistructgen-json-parser = "0.1"
unistructgen-openapi-parser = "0.1"
unistructgen-markdown-parser = "0.1"
# These parsers exist but are used primarily via proc-macros:
# unistructgen-sql-parser, unistructgen-graphql-parser, unistructgen-env-parser
```

Minimal example -- generate a Rust struct from JSON at compile time:

```rust
use unistructgen_macro::generate_struct_from_json;

generate_struct_from_json! {
    name = "User",
    json = r#"{"id": 1, "name": "Alice", "tags": ["admin"]}"#,
    serde = true
}

// Now `User` struct exists with fields: id (i64), name (String), tags (Vec<String>)
// Derives: Debug, Clone, PartialEq, Serialize, Deserialize
```

---

## Killer Example (60 Seconds)

One small program that shows the core value: **types + tool schemas + safe execution**.

```bash
cargo run -p killer-example
```

What it demonstrates:
- Compile-time Rust types from JSON
- LLM tool schema generation from functions
- Safe, structured tool execution

See: `examples/killer-example/README.md`

---

## Core Feature: `#[ai_tool]` Macro

Turn any Rust function into an LLM-callable tool with a single attribute. The macro generates a JSON Schema from the function signature, creates a tool struct implementing `AiTool`, and handles JSON argument deserialization.

```rust
use unistructgen_macro::ai_tool;
use unistructgen_core::{ToolRegistry, Context};

/// Calculate shipping cost based on weight and destination
#[ai_tool]
fn calculate_shipping(weight_kg: f64, destination: String) -> f64 {
    weight_kg * 2.5 + if destination == "international" { 15.0 } else { 5.0 }
}

// The macro generates `CalculateShippingTool` struct implementing `AiTool`

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut registry = ToolRegistry::new();
    registry.register(CalculateShippingTool);

    let context = Context::new();

    // Export OpenAI-compatible tool definitions for any LLM
    let definitions = registry.get_definitions();
    // Returns Vec<Value> in OpenAI function calling format:
    // [{"type": "function", "function": {"name": "calculate_shipping", ...}}]

    // Execute a tool call from LLM response
    let result = registry.execute(
        "calculate_shipping",
        r#"{"weight_kg": 3.0, "destination": "domestic"}"#,
        &context,
    ).await?;

    Ok(())
}
```

### What `#[ai_tool]` generates

Given a function `fn calculate_shipping(weight_kg: f64, destination: String) -> f64`:

1. **JSON Schema** (Draft 2020-12) derived from the Rust types via the IR type system
2. **`CalculateShippingTool` struct** implementing the `AiTool` trait
3. **Argument deserialization struct** with `serde::Deserialize`
4. **Description** extracted from the function's `///` doc comment

### Dependency injection with `#[context]`

Tools can access shared resources (database pools, API clients) via the `Context` container:

```rust
#[derive(Clone)]
struct DbPool { url: String }

/// Get user balance from database
#[ai_tool]
async fn get_user_balance(#[context] db: DbPool, user_id: i32) -> Result<f64, String> {
    // `db` is extracted from Context automatically
    Ok(1250.50)
}

// Setup
let mut context = Context::new();
context.insert(DbPool { url: "postgres://localhost/mydb".into() });

let mut registry = ToolRegistry::new();
registry.register(GetUserBalanceTool);

// Execute -- Context provides the DbPool, LLM provides user_id
let result = registry.execute("get_user_balance", r#"{"user_id": 42}"#, &context).await?;
```

### Parallel batch execution

```rust
use unistructgen_core::tools::ToolCall;

let calls = vec![
    ToolCall { name: "get_user_balance".into(), arguments: r#"{"user_id": 1}"#.into() },
    ToolCall { name: "calculate_shipping".into(), arguments: r#"{"weight_kg": 5.0, "destination": "domestic"}"#.into() },
];

let results = registry.execute_batch(calls, &context).await;
// Returns Vec<(String, ToolResult)> -- all executed concurrently
```

### Supported argument types

| Rust Type | JSON Schema Type | Notes |
|---|---|---|
| `String`, `&str` | `"string"` | |
| `i8`, `i16`, `i32` | `"integer"` | |
| `i64`, `isize` | `"integer"` | |
| `f32`, `f64` | `"number"` | |
| `bool` | `"boolean"` | |
| `Vec<T>` | `"array"` | Recursive |
| `Option<T>` | nullable | Recursive |

---

## Core Feature: JSON Schema for Structured LLM Outputs

Generate Draft 2020-12 JSON Schema from any IR module. Use it as a contract for OpenAI `response_format.json_schema` or inject into system prompts for Ollama.

```rust
use unistructgen_core::{StructGen, FieldType};
use unistructgen_codegen::JsonSchemaRenderer;
use unistructgen_core::CodeGenerator;

// Define the response structure
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
  "$defs": {
    "AgentResponse": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "answer": { "type": "string" },
        "confidence": { "type": "number" },
        "sources": { "type": "array", "items": { "type": "string" } },
        "requires_action": { "type": "boolean" }
      },
      "required": ["answer", "confidence", "sources", "requires_action"]
    }
  },
  "$ref": "#/$defs/AgentResponse"
}
```

### Using with OpenAI

```rust
use unistructgen_llm::{LlmClient, CompletionRequest, Message};
use unistructgen_llm::openai::OpenAiClient;

// Reads OPENAI_API_KEY from environment
let client = OpenAiClient::new("gpt-4o")?;

let schema_value: serde_json::Value = serde_json::from_str(&schema)?;

let response = client.complete(CompletionRequest {
    messages: vec![Message::user("Analyze this codebase")],
    response_schema: Some(schema_value), // strict: true is set automatically
    ..Default::default()
}).await?;
// Response is guaranteed to match the AgentResponse schema
```

### Schema features

- `$defs` with `$ref` for nested types and cross-references
- Recursive type support
- Strict mode (`additionalProperties: false`) for OpenAI compatibility
- Fragment mode (`.fragment()`) omits `$schema` for embedding in larger payloads
- All IR types mapped: primitives, `Option<T>`, `Vec<T>`, `HashMap<K,V>`, named references, enums as string unions

---

## Core Feature: Reverse IR (Rust -> IR -> Schema)

Define your types in Rust and generate the IR/Schema from them. This is the reverse of the standard flow, allowing you to use Rust as the Source of Truth.

```rust
use unistructgen_core::IntoIR;
use unistructgen_codegen::JsonSchemaRenderer;

#[derive(IntoIR)]
struct User {
    #[field(min_value = 1, doc = "Unique ID")]
    id: i64,
    
    #[field(max_length = 100)]
    name: String,
    
    #[field(format = "email", optional)]
    email: Option<String>,
}

// Get the IR definition at runtime
let definition = User::ir_definition().unwrap();

// Wrap in a module
let mut module = unistructgen_core::ir::IRModule::new("UserModule".to_string());
module.add_type(definition);

// Generate JSON Schema
let schema = JsonSchemaRenderer::new().generate(&module)?;
```

Supported `#[field]` attributes:
- `doc = "..."`: Overrides/adds documentation
- `min_length`, `max_length`: String/array length constraints
- `min_value`, `max_value`: Numeric range constraints
- `pattern = "..."`: Regex pattern
- `format = "..."`: Format string (e.g., "email", "date-time")
- `optional`: Force optionality in IR

---

## Core Feature: AI Validation Loop

LLMs produce malformed JSON. UniStructGen provides structured validation errors and auto-generated correction prompts to send back to the LLM for self-healing.

```rust
use unistructgen_core::{ValidationReport, AiValidationError, map_serde_error};

let mut response_json = llm_client.complete(request).await?;

for attempt in 0..3 {
    match serde_json::from_str::<AgentResponse>(&response_json) {
        Ok(valid) => break,
        Err(e) => {
            // Convert serde error to AI-friendly structured format
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

            response_json = llm_client.complete(CompletionRequest {
                messages: vec![Message::user(&correction)],
                response_schema: Some(schema.clone()),
                ..Default::default()
            }).await?;
        }
    }
}
```

### Validation types

| Type | Purpose |
|---|---|
| `AiValidationError` | Structured error with `path`, `message`, `invalid_value`, `correction_hint` |
| `ValidationReport` | Aggregates errors; generates correction prompts via `to_correction_prompt()` |
| `map_serde_error()` | Converts `serde_json::Error` to `AiValidationError` with field path extraction |
| `AiValidatable` trait | For types that can self-validate: `fn validate_ai(&self) -> ValidationReport` |

---

## Core Feature: Compiler Diagnostics for AI Agents

Build AI agents that write Rust code and iterate on compiler errors. The `diagnostics` module parses structured output from `cargo check --message-format=json`.

```rust
use unistructgen_core::diagnostics::CargoDiagnostics;
use std::path::Path;

// Run cargo check on a project directory
let errors = CargoDiagnostics::check(Path::new("./sandbox_project"))?;

for error in &errors {
    println!("Error: {}", error.message);
    println!("Rendered: {}", error.rendered);
    if let Some(span) = &error.primary_span {
        println!("At {}:{}:{}", span.file_name, span.line_start, span.column_start);
    }
}

// Feed errors back to AI for correction
if !errors.is_empty() {
    let feedback = errors.iter()
        .map(|e| e.rendered.clone())
        .collect::<Vec<_>>()
        .join("\n");
    // Send feedback to LLM for code correction
}
```

### Code patching from LLM output

The `patch` module provides `CodeFix` and `Hunk` structs for applying LLM-generated code fixes:

```rust
use unistructgen_core::patch::CodeFix;

// LLM can output structured fixes as JSON
let fix: CodeFix = serde_json::from_str(llm_response)?;
// fix.file_path, fix.explanation, fix.changes (Vec<Hunk>)

let fixed_code = fix.apply(&original_source)?;
```

---

## Core Feature: Compile-Time API Fetching

Fetch a JSON API at compile time and generate type-safe structs. No manual type definitions. No codegen scripts.

```rust
use unistructgen_macro::struct_from_external_api;

struct_from_external_api! {
    struct_name = "GithubRepo",
    url_api = "https://api.github.com/repos/rust-lang/rust",
    method = "GET",
    auth_bearer = "ghp_your_token",
    serde = true,
    optional = true,
    max_depth = 3,
    max_entity_count = 10
}

// GithubRepo struct is now available with all fields from the API response
```

### Authentication methods

| Parameter | Format | Protocol |
|---|---|---|
| `auth_bearer = "token"` | Bearer token | OAuth2, JWT |
| `auth_api_key = "X-API-Key:value"` | Custom header | API key |
| `auth_basic = "user:password"` | HTTP Basic Auth | Basic |

### Parameters

| Parameter | Type | Default | Description |
|---|---|---|---|
| `struct_name` | string | `"ApiResponse"` | Name of the generated struct |
| `url_api` / `url` | string | required | API endpoint URL |
| `method` | string | `"GET"` | HTTP method (GET, POST, PUT, DELETE) |
| `serde` | bool | `true` | Add Serialize/Deserialize derives |
| `default` | bool | `false` | Add Default derive |
| `optional` | bool | `false` | Make all fields Optional |
| `max_depth` | int | unlimited | Limit nested object depth |
| `max_entity_count` | int | unlimited | Limit array items used for inference |
| `timeout` | int | `30000` | Request timeout in ms |

---

## Core Feature: LLM Client Abstraction

Unified async trait for OpenAI and Ollama with built-in structured output support.

```rust
use unistructgen_llm::{LlmClient, CompletionRequest, Message};

// OpenAI (reads OPENAI_API_KEY from env)
use unistructgen_llm::openai::OpenAiClient;
let openai = OpenAiClient::new("gpt-4o")?;

// Ollama (local, defaults to http://localhost:11434)
use unistructgen_llm::ollama::OllamaClient;
let ollama = OllamaClient::new("llama3");

// Factory with auto-detection
use unistructgen_llm::{LlmClientFactory, Provider};
let client = LlmClientFactory::new()
    .with_provider(Provider::Auto) // OpenAI if key exists, else Ollama
    .with_model("gpt-4o")
    .build()?;
```

### `LlmClient` trait

```rust
#[async_trait]
pub trait LlmClient: Send + Sync {
    async fn complete(&self, request: CompletionRequest) -> Result<String>;
    async fn complete_stream(&self, request: CompletionRequest) -> Result<LlmStream>;
    fn model(&self) -> &str;
}
```

### `CompletionRequest` fields

| Field | Type | Description |
|---|---|---|
| `messages` | `Vec<Message>` | Conversation messages (system, user, assistant) |
| `temperature` | `Option<f32>` | Sampling temperature |
| `max_tokens` | `Option<u32>` | Max response tokens |
| `response_schema` | `Option<Value>` | JSON Schema for structured output |

### Structured output per provider

- **OpenAI**: Uses `response_format.json_schema` with `strict: true` (native API support)
// Ollama: Enables `format: "json"` and injects the schema into the system prompt

---

## Core Feature: MCP Server (Model Context Protocol)

Turn your Rust functions into an MCP Server compatible with Claude Desktop, Cursor, and Windsurf in one line.

```rust
use unistructgen_macro::ai_tool;
use unistructgen_core::{ToolRegistry, Context};
use unistructgen_mcp::serve_stdio;
use std::sync::Arc;

#[ai_tool]
fn query_database(sql: String) -> String { 
    // ... execute sql ...
    "Query result".to_string()
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut registry = ToolRegistry::new();
    registry.register(QueryDatabaseTool); // generated by macro

    // Run MCP server on stdio
    serve_stdio(Arc::new(registry), Context::new()).await?;
    Ok(())
}
```

This automatically implements the Model Context Protocol:
- `tools/list`: Exports your tools with full JSON Schema definitions
- `tools/call`: Executes your Rust functions with arguments provided by the LLM
- `initialize`: Handles handshake and capabilities

Supported transports:
- `serve_stdio`: For local agents (Claude Desktop, IDEs)
- `serve_sse`: For remote/web agents (requires `sse` feature)

---

## Core Feature: Agent Runtime & Orchestration

Build autonomous agents and pipelines directly in Rust. The runtime handles the ReAct loop (Reasoning + Acting), tool execution, and context management.

```rust
use unistructgen_agent::{Agent, AgentPipeline};
use unistructgen_core::ToolRegistry;
use unistructgen_macro::ai_tool;
use std::sync::Arc;

// 1. Define Tools
#[ai_tool]
fn search_web(query: String) -> String { /* ... */ }

// 2. Build Agent
let researcher = Agent::builder()
    .name("Researcher")
    .client(llm_client)
    .tools(Arc::new(registry))
    .system_prompt("You are a researcher. Use tools to find info.")
    .build()?;

// 3. Run (Auto-loop: Thought -> Action -> Observation -> Thought)
let answer = researcher.run("What is the latest Rust version?").await?;
```

### Multi-Agent DAG Pipeline

Chain agents together to solve complex tasks.

```rust
let pipeline = AgentPipeline::builder()
    .agent("planner", planner_agent)
    .agent("coder", coder_agent)
    .agent("reviewer", reviewer_agent)
    .start("planner")
    .transition("planner", "coder")
    .transition("coder", "reviewer")
    .build()?;

let result = pipeline.run("Create a snake game").await?;
```

---

## All 6 Parsers

UniStructGen includes parsers for 6 input formats. Each implements the `Parser` trait and produces `IRModule`.

### 1. JSON

```rust
// Proc macro (compile-time)
generate_struct_from_json! {
    name = "User",
    json = r#"{"id": 1, "name": "Alice", "tags": ["admin"]}"#,
    serde = true
}

// Runtime pipeline
use unistructgen_json_parser::{JsonParser, ParserOptions};
let mut parser = JsonParser::new(ParserOptions {
    struct_name: "User".into(),
    derive_serde: true,
    ..Default::default()
});
let ir = parser.parse(json_str)?;
```

Smart type inference detects: DateTime, UUID, Email, URL patterns in string values.

### 2. OpenAPI / Swagger

```rust
// Proc macro
openapi_to_rust! {
    file = "api/openapi.yaml",
    generate_client = true,
    generate_validation = true
}

// Also supports: spec = "inline yaml...", url = "https://..."
```

Client generation is a typed scaffold (best-effort) and may require manual adjustments for edge cases.

### 3. SQL DDL

```rust
generate_struct_from_sql! {
    sql = r#"CREATE TABLE users (
        id SERIAL PRIMARY KEY,
        name VARCHAR(100) NOT NULL,
        email VARCHAR(255) UNIQUE,
        created_at TIMESTAMP DEFAULT NOW()
    );"#,
    serde = true
}
```

### 4. GraphQL Schema

```rust
generate_struct_from_graphql! {
    schema = r#"
        type User { id: ID!, name: String!, email: String, posts: [Post!]! }
        type Post { id: ID!, title: String!, body: String! }
    "#,
    serde = true
}
```

### 5. .env Files

```rust
generate_struct_from_env! {
    name = "AppConfig",
    env = r#"
        DATABASE_URL=postgres://localhost/mydb
        PORT=8080
        DEBUG=true
    "#
}
// Generates: AppConfig { database_url: String, port: i64, debug: bool }
```

### 6. Markdown Tables

```rust
// Runtime via MarkdownParser or CLI:
// unistructgen generate --input schema.md --name Config
```

The markdown parser also includes a **semantic chunker** for RAG pipelines:

```rust
use unistructgen_markdown_parser::chunker::SemanticChunker;

let markdown = std::fs::read_to_string("docs/README.md")?;
let chunks = SemanticChunker::chunk(&markdown);
// Each chunk preserves heading hierarchy and semantic boundaries
```

---

## Builder API

Build IR structs and enums programmatically with a fluent API, then generate Rust code or JSON Schema.

```rust
use unistructgen_core::{StructGen, EnumGen, ModuleGen, FieldType, FieldBuilder};

// Struct
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

// Enum
let code = EnumGen::new()
    .name("OrderStatus")
    .variant("Pending")
    .variant_with_rename("InTransit", "in_transit")
    .variant("Delivered")
    .with_serde()
    .generate()?;

// Module with multiple types
let code = ModuleGen::new("models")
    .add_struct(StructGen::new().name("User").field("id", FieldType::I64))
    .add_enum(EnumGen::new().name("Status").variant("Active"))
    .generate()?;
```

### Field constraints

Constraints generate `#[validate(...)]` attributes on the rendered Rust struct:

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

### Quick JSON parsing

```rust
use unistructgen_core::from_json;

let code = from_json(r#"{"id": 1, "name": "Alice"}"#)
    .struct_name("User")
    .with_serde()
    .generate()?;
```

---

## Pipeline API

Chain a parser, transformers, and generator into a processing pipeline:

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

### Built-in transformers

| Transformer | Effect |
|---|---|
| `FieldOptionalizer` | Wraps all fields in `Option<T>` |
| `DocCommentAdder` | Adds doc comments to structs/fields |
| `TypeDeduplicator` | Deduplicates identical nested struct definitions |
| `FieldRenamer` | Renames fields (e.g. snake_case conversion) |

### Plugin system

Plugins hook into the pipeline at parse and generate stages:

```rust
use unistructgen_core::{PluginRegistry, plugin::LoggingPlugin};

let mut plugins = PluginRegistry::new();
plugins.register(Box::new(LoggingPlugin::new(true)))?;

let input = plugins.before_parse(input)?;
let module = plugins.after_parse(module)?;
let code = plugins.after_generate(code)?;
```

---

## CLI

```bash
cargo install unistructgen

# Generate Rust structs from JSON
unistructgen generate --input data.json --name MyStruct --serde

# Generate from Markdown table
unistructgen generate --input schema.md --name Config

# Generate HTTP client scaffold from OpenAPI spec
unistructgen client --spec api.yaml --name GitHub --output ./generated

# AI-powered error fixing (experimental)
unistructgen fix
```

---

## Architecture

```
                    +--------------------------------------------------------------+
                    |                        UniStructGen                           |
                    |                                                               |
  +----------+     |  +--------+    +----+    +-------------+    +----------+      |
  |  JSON    |--+  |  |        |    |    |    |             |    |  Rust    |      |
  |  OpenAPI |--+  |  |        |    |    |    |             |    |  Code    |      |
  |  SQL     |--+  |  | Parser |--->| IR |--->| Transformer |--->|  JSON    |      |
  |  GraphQL |--+->|  |        |    |    |    |             |    |  Schema  |      |
  |  .env    |--+  |  |        |    |    |    |             |    |          |      |
  |  Markdown|--+  |  +--------+    +----+    +-------------+    +----------+      |
  +----------+     |  [Plugins]                [Plugins]                            |
                   +--------------------------------------------------------------+
                                        |
                            +-----------+-----------+
                            v           v           v
                     +------------+ +--------+ +----------+ +--------+
                     | #[ai_tool] | |  LLM   | |Validation| |  MCP   |
                     | ToolRegist | | Client | |  Loop    | | Server |
                     | JSON Schema| |OpenAI  | | Reports  | | stdio/ |
                     +------------+ |Ollama  | | Prompts  | |  sse   |
                                    +--------+ +----------+ +--------+
```

### Core traits

| Trait | Module | Purpose | Implementations |
|---|---|---|---|
| `Parser` | `core::parser` | Input format to IR | `JsonParser`, `OpenApiParser`, `MarkdownParser`, `SqlParser`, `GraphqlParser`, `EnvParser` |
| `CodeGenerator` | `core::codegen` | IR to output code | `RustRenderer`, `JsonSchemaRenderer` |
| `IRTransformer` | `core::transformer` | Transform IR in-flight | `FieldOptionalizer`, `DocCommentAdder`, `TypeDeduplicator`, `FieldRenamer` |
| `Plugin` | `core::plugin` | Pipeline hooks | `LoggingPlugin`, `HeaderPlugin`, custom |
| `AiTool` | `core::tools` | LLM tool interface | Auto-generated by `#[ai_tool]` |
| `LlmClient` | `llm` | LLM provider abstraction | `OpenAiClient`, `OllamaClient` |
| `AiValidatable` | `core::validation` | Self-validation for AI | Custom types |
| `IRVisitor` | `core::visitor` | IR traversal/analysis | `StructNameCollector`, `FieldCounter`, `IRValidator` |

### IR type system

```rust
// core::ir -- the shared representation all parsers emit and all generators consume

IRModule { name: String, types: Vec<IRType> }
IRType::Struct(IRStruct) | IRType::Enum(IREnum)

IRStruct { name, fields: Vec<IRField>, derives, doc, attributes }
IRField  { name, source_name, ty: IRTypeRef, optional, default, constraints, attributes, doc }

IRTypeRef::Primitive(PrimitiveKind)  // String, I32, I64, F64, Bool, DateTime, Uuid, etc.
IRTypeRef::Option(Box<IRTypeRef>)    // Option<T>
IRTypeRef::Vec(Box<IRTypeRef>)       // Vec<T>
IRTypeRef::Named(String)             // Reference to another struct/enum
IRTypeRef::Map(Box, Box)             // HashMap<K, V>

FieldConstraints { min_length, max_length, min_value, max_value, pattern, format }
```

---

## Crate Map

```
unistructgen/
├── core/                    # unistructgen-core
│   └── src/
│       ├── lib.rs           # Re-exports all public API
│       ├── ir.rs            # IRModule, IRStruct, IRField, IRTypeRef, PrimitiveKind
│       ├── api.rs           # StructGen, EnumGen, ModuleGen, FieldBuilder, FieldType
│       ├── parser.rs        # Parser trait, ParserExt
│       ├── codegen.rs       # CodeGenerator trait, MultiGenerator
│       ├── transformer.rs   # IRTransformer trait + 4 built-in transformers
│       ├── pipeline.rs      # Pipeline, PipelineBuilder
│       ├── plugin.rs        # Plugin trait, PluginRegistry
│       ├── visitor.rs       # IRVisitor trait, walk_* functions
│       ├── tools.rs         # AiTool trait, ToolRegistry, ToolCall
│       ├── context.rs       # Context (type-safe dependency injection)
│       ├── validation.rs    # AiValidationError, ValidationReport, map_serde_error
│       ├── diagnostics.rs   # CargoDiagnostics, CompilerError
│       ├── patch.rs         # CodeFix, Hunk (LLM code patching)
│       └── error.rs         # Error types
│
├── codegen/                 # unistructgen-codegen
│   └── src/
│       ├── lib.rs           # RustRenderer, RenderOptions
│       ├── json_schema.rs   # JsonSchemaRenderer (Draft 2020-12)
│       └── builder.rs       # RustRendererBuilder
│
├── parsers/
│   ├── json_parser/         # unistructgen-json-parser
│   ├── openapi_parser/      # unistructgen-openapi-parser
│   ├── markdown_parser/     # unistructgen-markdown-parser (+ SemanticChunker)
│   ├── sql_parser/          # unistructgen-sql-parser
│   ├── graphql_parser/      # unistructgen-graphql-parser
│   └── env_parser/          # unistructgen-env-parser
│
├── proc-macro/              # unistructgen-macro
│   └── src/
│       ├── lib.rs           # 8 macros: generate_struct_from_json!, #[json_struct],
│       │                    #   struct_from_external_api!, openapi_to_rust!,
│       │                    #   generate_struct_from_sql!, generate_struct_from_graphql!,
│       │                    #   generate_struct_from_env!, #[ai_tool]
│       └── ai_tool.rs       # ai_tool macro implementation
│
├── llm/                     # unistructgen-llm
│   └── src/
│       ├── lib.rs           # LlmClient trait, CompletionRequest, Message
│       ├── openai.rs        # OpenAiClient
│       ├── ollama.rs        # OllamaClient
│       └── factory.rs       # LlmClientFactory, Provider enum
│
├── mcp/                     # unistructgen-mcp
│   └── src/
│       ├── lib.rs           # MCP Server exports (serve_stdio, serve_sse)
│       ├── protocol.rs      # JSON-RPC & MCP types
│       ├── server.rs        # Core MCP logic
│       ├── stdio.rs         # Stdio transport
│       └── sse.rs           # SSE transport (optional)
│
├── agent/                   # unistructgen-agent
│   └── src/
│       ├── lib.rs           # Agent & Pipeline exports
│       ├── agent.rs         # ReAct loop implementation
│       └── pipeline.rs      # DAG orchestration
│
├── cli/                     # unistructgen
│   └── src/
│       ├── main.rs          # generate, client, fix commands
│       └── commands/        # Command implementations
│
└── examples/
    ├── tools-agent/         # AI tool registry + batch execution demo
    ├── docu-agent/          # RAG ingestion + JSON Schema + validation loop
    ├── code-agent/          # Compiler-driven AI coding loop
    ├── github-client/       # GitHub API client from OpenAPI
    ├── blog-api/            # Blog API types from OpenAPI
    ├── api-example/         # Struct generation from live API
    └── proc-macro-example/  # All proc macros demonstrated
```

---

## Type Mapping Reference

How IR types map across parsers and generators:

| IR Type | Rust Output | JSON Schema Output | Source: JSON | Source: SQL | Source: GraphQL |
|---|---|---|---|---|---|
| `String` | `String` | `"string"` | string values | `VARCHAR`, `TEXT` | `String`, `ID` |
| `I32` | `i32` | `"integer"` | small ints | `INT`, `INTEGER` | `Int` |
| `I64` | `i64` | `"integer"` | large ints | `BIGINT`, `SERIAL` | -- |
| `F64` | `f64` | `"number"` | floats | `DOUBLE`, `REAL` | `Float` |
| `Bool` | `bool` | `"boolean"` | booleans | `BOOLEAN` | `Boolean` |
| `DateTime` | `chrono::DateTime<Utc>` | `"string" format:"date-time"` | ISO 8601 strings | `TIMESTAMP` | -- |
| `Uuid` | `uuid::Uuid` | `"string" format:"uuid"` | UUID strings | `UUID` | -- |
| `Decimal` | `rust_decimal::Decimal` | `"number"` | -- | `DECIMAL`, `NUMERIC` | -- |
| `Option(T)` | `Option<T>` | omitted from `required` | -- | nullable columns | nullable fields |
| `Vec(T)` | `Vec<T>` | `"array"` | arrays | -- | `[Type]` |
| `Map(K,V)` | `HashMap<K,V>` | `"object" + additionalProperties` | dynamic objects | -- | -- |
| `Named(S)` | `S` | `"$ref": "#/$defs/S"` | nested objects | -- | type references |

---

## Examples

| Example | What It Demonstrates |
|---|---|
| `tools-agent` | Register functions as AI tools, batch execution, dependency injection via Context, LlmClientFactory |
| `docu-agent` | RAG ingestion with SemanticChunker, JSON Schema contract, AI validation loop with auto-correction |
| `code-agent` | Compiler-driven development: AI writes code, CargoDiagnostics checks, errors fed back, AI fixes iteratively |
| `github-client` | GitHub API client scaffold generated from OpenAPI spec |
| `blog-api` | Blog API types from OpenAPI |
| `api-example` | Struct generation from live API responses with `struct_from_external_api!` |
| `proc-macro-example` | All proc macros: JSON, OpenAPI, SQL, GraphQL, .env |
| `killer-example` | Types + LLM tool schema + safe execution in one file |

## Blog

- `docs/blog/announcing-unistructgen.md`

---

## Development

```bash
# Check all workspace crates
cargo check --all

# Run all tests
cargo test --all

# Run tests for a specific crate
cargo test -p unistructgen-core

# Build release
cargo build --release

# Run CLI in dev
cargo run -p unistructgen -- generate --input data.json --name MyStruct
```

---

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
