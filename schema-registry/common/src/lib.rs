//! Common types and utilities for Schema Registry

pub mod models;
pub mod error;
pub mod diff;

pub use error::{Error, Result};
pub use models::*;
