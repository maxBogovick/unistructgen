//! OpenAPI/Swagger parser for UniStructGen
//!
//! This crate provides a parser that converts OpenAPI 3.0 specifications into
//! UniStructGen's Intermediate Representation (IR), enabling automatic generation
//! of Rust types, API clients, and validation code.
//!
//! # Features
//!
//! - Parse OpenAPI 3.0 specifications (YAML and JSON)
//! - Convert schemas to Rust structs and enums
//! - Generate API client traits with typed methods
//! - Support for request/response validation
//! - Handle references ($ref) and schema composition (allOf, oneOf, anyOf)
//! - Fetch specs from URLs or local files
//!
//! # Examples
//!
//! ```no_run
//! use unistructgen::parsers::openapi::{OpenApiParser, OpenApiParserOptions};
//! use unistructgen::core::Parser;
//!
//! let options = OpenApiParserOptions::builder()
//!     .generate_client(true)
//!     .generate_validation(true)
//!     .build();
//!
//! let mut parser = OpenApiParser::new(options);
//! let spec = std::fs::read_to_string("openapi.yaml").unwrap();
//! let ir_module = parser.parse(&spec).unwrap();
//! ```

pub mod client;
pub mod error;
pub mod options;
pub mod parser;
pub mod schema;
pub mod types;
pub mod validation;

#[cfg(feature = "fetch")]
pub mod fetch;

pub use error::{OpenApiError, Result};
pub use options::{OpenApiParserOptions, OpenApiParserOptionsBuilder};
pub use parser::OpenApiParser;

// Re-export core types for convenience
pub use crate::core::{IRModule, IRStruct, IRType, IRTypeRef, Parser, PrimitiveKind};
