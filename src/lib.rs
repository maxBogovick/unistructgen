pub mod core;
pub mod codegen;

#[cfg(feature = "llm")]
pub mod llm;

#[cfg(feature = "agent")]
pub mod agent;

#[cfg(feature = "mcp")]
pub mod mcp;

pub mod parsers;

// Prelude: Экспортируем типы из core для удобства
pub use core::*;
