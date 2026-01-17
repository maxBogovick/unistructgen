# 🧩 Modules Guide

Complete guide to UniStructGen's modular architecture - understand each module, when to use it, and how to leverage its full power.

## 📋 Table of Contents

1. [Architecture Overview](#-architecture-overview)
2. [Core Module](#-core-module)
3. [JSON Parser Module](#-json-parser-module)
4. [Codegen Module](#-codegen-module)
5. [Proc-Macro Module](#-proc-macro-module)
6. [CLI Module](#-cli-module)
6. [Advanced Patterns](#-advanced-patterns)
7. [Creating Custom Parsers](#-creating-custom-parsers)

---

## 🏗️ Architecture Overview

UniStructGen follows a **pipeline architecture** with clear separation of concerns:

```
┌─────────────────────────────────────────────────────────┐
│                    Input Layer                          │
│  ┌──────────┐  ┌───────────┐  ┌─────────┐  ┌────────┐  │
│  │   JSON   │  │ Markdown  │  │   SQL   │  │  API   │  │
│  └────┬─────┘  └─────┬─────┘  └────┬────┘  └───┬────┘  │
└───────┼──────────────┼──────────────┼───────────┼───────┘
        │              │              │           │
        └──────────────┴──────────────┴───────────┘
                            │
┌───────────────────────────▼─────────────────────────────┐
│                    Parser Layer                         │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  │
│  │ JSON Parser  │  │   MD Parser  │  │  SQL Parser  │  │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘  │
└─────────┼──────────────────┼──────────────────┼─────────┘
          │                  │                  │
          └──────────────────┴──────────────────┘
                            │
┌───────────────────────────▼─────────────────────────────┐
│                  Intermediate Layer                     │
│              ┌──────────────────────┐                   │
│              │  IR (Core Module)    │                   │
│              │  - IRModule          │                   │
│              │  - IRStruct          │                   │
│              │  - IRField           │                   │
│              │  - IRType            │                   │
│              └──────────┬───────────┘                   │
└─────────────────────────┼───────────────────────────────┘
                          │
┌─────────────────────────▼───────────────────────────────┐
│                  Code Generator Layer                   │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  │
│  │ Rust Codegen │  │   TS Codegen │  │   Go Codegen │  │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘  │
└─────────┼──────────────────┼──────────────────┼─────────┘
          │                  │                  │
          └──────────────────┴──────────────────┘
                            │
┌───────────────────────────▼─────────────────────────────┐
│                    Output Layer                         │
│     ┌────────┐  ┌────────┐  ┌────────┐  ┌────────┐     │
│     │ Rust   │  │  TS    │  │   Go   │  │  More  │     │
│     └────────┘  └────────┘  └────────┘  └────────┘     │
└─────────────────────────────────────────────────────────┘
```

### Why This Architecture?

**Benefits:**
- ✅ **Modularity** - Each module has one responsibility
- ✅ **Extensibility** - Easy to add new parsers/generators
- ✅ **Testability** - Test each layer independently
- ✅ **Reusability** - Share IR across different use cases
- ✅ **Maintainability** - Clear boundaries between components

---

## 🎯 Core Module

**Package:** `unistructgen-core`

### What It Does

The **foundation** of UniStructGen. Defines the Intermediate Representation (IR) - a language-agnostic way to represent data structures.

### Why You Need It

**Use directly when:**
- Building custom parsers
- Creating custom code generators
- Programmatically constructing types
- Building tools on top of UniStructGen

### Key Components

#### 1. IRModule

The top-level container for all generated types.

```rust
use unistructgen_core::{IRModule, IRType, IRStruct};

// Create a module
let mut module = IRModule::new("MyModule".to_string());

// Add types
module.add_type(IRType::Struct(my_struct));

println!("Module has {} types", module.types.len());
```

#### 2. IRStruct

Represents a struct/class/type.

```rust
use unistructgen_core::{IRStruct, IRField, IRTypeRef, PrimitiveKind};

// Create a struct
let mut user_struct = IRStruct::new("User".to_string());

// Add derives
user_struct.add_derive("Debug".to_string());
user_struct.add_derive("Clone".to_string());

// Add fields
let id_field = IRField::new(
    "id".to_string(),
    IRTypeRef::Primitive(PrimitiveKind::I64)
);
user_struct.add_field(id_field);

let name_field = IRField::new(
    "name".to_string(),
    IRTypeRef::Primitive(PrimitiveKind::String)
);
user_struct.add_field(name_field);
```

#### 3. IRField

Represents a field in a struct.

```rust
use unistructgen_core::{IRField, IRTypeRef, PrimitiveKind};

// Simple field
let mut field = IRField::new(
    "email".to_string(),
    IRTypeRef::Primitive(PrimitiveKind::String)
);

// Add attributes
field.attributes.push("serde(rename = \"emailAddress\")".to_string());

// Make optional
field.optional = true;
field.ty = field.ty.make_optional();

// Documentation
field.documentation = Some("User's email address".to_string());
```

#### 4. IRTypeRef

Represents type references.

```rust
use unistructgen_core::{IRTypeRef, PrimitiveKind};

// Primitive types
let int_type = IRTypeRef::Primitive(PrimitiveKind::I64);
let string_type = IRTypeRef::Primitive(PrimitiveKind::String);
let bool_type = IRTypeRef::Primitive(PrimitiveKind::Bool);

// Complex types
let uuid_type = IRTypeRef::Primitive(PrimitiveKind::Uuid);
let datetime_type = IRTypeRef::Primitive(PrimitiveKind::DateTime);

// Collections
let string_vec = IRTypeRef::Vec(Box::new(
    IRTypeRef::Primitive(PrimitiveKind::String)
));

// Optional
let optional_string = IRTypeRef::Option(Box::new(
    IRTypeRef::Primitive(PrimitiveKind::String)
));

// Named types (references to other structs)
let user_type = IRTypeRef::Named("User".to_string());
```

### Real-World Example: Building Types Programmatically

```rust
use unistructgen_core::{
    IRModule, IRType, IRStruct, IRField, IRTypeRef, PrimitiveKind
};

fn create_user_module() -> IRModule {
    let mut module = IRModule::new("UserModule".to_string());

    // Create User struct
    let mut user = IRStruct::new("User".to_string());
    user.add_derive("Debug".to_string());
    user.add_derive("Clone".to_string());
    user.add_derive("serde::Serialize".to_string());
    user.add_derive("serde::Deserialize".to_string());

    // Add ID field
    let mut id_field = IRField::new(
        "id".to_string(),
        IRTypeRef::Primitive(PrimitiveKind::Uuid)
    );
    id_field.documentation = Some("Unique user identifier".to_string());
    user.add_field(id_field);

    // Add name field
    let mut name_field = IRField::new(
        "name".to_string(),
        IRTypeRef::Primitive(PrimitiveKind::String)
    );
    name_field.documentation = Some("User's full name".to_string());
    user.add_field(name_field);

    // Add optional email
    let mut email_field = IRField::new(
        "email".to_string(),
        IRTypeRef::Option(Box::new(
            IRTypeRef::Primitive(PrimitiveKind::String)
        ))
    );
    email_field.optional = true;
    user.add_field(email_field);

    // Add created_at
    let created_field = IRField::new(
        "created_at".to_string(),
        IRTypeRef::Primitive(PrimitiveKind::DateTime)
    );
    user.add_field(created_field);

    // Add tags array
    let tags_field = IRField::new(
        "tags".to_string(),
        IRTypeRef::Vec(Box::new(
            IRTypeRef::Primitive(PrimitiveKind::String)
        ))
    );
    user.add_field(tags_field);

    module.add_type(IRType::Struct(user));
    module
}

fn main() {
    let module = create_user_module();
    println!("Created module with {} types", module.types.len());

    // Now you can pass this to a code generator!
}
```

---

## 📦 JSON Parser Module

**Package:** `unistructgen-json-parser`

### What It Does

Parses JSON into IR with **smart type inference** and field name sanitization.

### Why You Need It

**Use directly when:**
- Building custom CLI tools
- Integrating into build systems
- Processing JSON at runtime
- Need fine-grained control over parsing

### Key Components

#### 1. JsonParser

The main parser implementation.

```rust
use unistructgen_json_parser::{JsonParser, ParserOptions};
use unistructgen_core::Parser;

fn main() {
    // Create parser with options
    let options = ParserOptions {
        struct_name: "User".to_string(),
        derive_serde: true,
        derive_default: false,
        make_fields_optional: false,
    };

    let mut parser = JsonParser::new(options);

    // Parse JSON
    let json = r#"{
        "id": 1,
        "name": "Alice",
        "email": "alice@example.com"
    }"#;

    let module = parser.parse(json).unwrap();

    println!("Parsed {} types", module.types.len());
}
```

#### 2. JsonParserBuilder

Fluent API for configuring parsers.

```rust
use unistructgen_json_parser::JsonParserBuilder;
use unistructgen_core::Parser;

fn main() {
    // Build parser with fluent API
    let mut parser = JsonParserBuilder::new()
        .struct_name("Product")
        .derive_serde(true)
        .derive_default(true)
        .make_optional(false)
        .build();

    let json = r#"{
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "name": "Laptop",
        "price": 999.99
    }"#;

    let module = parser.parse(json).unwrap();

    // Module contains IR ready for code generation
}
```

#### 3. Smart Type Inference

Automatic detection of special types.

```rust
use unistructgen_json_parser::{
    JsonParser, ParserOptions,
    SmartTypeInference, TypeInferenceStrategy,
    UuidDetector, DateTimeDetector
};

fn main() {
    // Configure type inference
    let mut inference = SmartTypeInference::new();

    // Add custom detectors
    inference.add_detector(Box::new(UuidDetector));
    inference.add_detector(Box::new(DateTimeDetector));

    // Use with parser
    let options = ParserOptions {
        struct_name: "Event".to_string(),
        derive_serde: true,
        derive_default: false,
        make_fields_optional: false,
    };

    let mut parser = JsonParser::new(options);

    let json = r#"{
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "timestamp": "2024-01-15T10:30:00Z",
        "message": "Event logged"
    }"#;

    let module = parser.parse(json).unwrap();

    // id -> uuid::Uuid
    // timestamp -> chrono::DateTime<Utc>
    // message -> String
}
```

### Real-World Example: Custom JSON Processor

```rust
use unistructgen_json_parser::{JsonParser, ParserOptions};
use unistructgen_core::Parser;
use std::fs;

/// Processes JSON files from a directory
fn process_json_directory(dir: &str, output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let entries = fs::read_dir(dir)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            // Read JSON file
            let json_content = fs::read_to_string(&path)?;

            // Create parser
            let struct_name = path
                .file_stem()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string();

            let options = ParserOptions {
                struct_name: struct_name.clone(),
                derive_serde: true,
                derive_default: true,
                make_fields_optional: false,
            };

            let mut parser = JsonParser::new(options);

            // Parse
            match parser.parse(&json_content) {
                Ok(module) => {
                    println!("✓ Parsed {}: {} types", struct_name, module.types.len());

                    // Now you can pass module to code generator
                    // (see Codegen Module section)
                }
                Err(e) => {
                    eprintln!("✗ Failed to parse {}: {}", struct_name, e);
                }
            }
        }
    }

    Ok(())
}

fn main() {
    process_json_directory("./schemas", "./src/models").unwrap();
}
```

### Advanced: Custom Type Inference

```rust
use unistructgen_json_parser::{
    CustomTypeDetector, TypeInferenceStrategy, SmartTypeInference
};
use unistructgen_core::{IRTypeRef, PrimitiveKind};

/// Custom detector for email addresses
struct EmailDetector;

impl CustomTypeDetector for EmailDetector {
    fn detect(&self, value: &str) -> Option<IRTypeRef> {
        if value.contains('@') && value.contains('.') {
            // Could return a custom Email type
            // For now, just return String with validation hint
            Some(IRTypeRef::Primitive(PrimitiveKind::String))
        } else {
            None
        }
    }

    fn name(&self) -> &'static str {
        "EmailDetector"
    }
}

fn main() {
    let mut inference = SmartTypeInference::new();
    inference.add_detector(Box::new(EmailDetector));

    // Now emails will be detected
}
```

---

## 🎨 Codegen Module

**Package:** `unistructgen-codegen`

### What It Does

Transforms IR into beautiful, idiomatic Rust code.

### Why You Need It

**Use directly when:**
- Building custom code generation tools
- Generating code from IR programmatically
- Customizing output formatting
- Integrating with other tools

### Key Components

#### 1. RustRenderer

The main code generator for Rust.

```rust
use unistructgen_codegen::{RustRenderer, RenderOptions};
use unistructgen_core::{IRModule, IRType, IRStruct, IRField, IRTypeRef, PrimitiveKind};

fn main() {
    // Create IR module (from parser or manually)
    let mut module = IRModule::new("User".to_string());

    let mut user = IRStruct::new("User".to_string());
    user.add_derive("Debug".to_string());

    let id_field = IRField::new(
        "id".to_string(),
        IRTypeRef::Primitive(PrimitiveKind::I64)
    );
    user.add_field(id_field);

    module.add_type(IRType::Struct(user));

    // Create renderer
    let options = RenderOptions {
        add_header: true,
        add_clippy_allows: true,
    };

    let renderer = RustRenderer::new(options);

    // Generate code
    let rust_code = renderer.render(&module).unwrap();

    println!("{}", rust_code);
}
```

**Output:**
```rust
// Generated by unistructgen v0.1.0
// Do not edit this file manually

#![allow(dead_code)]
#![allow(unused_imports)]

#[derive(Debug)]
pub struct User {
    pub id: i64,
}
```

#### 2. RenderOptions

Customize code generation.

```rust
use unistructgen_codegen::{RustRenderer, RenderOptions};

// Minimal output (no header, no clippy allows)
let minimal = RustRenderer::new(RenderOptions {
    add_header: false,
    add_clippy_allows: false,
});

// Full output (with header and allows)
let full = RustRenderer::new(RenderOptions {
    add_header: true,
    add_clippy_allows: true,
});
```

### Real-World Example: Code Generation Pipeline

```rust
use unistructgen_json_parser::{JsonParser, ParserOptions};
use unistructgen_codegen::{RustRenderer, RenderOptions};
use unistructgen_core::Parser;
use std::fs;

/// Complete pipeline: JSON -> IR -> Rust code
fn generate_rust_from_json(
    json: &str,
    struct_name: &str,
    output_path: &str
) -> Result<(), Box<dyn std::error::Error>> {
    // Step 1: Parse JSON to IR
    let options = ParserOptions {
        struct_name: struct_name.to_string(),
        derive_serde: true,
        derive_default: true,
        make_fields_optional: false,
    };

    let mut parser = JsonParser::new(options);
    let module = parser.parse(json)?;

    println!("✓ Parsed {} types", module.types.len());

    // Step 2: Generate Rust code from IR
    let render_options = RenderOptions {
        add_header: true,
        add_clippy_allows: true,
    };

    let renderer = RustRenderer::new(render_options);
    let rust_code = renderer.render(&module)?;

    println!("✓ Generated {} lines of code", rust_code.lines().count());

    // Step 3: Write to file
    fs::write(output_path, rust_code)?;

    println!("✓ Written to {}", output_path);

    Ok(())
}

fn main() {
    let json = r#"{
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "name": "John Doe",
        "email": "john@example.com",
        "createdAt": "2024-01-15T10:30:00Z"
    }"#;

    generate_rust_from_json(
        json,
        "User",
        "src/generated/user.rs"
    ).unwrap();
}
```

### Advanced: Batch Processing

```rust
use unistructgen_json_parser::JsonParser;
use unistructgen_codegen::RustRenderer;
use std::fs;
use std::path::Path;

/// Process multiple JSON schemas in parallel
fn batch_generate(
    input_dir: &str,
    output_dir: &str
) -> Result<(), Box<dyn std::error::Error>> {
    use rayon::prelude::*;

    let entries: Vec<_> = fs::read_dir(input_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().extension().and_then(|s| s.to_str()) == Some("json")
        })
        .collect();

    // Process in parallel
    entries.par_iter().for_each(|entry| {
        let path = entry.path();
        let struct_name = path.file_stem()
            .unwrap()
            .to_str()
            .unwrap();

        let json = fs::read_to_string(&path).unwrap();

        // Parse
        let options = unistructgen_json_parser::ParserOptions {
            struct_name: struct_name.to_string(),
            derive_serde: true,
            derive_default: false,
            make_fields_optional: false,
        };

        let mut parser = JsonParser::new(options);
        let module = parser.parse(&json).unwrap();

        // Generate
        let renderer = RustRenderer::new(
            unistructgen_codegen::RenderOptions {
                add_header: true,
                add_clippy_allows: true,
            }
        );

        let code = renderer.render(&module).unwrap();

        // Write
        let output_path = Path::new(output_dir)
            .join(format!("{}.rs", struct_name.to_lowercase()));

        fs::write(output_path, code).unwrap();

        println!("✓ Generated {}.rs", struct_name);
    });

    Ok(())
}

fn main() {
    batch_generate("./schemas", "./src/models").unwrap();
}
```

---

## ⚡ Proc-Macro Module

**Package:** `unistructgen-macro`

### What It Does

Provides **compile-time code generation** through procedural macros.

### Why You Need It

**Use when:**
- You want zero runtime overhead
- Schemas are known at compile time
- You want IDE autocomplete
- You're building Rust applications

### Key Macros

#### 1. generate_struct_from_json!

Function-like macro for inline JSON.

```rust
use unistructgen_macro::generate_struct_from_json;

// Basic usage
generate_struct_from_json! {
    name = "User",
    json = r#"{"id": 1, "name": "Alice"}"#
}

// With options
generate_struct_from_json! {
    name = "Config",
    json = r#"{"debug": true, "port": 8080}"#,
    serde = true,
    default = true,
    optional = false
}

fn main() {
    let user = User {
        id: 42,
        name: "Bob".to_string(),
    };

    let config = Config::default();
}
```

#### 2. struct_from_external_api!

Fetch from external APIs at compile time.

```rust
use unistructgen_macro::struct_from_external_api;

// Basic API call
struct_from_external_api! {
    struct_name = "User",
    url_api = "https://jsonplaceholder.typicode.com/users/1"
}

// With all options
struct_from_external_api! {
    struct_name = "Todo",
    url_api = "https://jsonplaceholder.typicode.com/todos",
    timeout = 10000,
    max_depth = 3,
    optional = false,
    default = true,
    serde = true
}

fn main() {
    let user = User { /* ... */ };
    let todo = Todo::default();
}
```

#### 3. #[json_struct]

Attribute macro for constants.

```rust
use unistructgen_macro::json_struct;

#[json_struct(name = "Config")]
const APP_CONFIG: &str = r#"{
    "database": {
        "host": "localhost",
        "port": 5432
    }
}"#;

fn main() {
    let config = Config {
        database: Database {
            host: "prod.example.com".to_string(),
            port: 5432,
        },
    };
}
```

### Real-World Example: Multi-API Integration

```rust
use unistructgen_macro::struct_from_external_api;

// GitHub API
struct_from_external_api! {
    struct_name = "GithubRepo",
    url_api = "https://api.github.com/repos/rust-lang/rust"
}

// JSONPlaceholder
struct_from_external_api! {
    struct_name = "Post",
    url_api = "https://jsonplaceholder.typicode.com/posts/1"
}

struct_from_external_api! {
    struct_name = "Comment",
    url_api = "https://jsonplaceholder.typicode.com/comments/1"
}

// Your API
#[cfg(not(debug_assertions))]
struct_from_external_api! {
    struct_name = "CustomData",
    url_api = "https://your-api.com/data"
}

#[cfg(debug_assertions)]
use unistructgen_macro::generate_struct_from_json;

#[cfg(debug_assertions)]
generate_struct_from_json! {
    name = "CustomData",
    json = r#"{"id": 1, "value": "dev"}"#
}

async fn fetch_all_data() {
    // All types are available and fully typed!
    let repo: GithubRepo = fetch_github_repo().await;
    let post: Post = fetch_post().await;
    let comment: Comment = fetch_comment().await;
    let data: CustomData = fetch_custom_data().await;
}
```

---

## 💻 CLI Module

**Package:** `unistructgen` (binary)

### What It Does

Command-line interface for generating code from schemas.

### Why You Need It

**Use when:**
- Building in CI/CD pipelines
- Pre-generating code
- Committing generated files
- Batch processing schemas

### Commands

#### Generate Command

```bash
# Basic usage
unistructgen generate \
    --input schema.json \
    --name User \
    --output user.rs

# With options
unistructgen generate \
    -i schema.json \
    -n User \
    -o user.rs \
    --serde true \
    --default true \
    --optional false

# From stdin
cat schema.json | unistructgen generate -n User

# To stdout
unistructgen generate -i schema.json -n User > user.rs
```

### Real-World Example: Build Script Integration

```rust
// build.rs
use std::process::Command;
use std::fs;

fn main() {
    println!("cargo:rerun-if-changed=schemas/");

    // Find all JSON schemas
    let entries = fs::read_dir("schemas").unwrap();

    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            let struct_name = path
                .file_stem()
                .unwrap()
                .to_str()
                .unwrap();

            let output_path = format!(
                "src/models/{}.rs",
                struct_name.to_lowercase()
            );

            // Generate code
            let status = Command::new("unistructgen")
                .args(&[
                    "generate",
                    "-i", path.to_str().unwrap(),
                    "-n", struct_name,
                    "-o", &output_path,
                    "--serde", "true",
                ])
                .status()
                .expect("Failed to run unistructgen");

            if !status.success() {
                panic!("Failed to generate {}", struct_name);
            }

            println!("✓ Generated {}", struct_name);
        }
    }
}
```

---

## 🚀 Advanced Patterns

### Pattern 1: Complete Custom Pipeline

Build your own tool using all modules:

```rust
use unistructgen_core::{Parser, IRModule};
use unistructgen_json_parser::{JsonParser, ParserOptions};
use unistructgen_codegen::{RustRenderer, RenderOptions};
use std::fs;

struct CodeGenerator {
    parser: JsonParser,
    renderer: RustRenderer,
}

impl CodeGenerator {
    fn new(struct_name: &str) -> Self {
        let parser_options = ParserOptions {
            struct_name: struct_name.to_string(),
            derive_serde: true,
            derive_default: true,
            make_fields_optional: false,
        };

        let render_options = RenderOptions {
            add_header: true,
            add_clippy_allows: true,
        };

        Self {
            parser: JsonParser::new(parser_options),
            renderer: RustRenderer::new(render_options),
        }
    }

    fn generate_from_json(&mut self, json: &str) -> Result<String, Box<dyn std::error::Error>> {
        // Parse
        let module = self.parser.parse(json)?;

        // Generate
        let code = self.renderer.render(&module)?;

        Ok(code)
    }

    fn generate_from_file(
        &mut self,
        input_path: &str,
        output_path: &str
    ) -> Result<(), Box<dyn std::error::Error>> {
        let json = fs::read_to_string(input_path)?;
        let code = self.generate_from_json(&json)?;
        fs::write(output_path, code)?;
        Ok(())
    }
}

fn main() {
    let mut generator = CodeGenerator::new("User");

    generator.generate_from_file(
        "schemas/user.json",
        "src/models/user.rs"
    ).unwrap();
}
```

### Pattern 2: Watch Mode

Auto-regenerate on file changes:

```rust
use notify::{Watcher, RecursiveMode, watcher};
use std::sync::mpsc::channel;
use std::time::Duration;

fn watch_and_generate(schema_dir: &str, output_dir: &str) {
    let (tx, rx) = channel();

    let mut watcher = watcher(tx, Duration::from_secs(2)).unwrap();
    watcher.watch(schema_dir, RecursiveMode::Recursive).unwrap();

    println!("👀 Watching {} for changes...", schema_dir);

    loop {
        match rx.recv() {
            Ok(event) => {
                println!("🔄 Change detected, regenerating...");

                // Regenerate all schemas
                // (use batch_generate from earlier example)

                println!("✓ Regeneration complete");
            }
            Err(e) => println!("Watch error: {:?}", e),
        }
    }
}
```

### Pattern 3: Custom Transformations

Transform IR before code generation:

```rust
use unistructgen_core::{IRModule, IRType, IRField};

fn add_custom_derives(mut module: IRModule) -> IRModule {
    for ty in &mut module.types {
        if let IRType::Struct(ref mut struct_def) = ty {
            // Add custom derives
            struct_def.add_derive("PartialEq".to_string());
            struct_def.add_derive("Eq".to_string());
            struct_def.add_derive("Hash".to_string());
        }
    }
    module
}

fn add_id_fields(mut module: IRModule) -> IRModule {
    use unistructgen_core::{IRTypeRef, PrimitiveKind};

    for ty in &mut module.types {
        if let IRType::Struct(ref mut struct_def) = ty {
            // Check if struct has ID field
            let has_id = struct_def.fields.iter()
                .any(|f| f.name == "id");

            if !has_id {
                // Add UUID id field
                let id_field = IRField::new(
                    "id".to_string(),
                    IRTypeRef::Primitive(PrimitiveKind::Uuid)
                );
                struct_def.fields.insert(0, id_field);
            }
        }
    }
    module
}

fn main() {
    // Parse JSON
    let mut parser = JsonParser::new(/* ... */);
    let module = parser.parse(json).unwrap();

    // Transform IR
    let module = add_custom_derives(module);
    let module = add_id_fields(module);

    // Generate code
    let renderer = RustRenderer::new(/* ... */);
    let code = renderer.render(&module).unwrap();
}
```

---

## 🛠️ Creating Custom Parsers

Want to parse other formats? Here's how:

### Step 1: Implement Parser Trait

```rust
use unistructgen_core::{Parser, ParserMetadata, IRModule};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum YamlParserError {
    #[error("YAML syntax error: {0}")]
    SyntaxError(String),
}

pub struct YamlParser {
    struct_name: String,
}

impl Parser for YamlParser {
    type Error = YamlParserError;

    fn parse(&mut self, input: &str) -> Result<IRModule, Self::Error> {
        // Parse YAML to IR
        // Similar to JsonParser implementation
        todo!("Implement YAML parsing")
    }

    fn name(&self) -> &'static str {
        "YAML"
    }

    fn extensions(&self) -> &[&'static str] {
        &["yaml", "yml"]
    }

    fn validate(&self, input: &str) -> Result<(), Self::Error> {
        // Validate YAML syntax
        todo!()
    }

    fn metadata(&self) -> ParserMetadata {
        ParserMetadata::new()
            .with_version("0.1.0")
            .with_description("YAML parser")
    }
}
```

### Step 2: Use Your Parser

```rust
fn main() {
    let yaml = r#"
user:
  id: 1
  name: Alice
  email: alice@example.com
"#;

    let mut parser = YamlParser {
        struct_name: "User".to_string(),
    };

    let module = parser.parse(yaml).unwrap();

    // Use with existing code generator!
    let renderer = RustRenderer::new(RenderOptions::default());
    let code = renderer.render(&module).unwrap();
}
```

---

## 📚 Summary

### Module Dependencies

```
proc-macro ──────┐
                 ├──> json_parser ──> core
cli ─────────────┤                      │
                 └──> codegen ──────────┘
```

### When to Use Each Module

| Module | Use When | Don't Use When |
|--------|----------|----------------|
| **proc-macro** | Compile-time gen, known schemas | Runtime gen, unknown schemas |
| **json_parser** | Custom tools, runtime parsing | Just need macros |
| **codegen** | Custom output, multiple languages | Just generating Rust |
| **core** | Building extensions, custom parsers | Just using the library |
| **CLI** | Build scripts, CI/CD | Compile-time generation |

### Quick Decision Tree

```
Need to generate code?
├─ At compile time?
│  └─ Yes → Use proc-macro module
│      └─ generate_struct_from_json! or struct_from_external_api!
│
├─ At build time?
│  └─ Yes → Use CLI module
│      └─ unistructgen generate
│
└─ Programmatically/Custom tool?
   └─ Yes → Use json_parser + codegen modules
       └─ Build custom pipeline
```

---

<div align="center">

**[⬅️ Back to README](README.md)** • **[📖 Quick Start](QUICKSTART.md)** • **[✨ Features](FEATURES.md)**

Master every module of UniStructGen

Made with 🦀 for extensibility and power

</div>
