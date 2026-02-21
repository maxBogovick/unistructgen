//! UniStructGen Core Library
//!
//! This crate provides the core types and traits for UniStructGen:
//! - **IR (Intermediate Representation)**: Language-agnostic type system
//! - **Parser trait**: Interface for implementing parsers for different formats
//! - **CodeGenerator trait**: Interface for implementing code generators for different languages
//! - **Transformer trait**: Interface for transforming IR between parsing and generation
//! - **Pipeline**: Composable processing pipeline with transformers
//! - **Plugin system**: Extensible plugin architecture for custom processing
//! - **Error types**: Common error handling infrastructure
//!
//! # Architecture
//!
//! UniStructGen follows a plugin-based pipeline architecture:
//!
//! ```text
//! Input → [Plugins] → Parser → IR → Transformers → Generator → [Plugins] → Output
//! ```
//!
//! The core library provides the foundation for this pipeline through traits
//! and data structures that ensure type safety and composability.
//!
//! # Examples
//!
//! ## Basic usage with a parser and generator:
//!
//! ```ignore
//! use unistructgen::core::{Parser, CodeGenerator};
//!
//! // Create parser and generator
//! let mut parser = MyParser::new();
//! let generator = MyGenerator::new();
//!
//! // Parse input
//! let ir_module = parser.parse(input)?;
//!
//! // Generate code
//! let code = generator.generate(&ir_module)?;
//! ```
//!
//! ## Using the Pipeline API:
//!
//! ```ignore
//! use unistructgen::core::{Pipeline, transformer::FieldOptionalizer};
//!
//! let mut pipeline = Pipeline::new(parser, generator)
//!     .add_transformer(Box::new(FieldOptionalizer::new()));
//!
//! let code = pipeline.execute(input)?;
//! ```
//!
//! ## Using Plugins:
//!
//! ```ignore
//! use unistructgen::core::{PluginRegistry, plugin::LoggingPlugin};
//!
//! let mut registry = PluginRegistry::new();
//! registry.register(Box::new(LoggingPlugin::new(true)))?;
//!
//! // Use plugins in your processing pipeline
//! let input = registry.before_parse(input)?;
//! let module = registry.after_parse(module)?;
//! let code = registry.after_generate(code)?;
//! ```

pub mod ir;
pub mod error;
pub mod parser;
pub mod codegen;
pub mod transformer;
pub mod pipeline;
pub mod plugin;
pub mod visitor;
pub mod api;
pub mod validation;
pub mod tools;
pub mod context;
pub mod diagnostics;
pub mod patch;
pub mod into_ir;

pub use async_trait::async_trait;
pub use codegen::{CodeGenerator, CodeGeneratorExt, CodegenResult, GeneratorMetadata, MultiGenerator};
pub use context::Context;
pub use error::*;
pub use into_ir::IntoIR;
// Re-export main types and traits
pub use ir::*;
pub use parser::{Parser, ParserExt, ParserMetadata, ParserResult};
pub use pipeline::{Pipeline, PipelineBuilder, PipelineError};
pub use plugin::{Plugin, PluginError, PluginRegistry};
pub use tools::{AiTool, ToolError, ToolRegistry, ToolResult};
pub use transformer::{IRTransformer, TransformError};
pub use validation::*;
pub use visitor::{walk_field, walk_module, walk_struct, walk_type, walk_type_ref, IRVisitor};

// Re-export unified API for convenient access
pub use api::{
    from_json, render_module, render_module_with_options,
    ApiError, ApiResult,
    EnumGen,
    FieldBuilder, FieldType,
    JsonGenBuilder, ModuleGen,
    RenderOptions as ApiRenderOptions, StructGen,
};
