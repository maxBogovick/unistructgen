mod builder;
pub mod inference;

pub use builder::JsonParserBuilder;
pub use inference::{
    CustomTypeDetector, DateTimeDetector, EmailDetector,
    SmartTypeInference, TypeInferenceStrategy, UrlDetector, UuidDetector,
};

use crate::core::{
    IRField, IRModule, IRStruct, IRType, IRTypeRef, Parser, ParserMetadata, PrimitiveKind,
};
use serde_json::Value;
use std::collections::HashSet;
use thiserror::Error;

/// Errors that can occur during JSON parsing
///
/// Each error variant provides detailed context about what went wrong
/// and where in the input it occurred.
#[derive(Error, Debug)]
pub enum JsonParserError {
    /// Failed to parse JSON syntax
    ///
    /// This typically means the input is not valid JSON.
    #[error("JSON syntax error at line {line}, column {column}: {message}")]
    SyntaxError {
        /// Line number where the error occurred
        line: usize,
        /// Column number where the error occurred
        column: usize,
        /// Description of the syntax error
        message: String,
        /// The underlying serde_json error
        #[source]
        source: serde_json::Error,
    },

    /// The JSON structure is not what was expected
    ///
    /// For example, the root must be an object, not an array or primitive.
    #[error("Invalid JSON structure at path '{path}': expected {expected}, found {found}")]
    InvalidStructure {
        /// JSON path where the error occurred (e.g., "$.user.address")
        path: String,
        /// What was expected (e.g., "object")
        expected: String,
        /// What was actually found (e.g., "array")
        found: String,
    },

    /// Failed to infer the type of a field
    ///
    /// This can happen with ambiguous or unsupported JSON values.
    #[error("Type inference failed for field '{field}' at path '{path}': {reason}")]
    TypeInferenceFailed {
        /// The field name that failed type inference
        field: String,
        /// JSON path to the field (e.g., "$.user.metadata")
        path: String,
        /// Reason why inference failed
        reason: String,
        /// Optional suggestion for fixing the issue
        suggestion: Option<String>,
    },

    /// Conflicting types detected when merging multiple samples
    ///
    /// This happens when the same field has different types in different samples.
    #[error("Type conflict for field '{field}' at path '{path}': found both {type1} and {type2}")]
    TypeConflict {
        /// The field with conflicting types
        field: String,
        /// JSON path to the field
        path: String,
        /// First type encountered
        type1: String,
        /// Second conflicting type encountered
        type2: String,
    },

    /// Invalid field name that cannot be converted to valid Rust identifier
    #[error("Invalid field name '{original}' at path '{path}': {reason}")]
    InvalidFieldName {
        /// The original field name from JSON
        original: String,
        /// JSON path where the field is located
        path: String,
        /// Reason why the name is invalid
        reason: String,
    },

    /// Maximum nesting depth exceeded
    ///
    /// Prevents stack overflow from deeply nested JSON structures.
    #[error("Maximum nesting depth of {max_depth} exceeded at path '{path}'")]
    MaxDepthExceeded {
        /// JSON path where max depth was reached
        path: String,
        /// The maximum allowed depth
        max_depth: usize,
    },
}

impl JsonParserError {
    /// Create a syntax error from a serde_json error
    pub(crate) fn from_serde_error(err: serde_json::Error) -> Self {
        Self::SyntaxError {
            line: err.line(),
            column: err.column(),
            message: err.to_string(),
            source: err,
        }
    }

    /// Create an invalid structure error
    pub(crate) fn invalid_structure(
        path: impl Into<String>,
        expected: impl Into<String>,
        found: impl Into<String>,
    ) -> Self {
        Self::InvalidStructure {
            path: path.into(),
            expected: expected.into(),
            found: found.into(),
        }
    }

    /// Create a type inference error
    #[allow(dead_code)]
    pub(crate) fn type_inference_failed(
        field: impl Into<String>,
        path: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self::TypeInferenceFailed {
            field: field.into(),
            path: path.into(),
            reason: reason.into(),
            suggestion: None,
        }
    }

    /// Add a suggestion to a type inference error
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        if let Self::TypeInferenceFailed {
            suggestion: ref mut s,
            ..
        } = self
        {
            *s = Some(suggestion.into());
        }
        self
    }
}

pub type Result<T> = std::result::Result<T, JsonParserError>;

#[derive(Debug, Clone)]
pub struct ParserOptions {
    pub struct_name: String,
    pub derive_serde: bool,
    pub derive_default: bool,
    pub make_fields_optional: bool,
}

impl Default for ParserOptions {
    fn default() -> Self {
        Self {
            struct_name: "Root".to_string(),
            derive_serde: true,
            derive_default: false,
            make_fields_optional: false,
        }
    }
}

/// JSON parser that converts JSON input into IR
///
/// This parser implements the [`Parser`] trait and provides smart type inference,
/// field name sanitization, and support for nested structures.
///
/// # Features
///
/// - **Smart Type Detection**: Automatically detects DateTime, UUID, and other special types
/// - **Nested Objects**: Generates separate struct types for nested objects
/// - **Field Naming**: Converts camelCase/PascalCase to snake_case
/// - **Serde Integration**: Optional serde derive macros and rename attributes
///
/// # Examples
///
/// ```
/// use unistructgen::parsers::json::{JsonParser, ParserOptions};
/// use unistructgen::core::Parser;
///
/// let json = r#"{"id": 1, "name": "Alice"}"#;
/// let mut parser = JsonParser::new(ParserOptions {
///     struct_name: "User".to_string(),
///     derive_serde: true,
///     ..Default::default()
/// });
///
/// let module = parser.parse(json).expect("Failed to parse");
/// assert_eq!(module.types.len(), 1);
/// ```
pub struct JsonParser {
    options: ParserOptions,
    /// Generated type names to avoid collisions
    type_names: HashSet<String>,
    /// Accumulated types (nested structs)
    accumulated_types: Vec<IRStruct>,
}

impl JsonParser {
    /// Create a new JSON parser with the given options
    ///
    /// # Arguments
    ///
    /// * `options` - Configuration options for parsing
    ///
    /// # Examples
    ///
    /// ```
    /// use unistructgen::parsers::json::{JsonParser, ParserOptions};
    ///
    /// let parser = JsonParser::new(ParserOptions {
    ///     struct_name: "User".to_string(),
    ///     derive_serde: true,
    ///     ..Default::default()
    /// });
    /// ```
    pub fn new(options: ParserOptions) -> Self {
        Self {
            options,
            type_names: HashSet::new(),
            accumulated_types: Vec::new(),
        }
    }

    fn parse_object(
        &mut self,
        obj: &serde_json::Map<String, Value>,
        struct_name: &str,
    ) -> Result<IRStruct> {
        let mut ir_struct = IRStruct::new(struct_name.to_string());

        // Add serde derives if requested
        if self.options.derive_serde {
            ir_struct.add_derive("serde::Serialize".to_string());
            ir_struct.add_derive("serde::Deserialize".to_string());
        }

        if self.options.derive_default {
            ir_struct.add_derive("Default".to_string());
        }

        for (key, value) in obj {
            let field_name = Self::sanitize_field_name(key);
            let field_type = self.infer_type(value, &Self::to_pascal_case(&field_name))?;

            let mut field = IRField::new(field_name.clone(), field_type);

            // Store original JSON key if different from sanitized name
            if key != &field_name {
                field.source_name = Some(key.clone());
            }

            // Add serde rename attribute if needed
            if self.options.derive_serde && field.source_name.is_some() {
                field.attributes.push(format!(
                    "serde(rename = \"{}\")",
                    field.source_name.as_ref().unwrap()
                ));
            }

            // Make field optional if configured
            if self.options.make_fields_optional {
                field.optional = true;
                field.ty = field.ty.make_optional();
            }

            ir_struct.add_field(field);
        }

        self.type_names.insert(struct_name.to_string());
        Ok(ir_struct)
    }

    fn infer_type(&mut self, value: &Value, type_hint: &str) -> Result<IRTypeRef> {
        match value {
            Value::Null => Ok(IRTypeRef::Option(Box::new(IRTypeRef::Primitive(
                PrimitiveKind::Json,
            )))),
            Value::Bool(_) => Ok(IRTypeRef::Primitive(PrimitiveKind::Bool)),
            Value::Number(num) => {
                if num.is_i64() {
                    Ok(IRTypeRef::Primitive(PrimitiveKind::I64))
                } else if num.is_u64() {
                    Ok(IRTypeRef::Primitive(PrimitiveKind::U64))
                } else {
                    Ok(IRTypeRef::Primitive(PrimitiveKind::F64))
                }
            }
            Value::String(s) => {
                // Try to detect special string formats
                if Self::is_datetime(s) {
                    Ok(IRTypeRef::Primitive(PrimitiveKind::DateTime))
                } else if Self::is_uuid(s) {
                    Ok(IRTypeRef::Primitive(PrimitiveKind::Uuid))
                } else {
                    Ok(IRTypeRef::Primitive(PrimitiveKind::String))
                }
            }
            Value::Array(arr) => {
                if arr.is_empty() {
                    // Empty array - use generic JSON value
                    Ok(IRTypeRef::Vec(Box::new(IRTypeRef::Primitive(
                        PrimitiveKind::Json,
                    ))))
                } else {
                    // Infer type from first element
                    let elem_type = self.infer_type(&arr[0], &format!("{}Item", type_hint))?;
                    Ok(IRTypeRef::Vec(Box::new(elem_type)))
                }
            }
            Value::Object(obj) => {
                // Generate a nested struct
                let nested_struct_name = self.generate_unique_type_name(type_hint);
                let nested_struct = self.parse_object(obj, &nested_struct_name)?;

                // Store nested struct in accumulated types
                self.accumulated_types.push(nested_struct);
                Ok(IRTypeRef::Named(nested_struct_name))
            }
        }
    }

    fn sanitize_field_name(name: &str) -> String {
        // Convert to snake_case and remove invalid characters
        let mut result = String::new();
        let mut prev_was_upper = false;

        for (i, ch) in name.chars().enumerate() {
            if ch.is_ascii_alphanumeric() {
                if ch.is_uppercase() && i > 0 && !prev_was_upper {
                    result.push('_');
                }
                result.push(ch.to_ascii_lowercase());
                prev_was_upper = ch.is_uppercase();
            } else if ch == '_' || ch == '-' || ch == ' ' {
                if !result.is_empty() && !result.ends_with('_') {
                    result.push('_');
                }
                prev_was_upper = false;
            } else {
                prev_was_upper = false;
            }
        }

        // Ensure it starts with a letter or underscore
        if result.is_empty() || result.chars().next().unwrap().is_numeric() {
            result.insert(0, '_');
        }

        // Avoid Rust keywords
        if Self::is_rust_keyword(&result) {
            result.push('_');
        }

        result
    }

    fn to_pascal_case(name: &str) -> String {
        let mut result = String::new();
        let mut capitalize_next = true;

        for ch in name.chars() {
            if ch == '_' || ch == '-' || ch == ' ' {
                capitalize_next = true;
            } else if capitalize_next {
                result.push(ch.to_ascii_uppercase());
                capitalize_next = false;
            } else {
                result.push(ch);
            }
        }

        if result.is_empty() {
            result.push('T');
        }

        result
    }

    fn generate_unique_type_name(&mut self, base_name: &str) -> String {
        let mut name = base_name.to_string();
        let mut counter = 1;

        while self.type_names.contains(&name) {
            name = format!("{}{}", base_name, counter);
            counter += 1;
        }

        self.type_names.insert(name.clone());
        name
    }

    fn is_datetime(s: &str) -> bool {
        // Simple heuristic for ISO 8601 datetime
        s.contains('T') && (s.contains('Z') || s.contains('+') || s.contains('-'))
            && s.len() >= 19
    }

    fn is_uuid(s: &str) -> bool {
        // Simple UUID pattern check
        s.len() == 36
            && s.chars().filter(|c| *c == '-').count() == 4
            && s.chars().all(|c| c.is_ascii_hexdigit() || c == '-')
    }

    fn is_rust_keyword(name: &str) -> bool {
        matches!(
            name,
            "as" | "break"
                | "const"
                | "continue"
                | "crate"
                | "else"
                | "enum"
                | "extern"
                | "false"
                | "fn"
                | "for"
                | "if"
                | "impl"
                | "in"
                | "let"
                | "loop"
                | "match"
                | "mod"
                | "move"
                | "mut"
                | "pub"
                | "ref"
                | "return"
                | "self"
                | "Self"
                | "static"
                | "struct"
                | "super"
                | "trait"
                | "true"
                | "type"
                | "unsafe"
                | "use"
                | "where"
                | "while"
                | "async"
                | "await"
                | "dyn"
        )
    }
}

// Implementation of Parser trait for JsonParser
impl Parser for JsonParser {
    type Error = JsonParserError;

    fn parse(&mut self, input: &str) -> std::result::Result<IRModule, Self::Error> {
        let value: Value = serde_json::from_str(input)
            .map_err(JsonParserError::from_serde_error)?;

        let struct_name = self.options.struct_name.clone();
        let mut module = IRModule::new(struct_name.clone());

        match value {
            Value::Object(obj) => {
                let root_struct = self.parse_object(&obj, &struct_name)?;

                // Add nested types first, then root
                for nested_type in self.accumulated_types.drain(..) {
                    module.add_type(IRType::Struct(nested_type));
                }
                module.add_type(IRType::Struct(root_struct));
            }
            Value::Array(_arr) => {
                return Err(JsonParserError::invalid_structure(
                    "$",
                    "object",
                    "array",
                ));
            }
            _ => {
                return Err(JsonParserError::invalid_structure(
                    "$",
                    "object",
                    "primitive value",
                ));
            }
        }

        Ok(module)
    }

    fn name(&self) -> &'static str {
        "JSON"
    }

    fn extensions(&self) -> &[&'static str] {
        &["json"]
    }

    fn validate(&self, input: &str) -> std::result::Result<(), Self::Error> {
        // Quick validation: just check if it's valid JSON
        serde_json::from_str::<Value>(input)
            .map_err(JsonParserError::from_serde_error)?;
        Ok(())
    }

    fn metadata(&self) -> ParserMetadata {
        ParserMetadata::new()
            .with_version(env!("CARGO_PKG_VERSION"))
            .with_description("Parses JSON with smart type inference and field name sanitization")
            .with_feature("smart-type-inference")
            .with_feature("nested-objects")
            .with_feature("array-support")
            .with_feature("datetime-detection")
            .with_feature("uuid-detection")
            .with_feature("serde-integration")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_object() {
        let json = r#"{"id": 1, "name": "Alice", "active": true}"#;
        let mut parser = JsonParser::new(ParserOptions {
            struct_name: "User".to_string(),
            ..Default::default()
        });

        let result = parser.parse(json).unwrap();
        assert_eq!(result.types.len(), 1);

        assert!(matches!(&result.types[0], IRType::Struct(_)));
        if let IRType::Struct(s) = &result.types[0] {
            assert_eq!(s.name, "User");
            assert_eq!(s.fields.len(), 3);
        }
    }

    #[test]
    fn test_field_name_sanitization() {
        assert_eq!(JsonParser::sanitize_field_name("camelCase"), "camel_case");
        assert_eq!(JsonParser::sanitize_field_name("PascalCase"), "pascal_case");
        assert_eq!(JsonParser::sanitize_field_name("snake_case"), "snake_case");
        assert_eq!(JsonParser::sanitize_field_name("kebab-case"), "kebab_case");
        assert_eq!(JsonParser::sanitize_field_name("123field"), "_123field");
    }

    #[test]
    fn test_type_inference() {
        let mut parser = JsonParser::new(ParserOptions::default());

        let json_value = serde_json::json!(42);
        let ty = parser.infer_type(&json_value, "Test").unwrap();
        assert_eq!(ty, IRTypeRef::Primitive(PrimitiveKind::I64));

        let json_value = serde_json::json!("hello");
        let ty = parser.infer_type(&json_value, "Test").unwrap();
        assert_eq!(ty, IRTypeRef::Primitive(PrimitiveKind::String));
    }
}
