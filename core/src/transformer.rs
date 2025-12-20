//! IR transformation infrastructure
//!
//! This module provides the `IRTransformer` trait and built-in transformers
//! for modifying the intermediate representation before code generation.

use crate::{IRModule, IRStruct, IRType};
use std::error::Error as StdError;
use thiserror::Error;

/// Errors that can occur during IR transformation
#[derive(Error, Debug)]
pub enum TransformError {
    /// General transformation error
    #[error("Transformation error: {message}")]
    Transform {
        /// The transformer that failed
        transformer: String,
        /// Description of what went wrong
        message: String,
    },

    /// Invalid IR structure
    #[error("Invalid IR: {0}")]
    InvalidIR(String),

    /// Transformer-specific error
    #[error("Transformer '{transformer}' error: {message}")]
    Custom {
        /// Name of the transformer
        transformer: String,
        /// Error message
        message: String,
        /// Optional underlying error
        #[source]
        source: Option<Box<dyn StdError + Send + Sync>>,
    },
}

impl TransformError {
    /// Create a transformation error
    pub fn transform(transformer: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Transform {
            transformer: transformer.into(),
            message: message.into(),
        }
    }

    /// Create a custom error
    pub fn custom(transformer: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Custom {
            transformer: transformer.into(),
            message: message.into(),
            source: None,
        }
    }

    /// Add a source error
    pub fn with_source(mut self, error: Box<dyn StdError + Send + Sync>) -> Self {
        if let Self::Custom { source, .. } = &mut self {
            *source = Some(error);
        }
        self
    }
}

/// Trait for transforming IR modules
///
/// Transformers allow you to modify the IR between parsing and code generation.
/// This enables powerful transformations like:
/// - Making fields optional
/// - Deduplicating types
/// - Adding documentation
/// - Adding validation attributes
/// - Renaming fields
///
/// # Examples
///
/// ```
/// use unistructgen_core::{IRModule, IRTransformer, TransformError};
///
/// struct MyTransformer;
///
/// impl IRTransformer for MyTransformer {
///     fn name(&self) -> &str {
///         "MyTransformer"
///     }
///
///     fn transform(&self, module: IRModule) -> Result<IRModule, TransformError> {
///         // Modify the module here
///         Ok(module)
///     }
/// }
/// ```
pub trait IRTransformer: Send + Sync {
    /// Name of the transformer (for error messages and debugging)
    fn name(&self) -> &str;

    /// Transform the IR module
    ///
    /// This method receives ownership of the module and returns a new one.
    /// This allows transformers to freely modify the IR structure.
    fn transform(&self, module: IRModule) -> Result<IRModule, TransformError>;

    /// Optional description of what this transformer does
    fn description(&self) -> Option<&str> {
        None
    }
}

/// Transformer that makes all fields optional (wrapped in `Option<T>`)
///
/// This is useful when generating types for APIs that might return partial data.
///
/// # Examples
///
/// ```
/// use unistructgen_core::transformer::FieldOptionalizer;
/// use unistructgen_core::IRTransformer;
///
/// let transformer = FieldOptionalizer::new();
/// assert_eq!(transformer.name(), "FieldOptionalizer");
/// ```
#[derive(Debug, Clone)]
pub struct FieldOptionalizer;

impl FieldOptionalizer {
    /// Create a new field optionalizer
    pub fn new() -> Self {
        Self
    }
}

impl Default for FieldOptionalizer {
    fn default() -> Self {
        Self::new()
    }
}

impl IRTransformer for FieldOptionalizer {
    fn name(&self) -> &str {
        "FieldOptionalizer"
    }

    fn description(&self) -> Option<&str> {
        Some("Makes all struct fields optional by wrapping them in Option<T>")
    }

    fn transform(&self, mut module: IRModule) -> Result<IRModule, TransformError> {
        for ty in &mut module.types {
            if let IRType::Struct(ref mut s) = ty {
                for field in &mut s.fields {
                    if !field.ty.is_optional() {
                        field.ty = field.ty.clone().make_optional();
                    }
                }
            }
        }
        Ok(module)
    }
}

/// Transformer that adds documentation comments to all types and fields
///
/// This generates basic documentation based on the names of types and fields.
///
/// # Examples
///
/// ```
/// use unistructgen_core::transformer::DocCommentAdder;
///
/// let transformer = DocCommentAdder::new();
/// ```
#[derive(Debug, Clone)]
pub struct DocCommentAdder {
    /// Whether to generate docs for fields
    include_fields: bool,
}

impl DocCommentAdder {
    /// Create a new doc comment adder
    pub fn new() -> Self {
        Self {
            include_fields: true,
        }
    }

    /// Create a doc comment adder that only adds docs to types, not fields
    pub fn types_only() -> Self {
        Self {
            include_fields: false,
        }
    }

    /// Convert a name to a readable description
    fn humanize_name(&self, name: &str) -> String {
        // Convert snake_case or camelCase to readable text
        let words: Vec<&str> = name
            .split('_')
            .flat_map(|word| {
                // Split camelCase
                let mut result = Vec::new();
                let mut last_pos = 0;
                for (i, c) in word.char_indices() {
                    if c.is_uppercase() && i > 0 {
                        result.push(&word[last_pos..i]);
                        last_pos = i;
                    }
                }
                result.push(&word[last_pos..]);
                result
            })
            .collect();

        words.join(" ")
    }
}

impl Default for DocCommentAdder {
    fn default() -> Self {
        Self::new()
    }
}

impl IRTransformer for DocCommentAdder {
    fn name(&self) -> &str {
        "DocCommentAdder"
    }

    fn description(&self) -> Option<&str> {
        Some("Adds generated documentation comments to types and fields")
    }

    fn transform(&self, mut module: IRModule) -> Result<IRModule, TransformError> {
        for ty in &mut module.types {
            match ty {
                IRType::Struct(ref mut s) => {
                    if s.doc.is_none() {
                        s.doc = Some(format!("Represents a {}", self.humanize_name(&s.name)));
                    }

                    if self.include_fields {
                        for field in &mut s.fields {
                            if field.doc.is_none() {
                                field.doc = Some(self.humanize_name(&field.name));
                            }
                        }
                    }
                }
                IRType::Enum(ref mut e) => {
                    if e.doc.is_none() {
                        e.doc = Some(format!("Represents a {} enum", self.humanize_name(&e.name)));
                    }
                }
            }
        }
        Ok(module)
    }
}

/// Transformer that deduplicates identical struct definitions
///
/// When parsing multiple similar objects, you might end up with duplicate type definitions.
/// This transformer merges them into a single type.
///
/// # Examples
///
/// ```
/// use unistructgen_core::transformer::TypeDeduplicator;
///
/// let transformer = TypeDeduplicator::new();
/// ```
#[derive(Debug, Clone)]
pub struct TypeDeduplicator;

impl TypeDeduplicator {
    /// Create a new type deduplicator
    pub fn new() -> Self {
        Self
    }

    /// Check if two structs are identical
    fn structs_equal(&self, a: &IRStruct, b: &IRStruct) -> bool {
        // Compare field count
        if a.fields.len() != b.fields.len() {
            return false;
        }

        // Compare each field
        for (field_a, field_b) in a.fields.iter().zip(b.fields.iter()) {
            if field_a.name != field_b.name || field_a.ty != field_b.ty {
                return false;
            }
        }

        true
    }
}

impl Default for TypeDeduplicator {
    fn default() -> Self {
        Self::new()
    }
}

impl IRTransformer for TypeDeduplicator {
    fn name(&self) -> &str {
        "TypeDeduplicator"
    }

    fn description(&self) -> Option<&str> {
        Some("Removes duplicate struct definitions with identical fields")
    }

    fn transform(&self, mut module: IRModule) -> Result<IRModule, TransformError> {
        let mut seen_structs: Vec<(String, IRStruct)> = Vec::new();
        let mut unique_types: Vec<IRType> = Vec::new();

        for ty in module.types {
            match ty {
                IRType::Struct(s) => {
                    // Check if we've seen an identical struct
                    let duplicate = seen_structs
                        .iter()
                        .find(|(_, existing)| self.structs_equal(existing, &s));

                    if duplicate.is_none() {
                        seen_structs.push((s.name.clone(), s.clone()));
                        unique_types.push(IRType::Struct(s));
                    }
                    // If duplicate, skip it
                }
                other => {
                    unique_types.push(other);
                }
            }
        }

        module.types = unique_types;
        Ok(module)
    }
}

/// Transformer that renames fields based on a mapping
///
/// Useful for applying consistent naming conventions or mapping to domain-specific names.
///
/// # Examples
///
/// ```
/// use unistructgen_core::transformer::FieldRenamer;
/// use std::collections::HashMap;
///
/// let mut mappings = HashMap::new();
/// mappings.insert("id".to_string(), "identifier".to_string());
/// mappings.insert("name".to_string(), "full_name".to_string());
///
/// let transformer = FieldRenamer::new(mappings);
/// ```
#[derive(Debug, Clone)]
pub struct FieldRenamer {
    /// Mapping from old field names to new field names
    mappings: std::collections::HashMap<String, String>,
}

impl FieldRenamer {
    /// Create a new field renamer with the given mappings
    pub fn new(mappings: std::collections::HashMap<String, String>) -> Self {
        Self { mappings }
    }

    /// Add a rename mapping
    pub fn add_mapping(&mut self, from: impl Into<String>, to: impl Into<String>) {
        self.mappings.insert(from.into(), to.into());
    }
}

impl IRTransformer for FieldRenamer {
    fn name(&self) -> &str {
        "FieldRenamer"
    }

    fn description(&self) -> Option<&str> {
        Some("Renames fields based on a provided mapping")
    }

    fn transform(&self, mut module: IRModule) -> Result<IRModule, TransformError> {
        for ty in &mut module.types {
            if let IRType::Struct(ref mut s) = ty {
                for field in &mut s.fields {
                    if let Some(new_name) = self.mappings.get(&field.name) {
                        field.name = new_name.clone();
                    }
                }
            }
        }
        Ok(module)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{IRTypeRef, PrimitiveKind};

    fn create_test_struct(name: &str, fields: Vec<(&str, IRTypeRef)>) -> IRStruct {
        let mut s = IRStruct::new(name.to_string());
        for (field_name, ty) in fields {
            let field = crate::IRField::new(field_name.to_string(), ty);
            s.add_field(field);
        }
        s
    }

    #[test]
    fn test_field_optionalizer() {
        let mut module = IRModule::new("Test".to_string());
        let s = create_test_struct(
            "User",
            vec![
                ("id", IRTypeRef::Primitive(PrimitiveKind::I64)),
                ("name", IRTypeRef::Primitive(PrimitiveKind::String)),
            ],
        );
        module.add_type(IRType::Struct(s));

        let transformer = FieldOptionalizer::new();
        let result = transformer.transform(module).unwrap();

        if let IRType::Struct(s) = &result.types[0] {
            assert!(s.fields[0].ty.is_optional());
            assert!(s.fields[1].ty.is_optional());
        } else {
            panic!("Expected struct");
        }
    }

    #[test]
    fn test_doc_comment_adder() {
        let mut module = IRModule::new("Test".to_string());
        let s = create_test_struct(
            "UserProfile",
            vec![("user_name", IRTypeRef::Primitive(PrimitiveKind::String))],
        );
        module.add_type(IRType::Struct(s));

        let transformer = DocCommentAdder::new();
        let result = transformer.transform(module).unwrap();

        if let IRType::Struct(s) = &result.types[0] {
            assert!(s.doc.is_some());
            assert!(s.fields[0].doc.is_some());
        } else {
            panic!("Expected struct");
        }
    }

    #[test]
    fn test_type_deduplicator() {
        let mut module = IRModule::new("Test".to_string());

        // Add two identical structs
        let s1 = create_test_struct(
            "User1",
            vec![("id", IRTypeRef::Primitive(PrimitiveKind::I64))],
        );
        let s2 = create_test_struct(
            "User2",
            vec![("id", IRTypeRef::Primitive(PrimitiveKind::I64))],
        );

        module.add_type(IRType::Struct(s1));
        module.add_type(IRType::Struct(s2));

        let transformer = TypeDeduplicator::new();
        let result = transformer.transform(module).unwrap();

        // Should have deduplicated to 1 struct
        assert_eq!(result.types.len(), 1);
    }

    #[test]
    fn test_field_renamer() {
        let mut module = IRModule::new("Test".to_string());
        let s = create_test_struct(
            "User",
            vec![
                ("id", IRTypeRef::Primitive(PrimitiveKind::I64)),
                ("name", IRTypeRef::Primitive(PrimitiveKind::String)),
            ],
        );
        module.add_type(IRType::Struct(s));

        let mut mappings = std::collections::HashMap::new();
        mappings.insert("id".to_string(), "identifier".to_string());
        mappings.insert("name".to_string(), "full_name".to_string());

        let transformer = FieldRenamer::new(mappings);
        let result = transformer.transform(module).unwrap();

        if let IRType::Struct(s) = &result.types[0] {
            assert_eq!(s.fields[0].name, "identifier");
            assert_eq!(s.fields[1].name, "full_name");
        } else {
            panic!("Expected struct");
        }
    }
}
