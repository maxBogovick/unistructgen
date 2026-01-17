# UniStructGen - Project Context for Claude

This file provides context for AI assistants working on this project.

## Project Overview

**UniStructGen** — Rust code generator that creates type-safe structs from JSON, Markdown tables, OpenAPI specs, and more.

**Version**: 0.1.0
**Architecture**: Plugin-based pipeline with language-agnostic IR (Intermediate Representation)

```
Input → [Plugins] → Parser → IR → Transformers → Generator → [Plugins] → Output
```

## Workspace Structure

```
unistructgen/
├── core/                    # Core library (IR types, traits, pipeline)
│   └── src/
│       ├── lib.rs          # Main exports
│       ├── ir.rs           # IRModule, IRStruct, IRField, IRTypeRef, PrimitiveKind
│       ├── api.rs          # Unified API (StructGen, EnumGen, FieldBuilder) ← NEW
│       ├── parser.rs       # Parser trait
│       ├── codegen.rs      # CodeGenerator trait, MultiGenerator
│       ├── transformer.rs  # IRTransformer, built-in transformers
│       ├── pipeline.rs     # Pipeline, PipelineBuilder
│       ├── plugin.rs       # Plugin system
│       ├── visitor.rs      # Visitor pattern for IR traversal
│       └── error.rs        # Error types
│
├── codegen/                 # Rust code generator
│   └── src/
│       ├── lib.rs          # RustRenderer, RenderOptions, CodegenError
│       └── builder.rs      # RustRendererBuilder
│
├── parsers/
│   ├── json_parser/        # JSON → IR parser
│   │   └── src/
│   │       ├── lib.rs      # JsonParser, ParserOptions
│   │       ├── builder.rs  # JsonParserBuilder
│   │       └── inference.rs # Smart type inference
│   │
│   ├── openapi_parser/     # OpenAPI → IR parser
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── parser.rs   # OpenApiParser
│   │       ├── schema.rs   # Schema handling
│   │       ├── client.rs   # Client generation
│   │       └── ...
│   │
│   └── markdown_parser/    # Markdown tables → IR
│       └── src/lib.rs      # MarkdownParser
│
├── proc-macro/             # Procedural macros
│   └── src/lib.rs          # generate_struct_from_json!, json_struct,
│                           # struct_from_external_api!, openapi_to_rust!
│
├── cli/                    # CLI tool
│   └── src/main.rs         # Commands: generate, client
│
└── examples/               # Usage examples
```

## Key Types

### IR (Intermediate Representation) - core/src/ir.rs

```rust
IRModule { name, types: Vec<IRType> }
IRType::Struct(IRStruct) | IRType::Enum(IREnum)
IRStruct { name, fields, derives, doc, attributes }
IRField { name, source_name, ty: IRTypeRef, optional, default, constraints, attributes, doc }
IRTypeRef::Primitive(PrimitiveKind) | Option(Box) | Vec(Box) | Named(String) | Map(Box, Box)
PrimitiveKind::String | I32 | I64 | F64 | Bool | DateTime | Uuid | Decimal | Json | ...
FieldConstraints { min_length, max_length, min_value, max_value, pattern, format }
```

### Core Traits - core/src/

```rust
// Parser trait (parser.rs)
trait Parser {
    type Error;
    fn parse(&mut self, input: &str) -> Result<IRModule, Self::Error>;
    fn name(&self) -> &'static str;
    fn extensions(&self) -> &[&'static str];
}

// CodeGenerator trait (codegen.rs)
trait CodeGenerator {
    type Error;
    fn generate(&self, module: &IRModule) -> Result<String, Self::Error>;
    fn language(&self) -> &'static str;
    fn file_extension(&self) -> &str;
}

// IRTransformer trait (transformer.rs)
trait IRTransformer {
    fn transform(&self, module: IRModule) -> Result<IRModule, TransformError>;
}

// Built-in transformers:
FieldOptionalizer, DocCommentAdder, TypeDeduplicator, FieldRenamer
```

### Unified API - core/src/api.rs (NEWLY CREATED)

```rust
use unistructgen_core::{StructGen, EnumGen, ModuleGen, FieldType, FieldBuilder};

// Struct generation with fluent API
let code = StructGen::new()
    .name("User")
    .doc("User entity")
    .field("id", FieldType::I64)
    .field("name", FieldType::String)
    .field_optional("email", FieldType::String)
    .field_with(|f| f.doc("Age").range(0.0, 150.0), "age", FieldType::I32)
    .with_serde()
    .with_default()
    .generate()?;

// Enum generation
let code = EnumGen::new()
    .name("Status")
    .variant("Active")
    .variant_with_rename("InProgress", "in_progress")
    .with_serde()
    .generate()?;

// Module with multiple types
let code = ModuleGen::new("models")
    .add_struct(StructGen::new().name("User").field("id", FieldType::I64))
    .add_enum(EnumGen::new().name("Status").variant("Active"))
    .generate()?;

// Field builder with constraints
FieldBuilder::new("email", FieldType::String)
    .optional()
    .doc("User email")
    .rename("user_email")
    .length(5, 100)
    .pattern(r"^[\w@.]+$")
    .format("email")
    .build()

// Field types
FieldType::String | I32 | I64 | F64 | Bool | DateTime | Uuid | Json
FieldType::optional(FieldType::String)   // Option<String>
FieldType::vec(FieldType::I32)           // Vec<i32>
FieldType::named("Address")              // Reference to struct
```

### Pipeline Usage

```rust
use unistructgen_core::Pipeline;
use unistructgen_json_parser::{JsonParser, ParserOptions};
use unistructgen_codegen::{RustRenderer, RenderOptions};

let parser = JsonParser::new(ParserOptions {
    struct_name: "User".to_string(),
    derive_serde: true,
    ..Default::default()
});
let generator = RustRenderer::new(RenderOptions::default());

let mut pipeline = Pipeline::new(parser, generator)
    .add_transformer(Box::new(FieldOptionalizer::new()));

let code = pipeline.execute(json_input)?;
```

### Proc Macros - proc-macro/src/lib.rs

```rust
// Inline JSON
generate_struct_from_json! {
    name = "User",
    json = r#"{"id": 1, "name": "Alice"}"#,
    serde = true
}

// Attribute macro
#[json_struct(name = "User")]
const SAMPLE: &str = r#"{"id": 1}"#;

// Fetch from API at compile time
struct_from_external_api! {
    struct_name = "ApiResponse",
    url_api = "https://api.example.com/data",
    method = "GET",
    auth_bearer = "token"
}

// OpenAPI
openapi_to_rust! {
    file = "openapi.yaml",
    generate_client = true
}
```

## Development Commands

```bash
# Check compilation
cargo check -p unistructgen-core
cargo check --all

# Run tests
cargo test -p unistructgen-core
cargo test --all

# Run specific test
cargo test -p unistructgen-core api::tests::test_struct_gen_basic

# Build
cargo build --release

# CLI usage
cargo run -p unistructgen-cli -- generate --input data.json --name MyStruct
```

## Code Style

- Use `thiserror` for error types
- Builder pattern for complex configurations
- Fluent API with method chaining
- Document public APIs with `///` comments
- Tests in `#[cfg(test)] mod tests { }` at end of file

## Recent Changes

- **Created `core/src/api.rs`** — Unified API with StructGen, EnumGen, ModuleGen, FieldBuilder
- Exports added to `core/src/lib.rs`

## TODO / Future Work

- [ ] Add `from_markdown()` quick function
- [ ] Add `from_openapi()` quick function
- [ ] TypeScript code generator
- [ ] Python code generator
- [ ] Schema registry integration
- [ ] LSP support
