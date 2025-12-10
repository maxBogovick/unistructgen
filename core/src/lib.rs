//! UniStructGen Core Library
//!
//! This crate provides the core types and traits for UniStructGen:
//! - **IR (Intermediate Representation)**: Language-agnostic type system
//! - **Parser trait**: Interface for implementing parsers for different formats
//! - **CodeGenerator trait**: Interface for implementing code generators for different languages
//! - **Error types**: Common error handling infrastructure
//!
//! # Architecture
//!
//! UniStructGen follows a pipeline architecture:
//!
//! ```text
//! Input → Parser → IR → Transformer → Generator → Output
//! ```
//!
//! The core library provides the foundation for this pipeline through traits
//! and data structures that ensure type safety and composability.
//!
//! # Examples
//!
//! Basic usage with a parser and generator:
//!
//! ```ignore
//! use unistructgen_core::{Parser, CodeGenerator};
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

pub mod ir;
pub mod error;
pub mod parser;
pub mod codegen;

// Re-export main types and traits
pub use ir::*;
pub use error::*;
pub use parser::{Parser, ParserExt, ParserMetadata, ParserResult};
pub use codegen::{CodeGenerator, CodeGeneratorExt, GeneratorMetadata, CodegenResult, MultiGenerator};
