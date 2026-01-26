# Architecture

Technical reference for the UniStructGen codebase. Covers every layer: IR design, pipeline, parsers, code generators, plugin system, AI tooling, LLM integration, validation, and agent infrastructure.

---

## Table of Contents

- [Design Philosophy](#design-philosophy)
- [System Overview](#system-overview)
- [Intermediate Representation (IR)](#intermediate-representation-ir)
- [Data Flow](#data-flow)
- [Parsers](#parsers)
- [Transformers](#transformers)
- [Code Generators](#code-generators)
- [Pipeline](#pipeline)
- [Plugin System](#plugin-system)
- [Visitor Pattern](#visitor-pattern)
- [Proc Macros](#proc-macros)
- [AI Tool System](#ai-tool-system)
- [JSON Schema Generator](#json-schema-generator)
- [LLM Client Abstraction](#llm-client-abstraction)
- [AI Validation System](#ai-validation-system)
- [Agent Infrastructure](#agent-infrastructure)
- [Error Handling Strategy](#error-handling-strategy)
- [Extensibility Guide](#extensibility-guide)
- [Workspace & Dependency Graph](#workspace--dependency-graph)

---

## Design Philosophy

UniStructGen is built on four principles:

1. **IR-centric**: Every input format is first parsed into a language-agnostic Intermediate Representation. Every output format is generated from that IR. The IR is the single source of truth.

2. **Trait-driven extensibility**: Every processing stage is defined by a trait (`Parser`, `CodeGenerator`, `IRTransformer`, `Plugin`, `AiTool`, `LlmClient`). Adding a new parser, generator, or tool means implementing a trait -- nothing else changes.

3. **Compile-time first**: Proc macros generate structs at compile time with zero runtime cost. The pipeline API exists for runtime use cases, but the default path is compile-time.

4. **AI-native**: The IR isn't just for code generation -- it generates JSON Schema for structured LLM outputs, powers the `#[ai_tool]` macro for function calling, and feeds validation loops for self-healing AI responses.

---

## System Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              UniStructGen                                   │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                         PROCESSING PIPELINE                         │    │
│  │                                                                     │    │
│  │  ┌──────────┐  ┌────────┐  ┌────┐  ┌─────────────┐  ┌──────────┐  │    │
│  │  │ Plugin   │  │        │  │    │  │             │  │ Plugin   │  │    │
│  │  │ before_  │─▶│ Parser │─▶│ IR │─▶│ Transformer │─▶│ after_   │  │    │
│  │  │ parse    │  │        │  │    │  │   chain     │  │ generate │  │    │
│  │  └──────────┘  └────────┘  └─┬──┘  └─────────────┘  └────┬─────┘  │    │
│  │                              │                            │        │    │
│  └──────────────────────────────┼────────────────────────────┼────────┘    │
│                                 │                            │             │
│              ┌──────────────────┼────────────────────────────┼──────┐      │
│              │                  ▼                            ▼      │      │
│              │  ┌──────────────────────┐  ┌──────────────────────┐  │      │
│              │  │    RustRenderer      │  │  JsonSchemaRenderer  │  │      │
│              │  │    (Rust code)       │  │  (Draft 2020-12)     │  │      │
│              │  └──────────────────────┘  └──────────┬───────────┘  │      │
│              │          CODE GENERATORS              │              │      │
│              └───────────────────────────────────────┼──────────────┘      │
│                                                      │                     │
│  ┌───────────────────────────────────────────────────┼──────────────────┐  │
│  │                       AI LAYER                    │                  │  │
│  │                                                   ▼                  │  │
│  │  ┌─────────────┐  ┌──────────────┐  ┌─────────────────────────┐     │  │
│  │  │ #[ai_tool]  │  │  LLM Client  │  │   Validation System     │     │  │
│  │  │ ToolRegistry│  │  OpenAI      │  │   ValidationReport      │     │  │
│  │  │ AiTool trait│  │  Ollama      │◀─│   to_correction_prompt() │     │  │
│  │  │ JSON Schema │  │  LlmClient   │  │   map_serde_error()     │     │  │
│  │  └─────────────┘  └──────────────┘  └─────────────────────────┘     │  │
│  │                                                                      │  │
│  │  ┌──────────────────────────────────────────────────────────────┐    │  │
│  │  │                  AGENT INFRASTRUCTURE                        │    │  │
│  │  │  RustSandbox · Compiler · SemanticChunker · CodeExtractor   │    │  │
│  │  └──────────────────────────────────────────────────────────────┘    │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
│  ┌──────────────────────────────────────────────────────────────────────┐   │
│  │                        PROC MACROS (compile-time)                    │   │
│  │  generate_struct_from_json!  ·  openapi_to_rust!  ·  #[ai_tool]     │   │
│  │  struct_from_external_api!   ·  generate_struct_from_sql!           │   │
│  │  generate_struct_from_graphql!  ·  generate_struct_from_env!        │   │
│  │  #[json_struct]                                                      │   │
│  └──────────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Input parsers** (left side):
```
JSON ──────────┐
OpenAPI/Swagger─┤
SQL DDL ────────┤
GraphQL Schema──┼──▶ Parser trait ──▶ IR
.env ───────────┤
Markdown Tables─┘
```

**Output generators** (right side):
```
IR ──▶ CodeGenerator trait ──┬──▶ Rust code (.rs)
                             └──▶ JSON Schema (.json)
```

---

## Intermediate Representation (IR)

The IR is the core data model. Every parser produces it, every generator consumes it. It lives in `core/src/ir.rs`.

### Type Hierarchy

```
IRModule
│   name: String
│   types: Vec<IRType>
│
├── IRType::Struct(IRStruct)
│   │   name: String
│   │   fields: Vec<IRField>
│   │   derives: Vec<String>
│   │   doc: Option<String>
│   │   attributes: Vec<String>
│   │
│   └── IRField
│           name: String
│           source_name: Option<String>      ← original name (for serde rename)
│           ty: IRTypeRef
│           optional: bool
│           default: Option<String>
│           constraints: FieldConstraints
│           attributes: Vec<String>
│           doc: Option<String>
│
└── IRType::Enum(IREnum)
    │   name: String
    │   variants: Vec<IREnumVariant>
    │   derives: Vec<String>
    │   doc: Option<String>
    │
    └── IREnumVariant
            name: String
            source_value: Option<String>     ← original value (for serde rename)
            doc: Option<String>
```

### Type References

`IRTypeRef` represents the type of a field. It is recursive:

```rust
enum IRTypeRef {
    Primitive(PrimitiveKind),           // String, i32, f64, bool, etc.
    Option(Box<IRTypeRef>),             // Option<T>
    Vec(Box<IRTypeRef>),                // Vec<T>
    Named(String),                      // Reference to another struct/enum by name
    Map(Box<IRTypeRef>, Box<IRTypeRef>),// HashMap<K, V>
}
```

### Primitive Types

`PrimitiveKind` covers all base types with their Rust mappings:

| PrimitiveKind | Rust Type | JSON Schema |
|---|---|---|
| `String` | `String` | `"string"` |
| `I8`, `I16`, `I32`, `I64`, `I128` | `i8`..`i128` | `"integer"` |
| `U8`, `U16`, `U32`, `U64`, `U128` | `u8`..`u128` | `"integer"` |
| `F32`, `F64` | `f32`, `f64` | `"number"` |
| `Bool` | `bool` | `"boolean"` |
| `Char` | `char` | `"string" format:"char"` |
| `DateTime` | `chrono::DateTime<Utc>` | `"string" format:"date-time"` |
| `Uuid` | `uuid::Uuid` | `"string" format:"uuid"` |
| `Decimal` | `rust_decimal::Decimal` | `"number"` |
| `Json` | `serde_json::Value` | `"object"` |

### Field Constraints

`FieldConstraints` holds validation rules that generators translate into `#[validate(...)]` attributes or JSON Schema keywords:

```rust
struct FieldConstraints {
    min_length: Option<usize>,    // validate(length(min = N))     / minLength
    max_length: Option<usize>,    // validate(length(max = N))     / maxLength
    min_value: Option<f64>,       // validate(range(min = N))      / minimum
    max_value: Option<f64>,       // validate(range(max = N))      / maximum
    pattern: Option<String>,      // validate(regex = "...")        / pattern
    format: Option<String>,       // validate(email) / validate(url)/ format
}
```

### Why This IR Design

- **No language-specific types**: `PrimitiveKind::I64` maps to `i64` in Rust, `"integer"` in JSON Schema, `BIGINT` in SQL. The IR is the neutral ground.
- **Source names preserved**: `source_name` / `source_value` track the original JSON key or enum value. Generators use this to emit `#[serde(rename = "...")]`.
- **Constraints are separate from types**: A `String` field with `min_length: 5` is still a `String` in IR. The constraint is metadata that generators can choose to use or ignore.
- **Recursive type refs**: `Option<Vec<HashMap<String, User>>>` is representable as nested `IRTypeRef` values. No limit on depth.

---

## Data Flow

### Runtime Pipeline

```
Input string
    │
    ▼
Plugin.before_parse(input)         ← modify raw input
    │
    ▼
Parser.parse(input) → IRModule     ← string to IR
    │
    ▼
Plugin.after_parse(module)         ← modify IR
    │
    ▼
Transformer[0].transform(module)   ← FieldOptionalizer, DocCommentAdder, etc.
Transformer[1].transform(module)
    ...
    │
    ▼
CodeGenerator.generate(module)     ← IR to code string
    │
    ▼
Plugin.after_generate(code)        ← modify output (add headers, format, etc.)
    │
    ▼
Output string (Rust code / JSON Schema)
```

### Compile-Time (Proc Macro)

```
Macro invocation
    │
    ├─▶ [struct_from_external_api!] HTTP fetch → JSON string
    │
    ▼
Parser.parse(input) → IRModule
    │
    ▼
RustRenderer.render(module) → String
    │
    ▼
string.parse::<TokenStream>() → compiled Rust code
```

No pipeline, no transformers, no plugins. Proc macros take the shortest path: parse → render → TokenStream.

### AI Tool Flow

```
#[ai_tool] on function
    │
    ▼
Extract function signature (syn)
    │
    ▼
Map arguments to IRField + IRTypeRef       ← reuses core IR types
    │
    ▼
Build IRStruct from arguments
    │
    ▼
JsonSchemaRenderer.generate(module)         ← reuses codegen module
    │
    ▼
Generate tool struct + AiTool impl
    │
    ▼
TokenStream output:
  - Original function preserved
  - {Name}Tool struct
  - {Name}Args struct (serde::Deserialize)
  - AiTool trait impl with name, description, parameters_schema, call
```

---

## Parsers

### Parser Trait

Defined in `core/src/parser.rs`:

```rust
pub trait Parser {
    type Error: std::error::Error + Send + Sync + 'static;

    fn parse(&mut self, input: &str) -> Result<IRModule, Self::Error>;
    fn name(&self) -> &'static str;
    fn extensions(&self) -> &[&'static str];

    // Optional
    fn validate(&self, _input: &str) -> Result<(), Self::Error> { Ok(()) }
    fn metadata(&self) -> ParserMetadata { ParserMetadata::new() }
}
```

`ParserExt` adds convenience methods: `parse_validated()`, `parse_with_metadata()`.

### Parser Implementations

#### JsonParser (`parsers/json_parser/`, 1393 lines)

**Input**: JSON string.
**Output**: `IRModule` with one root struct + nested structs for objects.

Key internals:
- **Smart type inference** via `TypeInferenceStrategy` trait with pluggable detectors:
  - `DateTimeDetector` -- ISO 8601 patterns
  - `UuidDetector` -- 8-4-4-4-12 hex format
  - `EmailDetector` -- `@` with domain
  - `UrlDetector` -- `http://` / `https://` prefix
- **Nested object handling**: JSON objects inside objects produce separate `IRStruct` entries with `IRTypeRef::Named(name)` references.
- **Array type inference**: Uses the first element of arrays to determine `Vec<T>`.
- **Field name sanitization**: `kebab-case` → `snake_case`, Rust keyword avoidance (`type` → `type_field`).
- **Builder**: `JsonParser::builder().struct_name("User").derive_serde().build()`

```rust
JsonParser::new(ParserOptions {
    struct_name: "User".into(),
    derive_serde: true,
    derive_default: false,
    make_fields_optional: false,
})
```

#### OpenApiParser (`parsers/openapi_parser/`, 1707 lines)

**Input**: OpenAPI 3.0/3.1 spec (YAML or JSON).
**Output**: `IRModule` with structs for schemas + enums for string enums + optional client types.

Key internals:
- **`SchemaConverter`** resolves `$ref` references with cycle detection (reference stack).
- **Schema composition**: handles `allOf` (merge fields), `oneOf`/`anyOf` (enum generation).
- **Constraint extraction**: `minLength`, `maxLength`, `minimum`, `maximum`, `pattern`, `format` → `FieldConstraints`.
- **`ClientGenerator`** produces request/response types from `paths` and `operations`.
- **Depth limiting** prevents infinite recursion on deeply nested specs.
- **Full options builder** with 13 configuration parameters.

```rust
OpenApiParser::new(
    OpenApiParserOptions::builder()
        .generate_client(true)
        .generate_validation(true)
        .max_depth(10)
        .build()
)
```

#### SqlParser (`parsers/sql_parser/`, 199 lines)

**Input**: SQL `CREATE TABLE` DDL statements.
**Output**: `IRModule` with one struct per table.

Type mapping:
```
INTEGER, INT, SMALLINT, TINYINT  → I32
BIGINT, SERIAL                    → I64
FLOAT, DOUBLE, REAL               → F64
DECIMAL, NUMERIC                  → Decimal
BOOLEAN, BOOL                     → Bool
VARCHAR, TEXT, CHAR, CLOB         → String
TIMESTAMP, DATETIME, DATE, TIME   → DateTime
UUID                              → Uuid
JSON, JSONB                       → Json
```

`NOT NULL` handling: fields without `NOT NULL` become `optional: true`.

#### GraphqlParser (`parsers/graphql_parser/`, 211 lines)

**Input**: GraphQL schema definition language.
**Output**: `IRModule` with structs for `type` and `input` definitions.

- Non-null (`!`) fields → `optional: false`
- List types (`[T]`) → `Vec<T>`
- `ID` type → `String`
- `Int` → `I32`, `Float` → `F64`, `Boolean` → `Bool`

#### MarkdownParser (`parsers/markdown_parser/`, 317 lines)

**Input**: Markdown document with tables.
**Output**: `IRModule` with structs from table definitions.

Detects columns by header name: `Name`/`Field`, `Type`, `Description`, `Required`/`Optional`.

#### EnvParser (`parsers/env_parser/`, 192 lines)

**Input**: `.env` file format (`KEY=value`).
**Output**: `IRModule` with one struct, fields from keys.

Type inference from values: numbers → `I64`/`F64`, `true`/`false` → `Bool`, everything else → `String`.

---

## Transformers

### IRTransformer Trait

Defined in `core/src/transformer.rs`:

```rust
pub trait IRTransformer {
    fn transform(&self, module: IRModule) -> Result<IRModule, TransformError>;
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str { "" }
}
```

Transformers are pure functions on IR: `IRModule → IRModule`. They don't know about parsers or generators.

### Built-in Transformers

| Transformer | What It Does |
|---|---|
| `FieldOptionalizer` | Wraps every field's type in `Option<T>` and sets `optional: true` |
| `DocCommentAdder` | Generates doc comments from field/struct names |
| `TypeDeduplicator` | Removes duplicate `IRStruct` definitions (by name) |
| `FieldRenamer` | Renames fields based on a `HashMap<String, String>` mapping |

Transformers are applied in order. The pipeline executes them sequentially:

```rust
Pipeline::new(parser, generator)
    .add_transformer(Box::new(FieldOptionalizer::new()))
    .add_transformer(Box::new(DocCommentAdder::new()))
    .add_transformer(Box::new(FieldRenamer::new(renames)))
```

### TransformError

```rust
enum TransformError {
    Transform { transformer: String, message: String },
    InvalidIR { message: String },
    Custom(Box<dyn Error + Send + Sync>),
}
```

---

## Code Generators

### CodeGenerator Trait

Defined in `core/src/codegen.rs`:

```rust
pub trait CodeGenerator {
    type Error: std::error::Error + Send + Sync + 'static;

    fn generate(&self, module: &IRModule) -> Result<String, Self::Error>;
    fn language(&self) -> &'static str;
    fn file_extension(&self) -> &str;

    // Optional
    fn validate(&self, module: &IRModule) -> Result<(), Self::Error> { Ok(()) }
    fn format(&self, code: String) -> Result<String, Self::Error> { Ok(code) }
    fn metadata(&self) -> GeneratorMetadata { GeneratorMetadata::new() }
}
```

`CodeGeneratorExt` adds: `generate_formatted()`, `generate_validated()`, `generate_complete()` (validate + generate + format).

`MultiGenerator` chains multiple generators and collects all outputs:

```rust
let multi = MultiGenerator::new()
    .add("rust", Box::new(RustRenderer::new(opts)))
    .add("schema", Box::new(JsonSchemaRenderer::new()));

let outputs: HashMap<String, String> = multi.generate_all(&module)?;
```

### RustRenderer (`codegen/src/lib.rs`, 541 lines)

Converts IR to idiomatic Rust code.

Generated output structure:
```rust
// Generated by unistructgen v0.1.0       ← header (optional)
// Do not edit this file manually

#![allow(dead_code)]                       ← clippy allows (optional)
#![allow(unused_imports)]

/// Doc comment from IR                    ← IRStruct.doc
#[derive(Debug, Clone, PartialEq)]        ← IRStruct.derives
#[custom_attribute]                        ← IRStruct.attributes
pub struct User {
    /// Field doc                          ← IRField.doc
    #[serde(rename = "user_name")]         ← IRField.attributes (from source_name)
    #[validate(length(min = 1, max = 100))]← generated from FieldConstraints
    pub name: String,                      ← IRField.name : render_type(IRField.ty)
    pub email: Option<String>,             ← optional field
    pub tags: Vec<String>,                 ← Vec type
    pub address: Address,                  ← Named type reference
}
```

Validation attribute generation from `FieldConstraints`:
- `min_length`/`max_length` → `validate(length(min = N, max = N))`
- `min_value`/`max_value` → `validate(range(min = N, max = N))`
- `pattern` → `validate(regex = "...")`
- `format: "email"` → `validate(email)`
- `format: "url"` → `validate(url)`

Builder: `RustRenderer::builder().add_header().add_clippy_allows().build()`

### JsonSchemaRenderer (`codegen/src/json_schema.rs`, 236 lines)

Converts IR to JSON Schema Draft 2020-12.

```rust
let renderer = JsonSchemaRenderer::new();          // full schema with $schema
let renderer = JsonSchemaRenderer::new().fragment();// no $schema (for embedding)
```

Output structure:
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$defs": {
    "User": {
      "type": "object",
      "properties": {
        "name": { "type": "string" },
        "age": { "type": "integer" },
        "address": { "$ref": "#/$defs/Address" }
      },
      "required": ["name", "age", "address"],
      "additionalProperties": false
    },
    "Address": { ... }
  },
  "$ref": "#/$defs/User"
}
```

Key behaviors:
- All struct types go into `$defs` with `$ref` cross-references.
- `IRTypeRef::Named("Foo")` → `{ "$ref": "#/$defs/Foo" }`
- `IRTypeRef::Option(inner)` → schema of `inner` (optionality via omission from `required`)
- `IRTypeRef::Vec(inner)` → `{ "type": "array", "items": <inner_schema> }`
- `IRTypeRef::Map(k, v)` → `{ "type": "object", "additionalProperties": <value_schema> }`
- `IREnum` → `{ "type": "string", "enum": ["Active", "Inactive"] }` (uses `source_value` if present)
- **Strict mode**: `additionalProperties: false` on all objects (required by OpenAI structured outputs).
- **Root detection**: Last type in module or type matching module name.

---

## Pipeline

Defined in `core/src/pipeline.rs`. Chains a parser, transformers, and generator into a single executable unit.

```rust
pub struct Pipeline<P: Parser, G: CodeGenerator> {
    parser: P,
    generator: G,
    transformers: Vec<Box<dyn IRTransformer>>,
}
```

### Execution Sequence

```rust
pub fn execute(&mut self, input: &str) -> Result<String, PipelineError> {
    // 1. Parse
    let mut ir = self.parser.parse(input)?;

    // 2. Transform (in order)
    for transformer in &self.transformers {
        ir = transformer.transform(ir)?;
    }

    // 3. Generate
    let code = self.generator.generate(&ir)?;
    Ok(code)
}
```

### PipelineBuilder

```rust
let pipeline = PipelineBuilder::new()
    .parser(JsonParser::new(opts))
    .generator(RustRenderer::new(render_opts))
    .transformer(Box::new(FieldOptionalizer::new()))
    .transformer(Box::new(DocCommentAdder::new()))
    .build();
```

### PipelineError

```rust
enum PipelineError {
    Parse(Box<dyn Error>),
    Transform { transformer: String, source: TransformError },
    Generate(Box<dyn Error>),
    Plugin { plugin: String, message: String },
}
```

---

## Plugin System

Defined in `core/src/plugin.rs` (569 lines). Plugins hook into the pipeline at three points.

### Plugin Trait

```rust
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn description(&self) -> Option<&str> { None }

    // Lifecycle
    fn initialize(&mut self) -> Result<(), PluginError>;
    fn shutdown(&mut self) -> Result<(), PluginError>;

    // Hooks
    fn before_parse(&mut self, input: &str) -> Result<String, PluginError>;
    fn after_parse(&mut self, module: IRModule) -> Result<IRModule, PluginError>;
    fn after_generate(&mut self, code: String) -> Result<String, PluginError>;
}
```

### PluginRegistry

```rust
let mut registry = PluginRegistry::new();
registry.register(Box::new(LoggingPlugin::new(true)))?;   // initialize called
registry.register(Box::new(HeaderPlugin::new("// MIT")))?;

// Execute hooks on all plugins (in registration order)
let input = registry.before_parse(input)?;
let module = registry.after_parse(module)?;
let code = registry.after_generate(code)?;

// Cleanup
registry.shutdown()?;  // also called on drop
```

Duplicate plugin names are rejected. Plugins are initialized on registration and shut down on removal or drop.

### Built-in Plugins

| Plugin | Hook | Behavior |
|---|---|---|
| `LoggingPlugin` | all | Prints processing stages when `verbose: true` |
| `HeaderPlugin` | `after_generate` | Prepends a header comment to generated code |

---

## Visitor Pattern

Defined in `core/src/visitor.rs` (496 lines). Read-only (or mutable) traversal of the IR without modifying the traversal logic.

### IRVisitor Trait

```rust
pub trait IRVisitor {
    fn visit_module(&mut self, module: &mut IRModule)     { walk_module(self, module); }
    fn visit_type(&mut self, ty: &mut IRType)             { walk_type(self, ty); }
    fn visit_struct(&mut self, s: &mut IRStruct)          { walk_struct(self, s); }
    fn visit_enum(&mut self, e: &mut IREnum)              { walk_enum(self, e); }
    fn visit_field(&mut self, field: &mut IRField)        { walk_field(self, field); }
    fn visit_type_ref(&mut self, ty: &mut IRTypeRef)      { walk_type_ref(self, ty); }
}
```

Walk functions handle recursive descent. Override `visit_*` methods to collect data or mutate nodes.

### Built-in Visitors

| Visitor | Purpose |
|---|---|
| `StructNameCollector` | Collects all struct names into `Vec<String>` |
| `FieldCounter` | Counts total fields across all structs |
| `PrimitiveTypeCollector` | Collects all `PrimitiveKind` values into `HashSet` |
| `IRValidator` | Validates IR integrity (empty names, structs with no fields) |
| `FieldPublicizer` | Placeholder for future visibility mutation |

---

## Proc Macros

All proc macros live in `proc-macro/src/lib.rs` (1348 lines). Each macro follows the same pattern: parse input → create parser → parse to IR → render with `RustRenderer` → return `TokenStream`.

### Macro Inventory

| Macro | Type | Input Source |
|---|---|---|
| `generate_struct_from_json!` | function-like | inline JSON string |
| `#[json_struct]` | attribute | const string value |
| `struct_from_external_api!` | function-like | HTTP URL (fetched at compile time) |
| `openapi_to_rust!` | function-like | file path, URL, or inline spec |
| `generate_struct_from_sql!` | function-like | inline SQL DDL |
| `generate_struct_from_graphql!` | function-like | inline GraphQL schema |
| `generate_struct_from_env!` | function-like | inline .env string |
| `#[ai_tool]` | attribute | function definition |

### struct_from_external_api! Internal Flow

```
1. Parse macro input (url, method, auth, options)
2. Build HTTP request with ureq
   - Apply auth: Bearer / ApiKey / Basic
   - Set timeout
3. Execute HTTP request at compile time
4. Parse response as JSON
5. Handle arrays (extract first element for inference)
6. Apply max_entity_count (truncate arrays)
7. Apply max_depth (replace deep values with null)
8. Feed processed JSON to JsonParser
9. Render IR with RustRenderer (no header, no clippy)
10. Parse string back to TokenStream
```

### openapi_to_rust! Sources

Accepts three source types (mutually exclusive):
- `spec = "..."` -- inline YAML/JSON specification
- `url = "..."` -- fetch from URL (with optional auth)
- `file = "..."` -- read from file (tries absolute path, then `CARGO_MANIFEST_DIR` relative)

---

## AI Tool System

### #[ai_tool] Macro (`proc-macro/src/ai_tool.rs`, 165 lines)

Transforms a regular Rust function into an LLM-callable tool.

**Input:**
```rust
/// Calculate shipping cost
#[ai_tool]
fn calculate_shipping(weight_kg: f64, destination: String) -> f64 {
    weight_kg * 2.5
}
```

**Generated output:**
```rust
// Original function preserved
fn calculate_shipping(weight_kg: f64, destination: String) -> f64 {
    weight_kg * 2.5
}

// Tool struct (PascalCase of function name + "Tool")
pub struct CalculateShippingTool;

// Arguments struct for deserialization
#[derive(serde::Deserialize)]
struct calculate_shippingArgs {
    pub weight_kg: f64,
    pub destination: String,
}

// AiTool trait implementation
impl unistructgen_core::AiTool for CalculateShippingTool {
    fn name(&self) -> &str { "calculate_shipping" }
    fn description(&self) -> &str { "Calculate shipping cost" }  // from doc comment
    fn parameters_schema(&self) -> serde_json::Value {
        // JSON Schema generated by JsonSchemaRenderer from IR
        serde_json::from_str(r#"{"type":"object","properties":{"weight_kg":{"type":"number"},...}}"#).unwrap()
    }
    fn call(&self, arguments_json: &str) -> ToolResult {
        let args: calculate_shippingArgs = serde_json::from_str(arguments_json)?;
        let result = calculate_shipping(args.weight_kg, args.destination);
        Ok(format!("{:?}", result))
    }
}
```

**Key implementation details:**
- Description extracted from `///` doc comments on the function.
- Argument types mapped via `map_syn_type_to_ir()`: `f64` → `PrimitiveKind::F64`, `String` → `PrimitiveKind::String`, `Vec<T>` → `IRTypeRef::Vec(...)`, `Option<T>` → `IRTypeRef::Option(...)`.
- JSON Schema generated by building an `IRModule` from the arguments, then calling `JsonSchemaRenderer::new().fragment().generate()`.
- Function name → PascalCase for struct name via `to_pascal_case()`.

### AiTool Trait (`core/src/tools.rs`)

```rust
pub trait AiTool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters_schema(&self) -> serde_json::Value;
    fn call(&self, arguments_json: &str) -> ToolResult;
}
```

### ToolRegistry (`core/src/tools.rs`)

```rust
pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn AiTool>>,
}
```

Methods:
- `register(tool)` -- adds tool by name
- `get_definitions() -> Vec<Value>` -- returns OpenAI-compatible function definitions
- `execute(name, args_json) -> ToolResult` -- dispatches to tool's `call()`
- `has_tool(name) -> bool` -- checks existence

Output format of `get_definitions()`:
```json
[{
    "type": "function",
    "function": {
        "name": "calculate_shipping",
        "description": "Calculate shipping cost",
        "parameters": { ... JSON Schema ... }
    }
}]
```

---

## JSON Schema Generator

`codegen/src/json_schema.rs` (236 lines). Implements `CodeGenerator` trait.

### Generation Algorithm

```
1. Iterate all types in IRModule
2. For each IRType::Struct:
   - Create object schema with properties, required, additionalProperties: false
   - Add to $defs
3. For each IRType::Enum:
   - Create string enum schema
   - Add to $defs
4. Determine root type (last type or module name match)
5. Build top-level schema:
   - $schema (unless fragment mode)
   - $defs with all type schemas
   - $ref pointing to root type
6. Serialize to JSON string
```

### Fragment Mode

`JsonSchemaRenderer::new().fragment()` omits `$schema` key. Used when the schema will be embedded inside a larger JSON object (e.g., OpenAI's `response_format.json_schema.schema`).

### Cross-References

Nested types use `$ref` instead of inline definitions:

```rust
// IR: field "address" with type Named("Address")
// JSON Schema output:
"address": { "$ref": "#/$defs/Address" }
```

This ensures the schema is valid for OpenAI's structured outputs which require all types to be defined in `$defs`.

---

## LLM Client Abstraction

`llm/src/` (279 lines across 3 files).

### LlmClient Trait

```rust
#[async_trait]
pub trait LlmClient: Send + Sync {
    async fn complete(&self, request: CompletionRequest) -> Result<String>;
    fn model(&self) -> &str;
}
```

### CompletionRequest

```rust
pub struct CompletionRequest {
    pub messages: Vec<Message>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub response_schema: Option<Value>,  // JSON Schema for structured output
}
```

`Message` has `role: Role` (System/User/Assistant) and `content: String`.

### OpenAI Client (`llm/src/openai.rs`)

```rust
let client = OpenAiClient::new("sk-...", "gpt-4o");
// or with custom base URL (Azure, proxies):
let client = OpenAiClient::from_env("gpt-4o")?.with_base_url("https://custom.endpoint/v1");
```

Structured output handling:
```rust
// When response_schema is Some(schema):
{
    "model": "gpt-4o",
    "messages": [...],
    "response_format": {
        "type": "json_schema",
        "json_schema": {
            "name": "response_schema",
            "strict": true,
            "schema": <your_schema>
        }
    }
}
```

### Ollama Client (`llm/src/ollama.rs`)

```rust
let client = OllamaClient::new("llama3");
// or custom URL:
let client = OllamaClient::new("llama3").with_url("http://gpu-server:11434");
```

Structured output handling (no native schema support):
```rust
// When response_schema is Some(schema):
// 1. Enables JSON mode: "format": "json"
// 2. Injects schema into system message:
//    "Response must be JSON.\nYou must output valid JSON that strictly matches this schema: {...}"
```

---

## AI Validation System

`core/src/validation.rs` (100 lines).

### Core Types

```rust
pub struct AiValidationError {
    pub path: String,                  // e.g., "users[0].age" or "confidence"
    pub message: String,               // human/AI-readable error description
    pub invalid_value: Option<String>,
    pub correction_hint: Option<String>,
}

pub struct ValidationReport {
    pub is_valid: bool,
    pub errors: Vec<AiValidationError>,
}
```

### Correction Prompt Generation

```rust
let report = ValidationReport::new();
report.add_error(error);

let prompt = report.to_correction_prompt();
// "The generated JSON response was invalid. Please fix the following errors:
//  1. Field `confidence`: invalid type: string "high", expected f64
//     Hint: Ensure the field name and type matches the schema exactly.
//  Return the corrected JSON only."
```

This prompt is designed to be sent back to the LLM as a follow-up message. The LLM reads the structured error list and produces corrected output.

### map_serde_error

```rust
pub fn map_serde_error(err: &serde_json::Error) -> AiValidationError
```

Converts a `serde_json::Error` into an `AiValidationError` by:
1. Extracting the full error message.
2. Using regex to extract the field name from messages like `"missing field \`id\` at line 1"`.
3. Setting `path` to the extracted field name (or `"unknown"`).
4. Adding a generic correction hint.

### AiValidatable Trait

```rust
pub trait AiValidatable {
    fn validate_ai(&self) -> ValidationReport;
}
```

Types that implement this can self-validate after deserialization, adding domain-specific checks beyond what serde catches.

### Validation Loop Pattern

```
LLM response ──▶ serde_json::from_str()
                       │
                  ┌────┴────┐
                  │ Success  │──▶ Use validated data
                  └─────────┘
                  ┌─────────┐
                  │ Error    │──▶ map_serde_error()
                  └────┬────┘       │
                       │            ▼
                       │    ValidationReport
                       │            │
                       │            ▼
                       │    to_correction_prompt()
                       │            │
                       │            ▼
                       └──── Send prompt to LLM ──▶ retry
```

---

## Agent Infrastructure

Components for building AI coding agents. Used in the `examples/code-agent/` and `examples/docu-agent/` examples.

### RustSandbox (`examples/code-agent/src/sandbox.rs`)

Creates ephemeral Rust projects for AI-generated code:

```rust
let sandbox = RustSandbox::new()?;
// Creates: /tmp/unistructgen_agent_XXXX/
//   ├── Cargo.toml   (with serde, anyhow, chrono, regex dependencies)
//   └── src/
//       └── lib.rs   (empty)

sandbox.write_code("pub fn validate_email(email: &str) -> bool { true }")?;
```

The sandbox is a real Cargo project. `cargo check` runs against it.

### Compiler (`examples/code-agent/src/compiler.rs`)

Runs `cargo check --message-format=json` and extracts structured diagnostics:

```rust
let errors: Vec<CompilerError> = Compiler::check(sandbox.path())?;

struct CompilerError {
    message: String,          // "cannot find value `ree` in this scope"
    location: Option<String>, // "src/lib.rs:5:10"
    rendered: String,         // Full colored error output
}
```

Parses the JSON diagnostic format that cargo emits, extracting message, span location, and rendered output. Falls back to stderr parsing if JSON parsing fails.

### Code Extractor (`examples/code-agent/src/sandbox.rs`)

```rust
pub fn extract_rust_code(markdown: &str) -> Option<String>
```

Extracts code from LLM responses that contain markdown code blocks:
1. Tries ` ```rust ... ``` ` blocks first
2. Falls back to generic ` ``` ... ``` ` blocks
3. If no fences found and no backticks present, assumes entire text is code

### SemanticChunker (`parsers/markdown_parser/src/chunker.rs`)

Splits markdown documents into semantically meaningful chunks for RAG:

```rust
pub struct MarkdownChunk {
    pub content: String,
    pub header_path: Vec<String>,    // ["Architecture", "Parsers", "JsonParser"]
    pub offset: usize,               // byte offset in original document
    pub metadata: ChunkMetadata,     // code_blocks count, language info
}
```

Algorithm:
1. Split by headings (preserves heading hierarchy as `header_path`).
2. Keep code blocks as atomic units (never split mid-code-block).
3. Track code block language for metadata.
4. Each chunk contains its full heading path for context in retrieval.

---

## Error Handling Strategy

Each crate defines its own error type using `thiserror`:

| Crate | Error Type | Key Variants |
|---|---|---|
| `core` | `CoreError` | TypeInference, InvalidFieldName, InvalidTypeRef, ConstraintViolation |
| `core` | `TransformError` | Transform, InvalidIR, Custom |
| `core` | `PipelineError` | Parse, Transform, Generate, Plugin |
| `core` | `PluginError` | Initialization, Execution, NotFound, AlreadyRegistered |
| `core` | `ToolError` | NotFound, ArgumentError, ExecutionError |
| `core` | `ApiError` | Generation, Parse, Validation, Config |
| `codegen` | `CodegenError` | RenderError, FormatError, ValidationError, InvalidIdentifier, UnsupportedType, MaxDepthExceeded |
| `codegen` | `JsonSchemaError` | Serialization, UnsupportedType |
| `json_parser` | `JsonParserError` | SyntaxError, InvalidStructure, TypeInferenceFailed, TypeConflict, InvalidFieldName, MaxDepthExceeded |
| `openapi_parser` | `OpenApiError` | YamlParse, JsonParse, InvalidSpec, MissingField, UnsupportedType, ReferenceResolution, CircularReference, InvalidComposition |
| `llm` | `LlmError` | Network, Api, Serialization, Config |

**Pattern**: Error types carry context (component name, file path, suggestion text). `CodegenError` includes `with_suggestion()` for enriching errors with fix hints. Pipeline errors wrap inner errors with `#[source]` for error chains.

---

## Extensibility Guide

### Adding a New Parser

1. Create a crate in `parsers/your_parser/`.
2. Implement the `Parser` trait:

```rust
use unistructgen_core::{Parser, IRModule};

pub struct YamlParser { /* config */ }

impl Parser for YamlParser {
    type Error = YamlParserError;

    fn parse(&mut self, input: &str) -> Result<IRModule, Self::Error> {
        let mut module = IRModule::new("YamlTypes".into());
        // ... parse input, create IRStruct/IREnum, add to module
        Ok(module)
    }

    fn name(&self) -> &'static str { "YamlParser" }
    fn extensions(&self) -> &[&'static str] { &["yaml", "yml"] }
}
```

3. Add to workspace `Cargo.toml`.
4. Optionally add a proc macro in `proc-macro/src/lib.rs`.

### Adding a New Code Generator

1. Implement `CodeGenerator`:

```rust
use unistructgen_core::{CodeGenerator, IRModule};

pub struct TypeScriptGenerator;

impl CodeGenerator for TypeScriptGenerator {
    type Error = TsError;

    fn generate(&self, module: &IRModule) -> Result<String, Self::Error> {
        // Iterate module.types, render TypeScript interfaces
        Ok(output)
    }

    fn language(&self) -> &'static str { "TypeScript" }
    fn file_extension(&self) -> &str { "ts" }
}
```

2. Handle all `IRTypeRef` variants and `PrimitiveKind` values.
3. Map `FieldConstraints` to target language validation (or ignore).

### Adding a New Transformer

```rust
use unistructgen_core::{IRTransformer, IRModule, TransformError};

pub struct FieldSorter;

impl IRTransformer for FieldSorter {
    fn transform(&self, mut module: IRModule) -> Result<IRModule, TransformError> {
        for ty in &mut module.types {
            if let IRType::Struct(s) = ty {
                s.fields.sort_by(|a, b| a.name.cmp(&b.name));
            }
        }
        Ok(module)
    }

    fn name(&self) -> &'static str { "FieldSorter" }
}
```

### Adding a New Plugin

```rust
use unistructgen_core::{Plugin, PluginError, IRModule};

pub struct MetricsPlugin { field_count: usize }

impl Plugin for MetricsPlugin {
    fn name(&self) -> &str { "metrics" }
    fn version(&self) -> &str { "1.0.0" }

    fn initialize(&mut self) -> Result<(), PluginError> { Ok(()) }
    fn shutdown(&mut self) -> Result<(), PluginError> {
        println!("Total fields processed: {}", self.field_count);
        Ok(())
    }

    fn after_parse(&mut self, module: IRModule) -> Result<IRModule, PluginError> {
        for ty in &module.types {
            if let IRType::Struct(s) = ty { self.field_count += s.fields.len(); }
        }
        Ok(module)
    }
}
```

### Adding a New AI Tool

Just annotate any function:

```rust
/// Search products by query
#[ai_tool]
fn search_products(query: String, max_results: i32) -> Vec<String> {
    // ... your logic
}

// Auto-generated: SearchProductsTool struct + AiTool impl
registry.register(SearchProductsTool);
```

### Adding a New LLM Provider

```rust
use llm_utl::{LlmClient, CompletionRequest, Result};

pub struct AnthropicClient { /* ... */ }

#[async_trait]
impl LlmClient for AnthropicClient {
    async fn complete(&self, request: CompletionRequest) -> Result<String> {
        // Map request to Anthropic API format
        // Handle response_schema for tool use
    }
    fn model(&self) -> &str { &self.model }
}
```

---

## Workspace & Dependency Graph

```
unistructgen (workspace root)
├── core                         ← foundation: IR, traits, pipeline, plugins, tools, validation
│   └── deps: serde, serde_json, thiserror, async-trait
│
├── codegen                      ← RustRenderer + JsonSchemaRenderer
│   └── deps: core, thiserror, serde_json
│
├── parsers/
│   ├── json_parser              ← JSON → IR
│   │   └── deps: core, serde_json, thiserror
│   ├── openapi_parser           ← OpenAPI → IR
│   │   └── deps: core, serde, serde_json, serde_yaml, thiserror
│   ├── sql_parser               ← SQL DDL → IR
│   │   └── deps: core, thiserror
│   ├── graphql_parser           ← GraphQL → IR
│   │   └── deps: core, thiserror
│   ├── markdown_parser          ← Markdown → IR + SemanticChunker
│   │   └── deps: core, thiserror
│   └── env_parser               ← .env → IR
│       └── deps: core, thiserror
│
├── proc-macro                   ← all proc macros + #[ai_tool]
│   └── deps: core, codegen, ALL parsers, syn, quote, proc-macro2, ureq
│
├── llm                          ← LLM client abstractions
│   └── deps: serde, serde_json, thiserror, async-trait, reqwest, tokio
│
├── cli                          ← CLI binary
│   └── deps: core, codegen, json_parser, openapi_parser, markdown_parser, clap, llm
│
└── examples/
    ├── tools-agent              ← deps: core, proc-macro, serde, colored
    ├── docu-agent               ← deps: core, markdown_parser, codegen, serde, colored
    └── code-agent               ← deps: serde, tempfile, regex, colored
```

### Dependency Direction

```
parsers ──▶ core ◀── codegen
               ▲
               │
          proc-macro (uses parsers + codegen + core)
               ▲
               │
            cli / examples
```

`core` depends on nothing project-internal. All other crates depend on `core`. `proc-macro` depends on everything because it needs to parse and render at compile time. `llm` is independent of `core` -- it only uses serde and async traits.
