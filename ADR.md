# Architecture Decision Records

This document captures the key architectural decisions made during the design and implementation of UniStructGen. Each record follows the format: **Context** (why we faced this decision), **Decision** (what we chose), **Consequences** (trade-offs and implications).

---

## Table of Contents

- [ADR-001: IR-Centric Architecture](#adr-001-ir-centric-architecture)
- [ADR-002: Trait-Based Extensibility Over Enums](#adr-002-trait-based-extensibility-over-enums)
- [ADR-003: Proc Macros as Primary Code Generation Path](#adr-003-proc-macros-as-primary-code-generation-path)
- [ADR-004: Compile-Time HTTP Fetching in Proc Macros](#adr-004-compile-time-http-fetching-in-proc-macros)
- [ADR-005: Reuse IR for JSON Schema Generation](#adr-005-reuse-ir-for-json-schema-generation)
- [ADR-006: Attribute Macro for AI Tool Generation](#adr-006-attribute-macro-for-ai-tool-generation)
- [ADR-007: OpenAI-Compatible Tool Definition Format](#adr-007-openai-compatible-tool-definition-format)
- [ADR-008: LLM Client Trait Abstraction with Per-Provider Structured Output Strategies](#adr-008-llm-client-trait-abstraction-with-per-provider-structured-output-strategies)
- [ADR-009: Validation Correction Prompts Over Silent Retries](#adr-009-validation-correction-prompts-over-silent-retries)
- [ADR-010: Plugin System Separate from Transformers](#adr-010-plugin-system-separate-from-transformers)
- [ADR-011: Workspace Monorepo with Fine-Grained Crates](#adr-011-workspace-monorepo-with-fine-grained-crates)
- [ADR-012: Builder Pattern for All Configuration](#adr-012-builder-pattern-for-all-configuration)
- [ADR-013: thiserror for Library Errors, anyhow for Binaries](#adr-013-thiserror-for-library-errors-anyhow-for-binaries)
- [ADR-014: Field Constraints in IR Rather Than Generator-Specific](#adr-014-field-constraints-in-ir-rather-than-generator-specific)
- [ADR-015: Semantic Chunker as Part of Markdown Parser](#adr-015-semantic-chunker-as-part-of-markdown-parser)
- [ADR-016: Ephemeral Sandbox for Compiler-Driven AI Loops](#adr-016-ephemeral-sandbox-for-compiler-driven-ai-loops)
- [ADR-017: Source Name Preservation in IR](#adr-017-source-name-preservation-in-ir)
- [ADR-018: Associated Error Types on Traits](#adr-018-associated-error-types-on-traits)

---

## ADR-001: IR-Centric Architecture

**Status**: Accepted

### Context

UniStructGen needs to parse multiple input formats (JSON, OpenAPI, SQL, GraphQL, Markdown, .env) and generate multiple output formats (Rust code, JSON Schema). A naive approach would create N*M direct conversions (each parser produces each output directly). With 6 parsers and 2 generators, that is 12 code paths, and every new parser or generator multiplies the total.

### Decision

Introduce a language-agnostic Intermediate Representation (IR) as the single interchange format. Every parser produces `IRModule`. Every generator consumes `IRModule`. The IR contains `IRStruct`, `IREnum`, `IRField`, `IRTypeRef`, and `FieldConstraints` -- enough to represent type structures from any source without being tied to any target language.

```
Parser₁ ──┐                ┌──▶ Generator₁ (Rust)
Parser₂ ──┼──▶ IRModule ──┼──▶ Generator₂ (JSON Schema)
Parser₃ ──┘                └──▶ Generator₃ (future: TypeScript)
```

### Consequences

**Positive:**
- Adding a new parser requires zero changes to generators (and vice versa). Cost is O(N+M) instead of O(N*M).
- Transformers operate on IR generically -- `FieldOptionalizer` works regardless of whether the IR came from JSON or SQL.
- The IR became the natural input for JSON Schema generation, which enabled the AI tooling layer without a separate schema system.
- IR is serializable (`serde::Serialize + Deserialize`), enabling persistence, caching, and debugging of intermediate results.

**Negative:**
- The IR is a lowest-common-denominator: it cannot represent source-specific nuances (e.g., SQL constraints like `UNIQUE`, `FOREIGN KEY` beyond what `FieldConstraints` covers).
- Two translation steps (source → IR → target) instead of one mean two places where information can be lost.
- IR design changes are breaking changes for all parsers and generators simultaneously.

**Mitigations:**
- `attributes: Vec<String>` on `IRStruct` and `IRField` serves as an escape hatch for source-specific metadata.
- `source_name` / `source_value` fields preserve original naming for serde rename generation.

---

## ADR-002: Trait-Based Extensibility Over Enums

**Status**: Accepted

### Context

The system needs to support new parsers, generators, transformers, plugins, AI tools, and LLM providers without modifying core code. Rust offers two extensibility mechanisms: enums (closed set, pattern matching) and traits (open set, dynamic dispatch).

### Decision

Define each extension point as a trait:

| Trait | Crate | Purpose |
|---|---|---|
| `Parser` | core | Input format → IR |
| `CodeGenerator` | core | IR → output code |
| `IRTransformer` | core | IR → IR |
| `Plugin` | core | Pipeline lifecycle hooks |
| `AiTool` | core | LLM function calling |
| `LlmClient` | llm | LLM provider abstraction |
| `IRVisitor` | core | IR traversal |
| `AiValidatable` | core | Domain-specific validation |

Each trait uses associated types for errors (`type Error: std::error::Error + Send + Sync + 'static`) and provides default implementations for optional methods.

### Consequences

**Positive:**
- Third-party crates can implement any trait without forking UniStructGen.
- Each implementation is a separate crate with its own dependencies (e.g., `openapi_parser` depends on `serde_yaml`, but `json_parser` does not).
- Testing is straightforward: mock implementations of traits for unit tests.

**Negative:**
- Dynamic dispatch (`Box<dyn IRTransformer>`) has a minor runtime cost compared to enum dispatch.
- Cannot exhaustively match on all implementations (unlike enums).
- Trait objects require `Send + Sync` bounds, which constrains implementation choices.

---

## ADR-003: Proc Macros as Primary Code Generation Path

**Status**: Accepted

### Context

Code generation tools typically use one of three approaches: (1) build scripts (`build.rs`) that run before compilation, (2) external CLI tools that generate files, (3) procedural macros that expand at compile time. Users need type-safe structs from data schemas with minimal friction.

### Decision

Use proc macros as the primary interface: `generate_struct_from_json!`, `openapi_to_rust!`, `#[ai_tool]`, etc. The runtime pipeline API (`Pipeline`) exists as a secondary interface for programmatic use.

### Consequences

**Positive:**
- Zero runtime cost -- all code generation happens at compile time.
- IDE support -- generated types are visible to rust-analyzer.
- No separate build step -- `cargo build` does everything.
- Generated code participates in type checking, borrow checking, and all compiler passes.

**Negative:**
- Proc macro errors are less readable than runtime errors (span information is limited).
- Proc macros cannot access the filesystem relative to the calling crate easily (`CARGO_MANIFEST_DIR` is the workaround).
- Compile-time network requests (see ADR-004) can cause non-deterministic builds.
- Debugging proc macros requires `cargo expand` or compile-error inspection.
- The proc-macro crate depends on all parsers and codegen, creating a large dependency tree.

---

## ADR-004: Compile-Time HTTP Fetching in Proc Macros

**Status**: Accepted

### Context

`struct_from_external_api!` generates Rust structs from live API responses. The struct definition must match the actual API shape. The question is when to fetch: at compile time (proc macro), at build time (build script), or at development time (manual/CLI step).

### Decision

Fetch HTTP responses at compile time inside the proc macro using `ureq` (synchronous HTTP client). Support Bearer, API key, and Basic authentication. Add `max_depth` and `max_entity_count` parameters to limit response complexity.

### Consequences

**Positive:**
- Single source of truth: the struct always matches the API response.
- No generated files to commit or maintain.
- Authentication parameters are declarative.

**Negative:**
- Builds require network access. Offline builds fail.
- API responses may change between builds, causing non-deterministic compilation.
- API rate limits can cause build failures.
- Secrets (tokens, API keys) appear in source code (proc macro arguments).
- Build times increase due to network latency.

**Mitigations:**
- `max_depth` prevents deeply nested types from exploding the generated code.
- `max_entity_count` limits array inference to avoid huge type definitions.
- `timeout` parameter (default 30s) prevents builds from hanging.
- Documentation recommends CI caching strategies and environment-variable-based tokens.

---

## ADR-005: Reuse IR for JSON Schema Generation

**Status**: Accepted

### Context

The AI tooling layer needs JSON Schema for two purposes: (1) `#[ai_tool]` exports function parameter schemas for LLM tool calling, (2) `JsonSchemaRenderer` generates response schemas for structured LLM outputs. We could build a separate JSON Schema system or reuse the existing IR.

### Decision

Implement `JsonSchemaRenderer` as a `CodeGenerator` that converts `IRModule` to JSON Schema (Draft 2020-12). The `#[ai_tool]` macro builds an `IRModule` from function arguments using `map_syn_type_to_ir()`, then calls `JsonSchemaRenderer` to generate the schema.

```
Function signature ──▶ syn types ──▶ IRField + IRTypeRef ──▶ IRModule
                                                                │
                                                                ▼
                                                     JsonSchemaRenderer
                                                                │
                                                                ▼
                                                      JSON Schema string
```

### Consequences

**Positive:**
- No separate schema definition system. The IR is the schema system.
- Any type expressible in IR automatically gets a JSON Schema representation.
- `StructGen` builder API works for both Rust code and JSON Schema generation.
- Parsers that produce IR (JSON, OpenAPI, SQL) indirectly produce JSON Schemas.

**Negative:**
- JSON Schema Draft 2020-12 has features (conditional schemas, `if`/`then`, `$dynamicRef`) that the IR cannot represent.
- The IR's `FieldConstraints` maps imperfectly to JSON Schema keywords (e.g., `pattern` in IR becomes `pattern` in JSON Schema, but IR has no `const`, `enum`, or `oneOf` for fields).
- `additionalProperties: false` is hardcoded for OpenAI compatibility, which may not suit all JSON Schema use cases.

---

## ADR-006: Attribute Macro for AI Tool Generation

**Status**: Accepted

### Context

Developers need to expose Rust functions as LLM-callable tools. Options: (1) manual `AiTool` trait implementation, (2) derive macro on a struct, (3) attribute macro on a function.

### Decision

Use an attribute macro `#[ai_tool]` on regular functions. The macro preserves the original function and generates a companion tool struct with the `AiTool` trait implementation.

```rust
#[ai_tool]
fn calculate(a: f64, b: f64) -> f64 { a + b }
// Generates: CalculateTool struct + AiTool impl
```

### Consequences

**Positive:**
- Lowest friction: one attribute on an existing function. No struct wrapping, no manual schema.
- The function remains callable as a normal Rust function.
- Description extracted from doc comments -- no duplication.
- JSON Schema generated from actual Rust types -- schema and implementation cannot drift.

**Negative:**
- The macro maps `syn::Type` to `IRTypeRef` with limited coverage: custom types default to `String`. Complex generics are not fully supported.
- Return type is always formatted with `Debug` (`format!("{:?}", result)`) -- no custom serialization.
- Async functions are not directly supported (the generated `call()` is synchronous).
- The tool struct name is derived by PascalCase conversion of the function name, which may conflict with existing types.

**Mitigations:**
- The type mapping covers all common types: `String`, `i32`, `i64`, `f64`, `bool`, `Vec<T>`, `Option<T>`.
- Future work: support `#[ai_tool(name = "...", returns = "json")]` for custom configuration.

---

## ADR-007: OpenAI-Compatible Tool Definition Format

**Status**: Accepted

### Context

`ToolRegistry::get_definitions()` needs to output tool definitions in a format that LLMs can consume. Options: (1) custom format, (2) OpenAI function calling format, (3) Anthropic tool use format, (4) abstract format that maps to multiple providers.

### Decision

Output in OpenAI's function calling format:

```json
{
  "type": "function",
  "function": {
    "name": "tool_name",
    "description": "...",
    "parameters": { /* JSON Schema */ }
  }
}
```

### Consequences

**Positive:**
- OpenAI's format is the de facto standard. Anthropic, Mistral, Llama, and most providers accept it or a close variant.
- The `parameters` field is standard JSON Schema, which we already generate.
- No conversion layer needed for the most common provider (OpenAI).

**Negative:**
- Anthropic's format has minor structural differences (`input_schema` instead of `parameters`). Users targeting only Anthropic need a thin adapter.
- The format may not support provider-specific extensions (e.g., Anthropic's `cache_control`).

**Mitigations:**
- The output is `Vec<serde_json::Value>`, so users can transform it to any format.
- Future work: `get_definitions_for(provider: Provider)` method.

---

## ADR-008: LLM Client Trait Abstraction with Per-Provider Structured Output Strategies

**Status**: Accepted

### Context

UniStructGen supports OpenAI and Ollama, which handle structured outputs differently:
- **OpenAI**: Native `response_format.json_schema` with `strict: true`.
- **Ollama**: `format: "json"` + schema injected into the system prompt as text.

We need a unified interface that abstracts over these differences.

### Decision

Define `LlmClient` as an async trait with `CompletionRequest` containing an optional `response_schema: Option<Value>`. Each provider implementation handles the schema differently:

- `OpenAiClient` sends it as `response_format.json_schema.schema` with `strict: true`.
- `OllamaClient` enables `format: "json"` and prepends the schema to the system message.

```rust
pub struct CompletionRequest {
    pub messages: Vec<Message>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub response_schema: Option<Value>,  // provider handles this differently
}
```

### Consequences

**Positive:**
- Callers write provider-agnostic code: same `CompletionRequest` works for both.
- Each provider uses the best available mechanism for structured outputs.
- Adding a new provider means implementing `LlmClient` with its own strategy.

**Negative:**
- The abstraction hides important behavioral differences: OpenAI guarantees schema compliance, Ollama does not.
- No way to pass provider-specific parameters (e.g., OpenAI's `top_p`, Ollama's `num_ctx`).
- Ollama's system prompt injection is fragile -- models may not follow the schema perfectly.

**Mitigations:**
- The validation system (ADR-009) handles cases where the LLM does not comply with the schema.
- Future work: `ProviderOptions` parameter for provider-specific settings.

---

## ADR-009: Validation Correction Prompts Over Silent Retries

**Status**: Accepted

### Context

LLMs produce malformed JSON responses (wrong types, missing fields, extra fields). The system needs a retry strategy. Options: (1) retry with the same prompt, (2) retry with a generic "try again" message, (3) generate a specific correction prompt from the validation error.

### Decision

Generate structured correction prompts from validation errors. `ValidationReport::to_correction_prompt()` produces an LLM-readable message that lists each error with its field path, message, and correction hint.

```
The generated JSON response was invalid. Please fix the following errors:
1. Field `confidence`: invalid type: string "high", expected f64
   Hint: Ensure the field name and type matches the schema exactly.
Return the corrected JSON only.
```

`map_serde_error()` converts raw `serde_json::Error` into `AiValidationError` with field path extraction via regex.

### Consequences

**Positive:**
- The LLM receives specific, actionable feedback -- not just "try again."
- Field path extraction points the LLM to the exact error location.
- Correction hints provide type-level guidance.
- Convergence is faster: typically 1-2 retries instead of 3-5.

**Negative:**
- Regex-based field path extraction from serde error messages is fragile -- serde's error format is not a stable API.
- The correction prompt format is English-only and optimized for GPT-4/Claude. Smaller models may not follow it.
- No backoff strategy or max-retry enforcement at the framework level (left to the caller).

---

## ADR-010: Plugin System Separate from Transformers

**Status**: Accepted

### Context

The system has two kinds of IR modifications: (1) `IRTransformer` changes IR structure (add fields, rename types), (2) side effects like logging, metrics, header injection, and input preprocessing. Should these be the same mechanism?

### Decision

Separate them:

- **Transformers** (`IRTransformer`): Pure functions `IRModule → IRModule`. Applied in sequence between parsing and generation. Stateless.
- **Plugins** (`Plugin`): Stateful objects with lifecycle (`initialize`/`shutdown`) and three hooks (`before_parse`, `after_parse`, `after_generate`). Can modify raw input, modify IR, and modify generated output.

### Consequences

**Positive:**
- Transformers are pure and composable -- easy to test and reason about.
- Plugins can handle concerns that span the entire pipeline (logging, metrics, auditing).
- Plugin `before_parse` can preprocess raw input (e.g., strip comments, normalize whitespace) before any parser sees it.
- Plugin `after_generate` can modify final output (e.g., add headers, run formatters) without touching the generator.

**Negative:**
- Two extension mechanisms for IR modification (`after_parse` plugin hook vs. transformer) can be confusing.
- Plugins execute in registration order with no dependency system.
- Plugin state (`&mut self`) prevents parallel plugin execution.

---

## ADR-011: Workspace Monorepo with Fine-Grained Crates

**Status**: Accepted

### Context

UniStructGen has ~15 logical components. Options: (1) single crate with feature flags, (2) monorepo with many small crates, (3) separate repositories.

### Decision

Cargo workspace monorepo. Each parser, the core, codegen, proc-macro, LLM, CLI, and each example are separate crates in one workspace.

```
unistructgen/
├── core/                 (unistructgen-core)
├── codegen/              (unistructgen-codegen)
├── parsers/
│   ├── json_parser/      (unistructgen-json-parser)
│   ├── openapi_parser/   (unistructgen-openapi-parser)
│   ├── sql_parser/       (unistructgen-sql-parser)
│   ├── graphql_parser/   (unistructgen-graphql-parser)
│   ├── markdown_parser/  (unistructgen-markdown-parser)
│   └── env_parser/       (unistructgen-env-parser)
├── proc-macro/           (unistructgen-proc-macro)
├── llm/                  (unistructgen-llm)
├── cli/                  (unistructgen-cli)
└── examples/             (separate binary crates)
```

### Consequences

**Positive:**
- Users depend only on what they need: `unistructgen-json-parser` doesn't pull in `serde_yaml` (that's `openapi_parser`'s dependency).
- Compile times are bounded: changing `sql_parser` only recompiles `sql_parser` and its dependents, not the whole workspace.
- Each crate has its own version, license, and documentation.
- Clear dependency direction: parsers → core ← codegen. No cycles.

**Negative:**
- `proc-macro` crate depends on all parsers and codegen, so it has the heaviest dependency tree.
- Publishing requires coordinating versions across ~12 crates.
- Path dependencies in `Cargo.toml` must be replaced with version dependencies for publishing.

---

## ADR-012: Builder Pattern for All Configuration

**Status**: Accepted

### Context

Parsers, generators, and pipeline components have many configuration options. Options: (1) plain structs with `Default`, (2) builder pattern, (3) TOML/YAML config files.

### Decision

Use the builder pattern with fluent API for all configurable components:

```rust
JsonParser::builder()
    .struct_name("User")
    .derive_serde()
    .derive_default()
    .make_optional()
    .build()

RustRenderer::builder()
    .add_header()
    .add_clippy_allows()
    .build()

OpenApiParserOptions::builder()
    .generate_client(true)
    .max_depth(10)
    .build()
```

Also provide `::new(options)` constructors for direct construction with option structs that implement `Default`.

### Consequences

**Positive:**
- Discoverable API: autocomplete shows all available options.
- Compile-time enforcement: `build()` can panic or return `Result` for invalid combinations.
- Backward-compatible: adding a new option to the builder doesn't break existing callers.
- Readable code: method names describe intent (`derive_serde()` vs `serde: true`).

**Negative:**
- More code to maintain (builder struct + methods + build logic for each component).
- Two ways to construct (builder vs. direct struct) can confuse new users.

---

## ADR-013: thiserror for Library Errors, anyhow for Binaries

**Status**: Accepted

### Context

Error handling strategy across the workspace. Library crates need typed errors for callers to match on. Binary crates (CLI, examples) need convenience.

### Decision

- **Library crates** (core, codegen, parsers, llm): Use `thiserror` for error enums with `#[error("...")]` display formatting and `#[source]` chains.
- **Binary crates** (cli, examples): Use `anyhow` for `Result<()>` and `.context("...")` error wrapping.

Each library crate defines its own error type:

```rust
// codegen
#[derive(Error, Debug)]
pub enum CodegenError {
    #[error("Rendering error for {component} in {context}: {message}")]
    RenderError { component: String, context: String, message: String },
    // ...
}
```

### Consequences

**Positive:**
- Library consumers can match on error variants for programmatic error handling.
- Error chains are preserved via `#[source]` for debugging.
- Binary crates have ergonomic error propagation with `?`.
- Error messages carry context (component name, field name, suggestion text).

**Negative:**
- Many error types to maintain (one per crate).
- `PipelineError` wraps inner errors as `Box<dyn Error>`, losing the specific error type.

---

## ADR-014: Field Constraints in IR Rather Than Generator-Specific

**Status**: Accepted

### Context

Validation rules (min/max length, range, pattern, format) come from source schemas (OpenAPI `minLength`, SQL `VARCHAR(100)`, builder API `.range(0, 150)`). These rules need to appear in generated Rust code as `#[validate(...)]` attributes and in JSON Schema as keywords like `minLength`, `maximum`, `pattern`.

### Decision

Store constraints in the IR as `FieldConstraints` on each `IRField`. Both `RustRenderer` and `JsonSchemaRenderer` translate them to their respective formats.

```rust
struct FieldConstraints {
    min_length: Option<usize>,
    max_length: Option<usize>,
    min_value: Option<f64>,
    max_value: Option<f64>,
    pattern: Option<String>,
    format: Option<String>,
}
```

### Consequences

**Positive:**
- Constraints survive the parser → IR → generator pipeline. OpenAPI's `minLength: 5` appears as both `#[validate(length(min = 5))]` in Rust and `"minLength": 5` in JSON Schema.
- Parsers set constraints once; generators read them without per-parser special cases.
- The builder API (`.length(5, 100)`, `.range(0, 150)`) maps directly to `FieldConstraints`.

**Negative:**
- `FieldConstraints` is a fixed struct. Adding a new constraint kind requires an IR change.
- Not all constraints translate to all targets (e.g., `pattern` generates `#[validate(regex = "...")]` in Rust but only works if the `validator` crate is a dependency).
- `format` is a free-form string. Only `"email"` and `"url"` are mapped to Rust validation attributes; others are ignored.

---

## ADR-015: Semantic Chunker as Part of Markdown Parser

**Status**: Accepted

### Context

The `docu-agent` example needs to split markdown documents into semantic chunks for RAG (Retrieval Augmented Generation). Options: (1) separate `unistructgen-chunker` crate, (2) external dependency, (3) include in `markdown_parser`.

### Decision

Include `SemanticChunker` as a module within `unistructgen-markdown-parser`. It shares the markdown parsing foundation and is co-located with the parser that understands markdown structure.

```rust
// parsers/markdown_parser/src/chunker.rs
pub struct SemanticChunker;
pub struct MarkdownChunk {
    pub content: String,
    pub header_path: Vec<String>,
    pub offset: usize,
    pub metadata: ChunkMetadata,
}
```

### Consequences

**Positive:**
- No additional crate to publish and maintain.
- Shared understanding of markdown structure between parser and chunker.
- `header_path` preserves heading hierarchy -- essential for RAG context.

**Negative:**
- Users who only need markdown table parsing still get the chunker code.
- The chunker is markdown-specific. If chunking for other formats is needed, it doesn't generalize.

---

## ADR-016: Ephemeral Sandbox for Compiler-Driven AI Loops

**Status**: Accepted

### Context

The `code-agent` pattern requires compiling AI-generated Rust code, extracting errors, and feeding them back. The generated code needs a real Cargo project to compile against. Options: (1) compile as a single file with `rustc`, (2) create a persistent project, (3) create ephemeral projects in `/tmp`.

### Decision

`RustSandbox` creates a temporary Cargo project with pre-configured dependencies (serde, anyhow, chrono, regex). `Compiler::check()` runs `cargo check --message-format=json` and parses structured diagnostics.

### Consequences

**Positive:**
- AI-generated code can use common crates (serde, chrono, regex) without the AI having to declare dependencies.
- Structured JSON diagnostics from cargo provide machine-readable errors with file locations.
- Ephemeral projects are cleaned up automatically (tempdir).
- The sandbox is isolated -- compilation failures cannot affect the host project.

**Negative:**
- First compilation in a new sandbox is slow (cargo needs to resolve and compile dependencies).
- The pre-configured dependency list is hardcoded. AI code that needs other crates will fail.
- No caching between sandbox instances -- each agent session starts cold.

**Mitigations:**
- Future work: shared target directory across sandbox instances for dependency caching.
- Future work: configurable dependency list.

---

## ADR-017: Source Name Preservation in IR

**Status**: Accepted

### Context

JSON keys, GraphQL field names, and SQL column names often don't match Rust naming conventions. `user_name` in JSON needs to be `user_name` in Rust (fine) but `created-at` in JSON must become `created_at` in Rust. The original name must be preserved for `#[serde(rename = "created-at")]`.

### Decision

`IRField` has both `name: String` (Rust-safe identifier) and `source_name: Option<String>` (original name from the source format). `IREnumVariant` has `name` and `source_value`. Generators use `source_name` to emit rename attributes.

### Consequences

**Positive:**
- Round-trip fidelity: serializing a struct back to JSON produces the original keys.
- Field sanitization (kebab-case → snake_case, keyword avoidance) doesn't lose information.
- JSON Schema uses `source_name` for property names, Rust code uses `name` for field names.

**Negative:**
- Every generator must check `source_name` and decide whether to emit a rename attribute.
- Two names per field increases IR complexity.

---

## ADR-018: Associated Error Types on Traits

**Status**: Accepted

### Context

`Parser` and `CodeGenerator` traits need to report errors. Options: (1) fixed error type (e.g., `Box<dyn Error>`), (2) associated type (`type Error`), (3) generic parameter (`Parser<E>`).

### Decision

Use associated types with bounds:

```rust
pub trait Parser {
    type Error: std::error::Error + Send + Sync + 'static;
    fn parse(&mut self, input: &str) -> Result<IRModule, Self::Error>;
}

pub trait CodeGenerator {
    type Error: std::error::Error + Send + Sync + 'static;
    fn generate(&self, module: &IRModule) -> Result<String, Self::Error>;
}
```

### Consequences

**Positive:**
- Each implementation has its own typed error. `JsonParser` returns `JsonParserError`, `RustRenderer` returns `CodegenError`. Callers can match on specific variants.
- `Send + Sync + 'static` bounds ensure errors work across threads and in `Box<dyn Error>`.
- No generic parameter pollution on structs that hold parsers/generators.

**Negative:**
- Pipeline must erase the error type to hold heterogeneous parsers/generators: `PipelineError::Parse(Box<dyn Error>)`. The specific error type is lost.
- Cannot use `?` directly across different error types without conversion.

**Mitigations:**
- `PipelineError` preserves the error chain via `#[source]` and `Box<dyn Error>`, so the original error is still accessible for debugging.
- Extension traits (`ParserExt`, `CodeGeneratorExt`) provide convenience methods that handle error conversion.

---

## Decision Log Summary

| ADR | Decision | Key Trade-off |
|---|---|---|
| 001 | IR-centric architecture | O(N+M) extensibility vs. information loss at IR boundary |
| 002 | Trait-based extensibility | Open extension vs. no exhaustive matching |
| 003 | Proc macros as primary path | Zero runtime cost vs. harder debugging |
| 004 | Compile-time HTTP fetching | Always-fresh types vs. network-dependent builds |
| 005 | IR reuse for JSON Schema | One schema system vs. limited JSON Schema expressiveness |
| 006 | `#[ai_tool]` attribute macro | Lowest friction vs. limited type support |
| 007 | OpenAI-compatible format | De facto standard vs. provider lock-in |
| 008 | Per-provider structured output | Clean abstraction vs. hidden behavioral differences |
| 009 | Correction prompts over retries | Faster convergence vs. fragile error parsing |
| 010 | Plugins separate from transformers | Clean separation vs. two IR modification mechanisms |
| 011 | Workspace monorepo | Minimal dependencies per user vs. publishing complexity |
| 012 | Builder pattern | Discoverable API vs. more code to maintain |
| 013 | thiserror + anyhow split | Typed errors in libraries, ergonomic binaries |
| 014 | Constraints in IR | Cross-generator consistency vs. fixed constraint set |
| 015 | Chunker in markdown parser | Co-location vs. coupling |
| 016 | Ephemeral sandbox | Isolation vs. cold compile times |
| 017 | Source name preservation | Round-trip fidelity vs. dual-name complexity |
| 018 | Associated error types | Typed errors vs. erasure in pipeline |
