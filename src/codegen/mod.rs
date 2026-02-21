mod builder;
mod json_schema;

pub use builder::RustRendererBuilder;
pub use json_schema::{JsonSchemaError, JsonSchemaRenderer};

use crate::core::{CodeGenerator, GeneratorMetadata, IREnum, IRField, IRModule, IRStruct, IRType, IRTypeRef};
use std::fmt::Write;
use thiserror::Error;

/// Errors that can occur during code generation
///
/// Each error variant provides detailed context about what went wrong
/// during the code generation process.
#[derive(Error, Debug)]
pub enum CodegenError {
    /// Failed to render a specific part of the code
    ///
    /// This typically indicates a bug in the generator logic.
    #[error("Rendering error for {component} in {context}: {message}")]
    RenderError {
        /// The component that failed to render (e.g., "struct", "field", "enum")
        component: String,
        /// Context where the error occurred (e.g., "User", "Address")
        context: String,
        /// Description of what went wrong
        message: String,
    },

    /// String formatting error
    ///
    /// This occurs when writing to the output buffer fails.
    #[error("Format error while rendering {context}: {source}")]
    FormatError {
        /// Context where formatting failed
        context: String,
        /// The underlying fmt::Error
        #[source]
        source: std::fmt::Error,
    },

    /// Module validation failed
    ///
    /// The IR module contains invalid data that cannot be generated.
    #[error("Validation error: {reason}")]
    ValidationError {
        /// Reason why validation failed
        reason: String,
        /// Optional suggestion for fixing the issue
        suggestion: Option<String>,
    },

    /// Empty or invalid identifier name
    #[error("Invalid identifier '{name}' in {context}: {reason}")]
    InvalidIdentifier {
        /// The invalid identifier name
        name: String,
        /// Context where the identifier is used (e.g., "struct name", "field name")
        context: String,
        /// Reason why it's invalid
        reason: String,
    },

    /// Unsupported type encountered
    #[error("Unsupported type '{type_name}' in {context}: {reason}")]
    UnsupportedType {
        /// The type that is not supported
        type_name: String,
        /// Context where the type was encountered
        context: String,
        /// Why it's not supported
        reason: String,
        /// Optional alternative or workaround
        alternative: Option<String>,
    },

    /// Maximum recursion depth exceeded
    ///
    /// Prevents stack overflow from deeply nested types.
    #[error("Maximum recursion depth of {max_depth} exceeded while rendering {context}")]
    MaxDepthExceeded {
        /// Context where max depth was reached
        context: String,
        /// The maximum allowed depth
        max_depth: usize,
    },
}

impl CodegenError {
    /// Create a render error
    #[allow(dead_code)]
    pub(crate) fn render_error(
        component: impl Into<String>,
        context: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::RenderError {
            component: component.into(),
            context: context.into(),
            message: message.into(),
        }
    }

    /// Create a validation error
    pub(crate) fn validation_error(reason: impl Into<String>) -> Self {
        Self::ValidationError {
            reason: reason.into(),
            suggestion: None,
        }
    }

    /// Add a suggestion to an error
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        let suggestion = suggestion.into();
        match &mut self {
            Self::ValidationError { suggestion: s, .. } => {
                *s = Some(suggestion);
            }
            Self::UnsupportedType { alternative, .. } => {
                *alternative = Some(suggestion);
            }
            _ => {}
        }
        self
    }

    /// Create an invalid identifier error
    pub(crate) fn invalid_identifier(
        name: impl Into<String>,
        context: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self::InvalidIdentifier {
            name: name.into(),
            context: context.into(),
            reason: reason.into(),
        }
    }
}

impl From<std::fmt::Error> for CodegenError {
    fn from(err: std::fmt::Error) -> Self {
        Self::FormatError {
            context: "unknown".to_string(),
            source: err,
        }
    }
}

pub type Result<T> = std::result::Result<T, CodegenError>;

#[derive(Debug, Clone)]
pub struct RenderOptions {
    pub add_header: bool,
    pub add_clippy_allows: bool,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            add_header: true,
            add_clippy_allows: true,
        }
    }
}

/// Rust code generator that converts IR to idiomatic Rust code
///
/// This generator implements the [`CodeGenerator`] trait and produces
/// clean, well-formatted Rust code with proper derives and attributes.
///
/// # Features
///
/// - **Derive Macros**: Automatically adds Debug, Clone, PartialEq, and optional serde derives
/// - **Nested Types**: Generates all nested struct definitions
/// - **Documentation**: Preserves doc comments from IR
/// - **Formatting**: Produces properly indented, formatted code
/// - **Type Safety**: Maps IR types to appropriate Rust types
///
/// # Examples
///
/// ```
/// use unistructgen::codegen::{RustRenderer, RenderOptions};
/// use unistructgen::core::{CodeGenerator, IRModule, IRStruct, IRType};
///
/// let mut module = IRModule::new("User".to_string());
/// let user_struct = IRStruct::new("User".to_string());
/// module.add_type(IRType::Struct(user_struct));
///
/// let renderer = RustRenderer::new(RenderOptions::default());
/// let code = renderer.generate(&module).expect("Failed to generate");
/// assert!(code.contains("pub struct User"));
/// ```
pub struct RustRenderer {
    options: RenderOptions,
}

impl RustRenderer {
    pub fn new(options: RenderOptions) -> Self {
        Self { options }
    }

    pub fn render(&self, module: &IRModule) -> Result<String> {
        let mut output = String::new();

        if self.options.add_header {
            writeln!(output, "// Generated by unistructgen v{}", env!("CARGO_PKG_VERSION"))?;
            writeln!(output, "// Do not edit this file manually")?;
            writeln!(output)?;
        }

        if self.options.add_clippy_allows {
            writeln!(output, "#![allow(dead_code)]")?;
            writeln!(output, "#![allow(unused_imports)]")?;
            writeln!(output)?;
        }

        for ty in &module.types {
            match ty {
                IRType::Struct(s) => {
                    self.render_struct(&mut output, s)?;
                    writeln!(output)?;
                }
                IRType::Enum(e) => {
                    self.render_enum(&mut output, e)?;
                    writeln!(output)?;
                }
            }
        }

        Ok(output)
    }

    fn render_struct(&self, output: &mut String, ir_struct: &IRStruct) -> Result<()> {
        // Doc comment
        if let Some(doc) = &ir_struct.doc {
            writeln!(output, "/// {}", doc)?;
        }

        // Derives
        if !ir_struct.derives.is_empty() {
            write!(output, "#[derive(")?;
            for (i, derive) in ir_struct.derives.iter().enumerate() {
                if i > 0 {
                    write!(output, ", ")?;
                }
                write!(output, "{}", derive)?;
            }
            writeln!(output, ")]")?;
        }

        // Additional attributes
        for attr in &ir_struct.attributes {
            writeln!(output, "#[{}]", attr)?;
        }

        writeln!(output, "pub struct {} {{", ir_struct.name)?;

        for field in &ir_struct.fields {
            self.render_field(output, field)?;
        }

        writeln!(output, "}}")?;

        Ok(())
    }

    fn render_field(&self, output: &mut String, field: &IRField) -> Result<()> {
        // Field doc
        if let Some(doc) = &field.doc {
            writeln!(output, "    /// {}", doc)?;
        }

        // Field attributes
        for attr in &field.attributes {
            writeln!(output, "    #[{}]", attr)?;
        }

        // Generate validation attributes from constraints
        let validation_attrs = self.generate_validation_attrs(&field.constraints);
        if !validation_attrs.is_empty() {
            writeln!(output, "    #[validate({})]", validation_attrs.join(", "))?;
        }

        // Field definition
        writeln!(
            output,
            "    pub {}: {},",
            field.name,
            self.render_type(&field.ty)?
        )?;

        Ok(())
    }

    /// Generate validation attributes from field constraints
    fn generate_validation_attrs(&self, constraints: &crate::core::FieldConstraints) -> Vec<String> {
        let mut attrs = Vec::new();

        // Length constraints (for strings and arrays)
        if constraints.min_length.is_some() || constraints.max_length.is_some() {
            let mut length_parts = Vec::new();
            if let Some(min) = constraints.min_length {
                length_parts.push(format!("min = {}", min));
            }
            if let Some(max) = constraints.max_length {
                length_parts.push(format!("max = {}", max));
            }
            attrs.push(format!("length({})", length_parts.join(", ")));
        }

        // Numeric range constraints
        if constraints.min_value.is_some() || constraints.max_value.is_some() {
            let mut range_parts = Vec::new();
            if let Some(min) = constraints.min_value {
                // Handle both integer and float values
                if min.fract() == 0.0 {
                    range_parts.push(format!("min = {}", min as i64));
                } else {
                    range_parts.push(format!("min = {}", min));
                }
            }
            if let Some(max) = constraints.max_value {
                if max.fract() == 0.0 {
                    range_parts.push(format!("max = {}", max as i64));
                } else {
                    range_parts.push(format!("max = {}", max));
                }
            }
            attrs.push(format!("range({})", range_parts.join(", ")));
        }

        // Pattern/regex constraints
        if let Some(pattern) = &constraints.pattern {
            attrs.push(format!("regex = \"{}\"", pattern.replace('\"', "\\\"")));
        }

        // Email format
        if let Some(format) = &constraints.format {
            match format.as_str() {
                "email" => attrs.push("email".to_string()),
                "url" => attrs.push("url".to_string()),
                _ => {} // Other formats not directly supported by validator crate
            }
        }

        attrs
    }

    fn render_enum(&self, output: &mut String, ir_enum: &IREnum) -> Result<()> {
        // Doc comment
        if let Some(doc) = &ir_enum.doc {
            writeln!(output, "/// {}", doc)?;
        }

        // Derives
        if !ir_enum.derives.is_empty() {
            write!(output, "#[derive(")?;
            for (i, derive) in ir_enum.derives.iter().enumerate() {
                if i > 0 {
                    write!(output, ", ")?;
                }
                write!(output, "{}", derive)?;
            }
            writeln!(output, ")]")?;
        }

        writeln!(output, "pub enum {} {{", ir_enum.name)?;

        for variant in &ir_enum.variants {
            if let Some(doc) = &variant.doc {
                writeln!(output, "    /// {}", doc)?;
            }
            // Add serde(rename) if the variant name differs from source value
            if let Some(source_value) = &variant.source_value {
                writeln!(output, "    #[serde(rename = \"{}\")]", source_value)?;
            }
            writeln!(output, "    {},", variant.name)?;
        }

        writeln!(output, "}}")?;

        Ok(())
    }

    fn render_type(&self, ty: &IRTypeRef) -> Result<String> {
        match ty {
            IRTypeRef::Primitive(p) => Ok(p.rust_type_name().to_string()),
            IRTypeRef::Option(inner) => {
                Ok(format!("Option<{}>", self.render_type(inner)?))
            }
            IRTypeRef::Vec(inner) => {
                Ok(format!("Vec<{}>", self.render_type(inner)?))
            }
            IRTypeRef::Named(name) => Ok(name.clone()),
            IRTypeRef::Map(key, value) => {
                Ok(format!(
                    "std::collections::HashMap<{}, {}>",
                    self.render_type(key)?,
                    self.render_type(value)?
                ))
            }
        }
    }
}

// Implementation of CodeGenerator trait for RustRenderer
impl CodeGenerator for RustRenderer {
    type Error = CodegenError;

    fn generate(&self, module: &IRModule) -> std::result::Result<String, Self::Error> {
        self.render(module)
    }

    fn language(&self) -> &'static str {
        "Rust"
    }

    fn file_extension(&self) -> &str {
        "rs"
    }

    fn validate(&self, module: &IRModule) -> std::result::Result<(), Self::Error> {
        // Basic validation: ensure module has at least one type
        if module.types.is_empty() {
            return Err(CodegenError::validation_error(
                "Module must contain at least one type"
            ).with_suggestion(
                "Ensure the parser generates at least one struct or enum"
            ));
        }

        // Validate that all types have valid names
        for ty in &module.types {
            match ty {
                IRType::Struct(s) => {
                    if s.name.is_empty() {
                        return Err(CodegenError::invalid_identifier(
                            "",
                            "struct name",
                            "name cannot be empty",
                        ));
                    }
                }
                IRType::Enum(e) => {
                    if e.name.is_empty() {
                        return Err(CodegenError::invalid_identifier(
                            "",
                            "enum name",
                            "name cannot be empty",
                        ));
                    }
                }
            }
        }

        Ok(())
    }

    fn format(&self, code: String) -> std::result::Result<String, Self::Error> {
        // For now, just return the code as-is
        // In the future, could integrate with rustfmt
        Ok(code)
    }

    fn metadata(&self) -> GeneratorMetadata {
        GeneratorMetadata::new()
            .with_version(env!("CARGO_PKG_VERSION"))
            .with_description("Generates idiomatic Rust code with derives and documentation")
            .with_min_language_version("1.70")
            .with_feature("derive-macros")
            .with_feature("nested-types")
            .with_feature("serde-support")
            .with_feature("doc-comments")
            .with_feature("type-safety")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{IRField, PrimitiveKind};

    #[test]
    fn test_render_simple_struct() {
        let mut ir_struct = IRStruct::new("User".to_string());
        ir_struct.add_field(IRField::new(
            "id".to_string(),
            IRTypeRef::Primitive(PrimitiveKind::I64),
        ));
        ir_struct.add_field(IRField::new(
            "name".to_string(),
            IRTypeRef::Primitive(PrimitiveKind::String),
        ));

        let mut module = IRModule::new("test".to_string());
        module.add_type(IRType::Struct(ir_struct));

        let renderer = RustRenderer::new(RenderOptions::default());
        let output = renderer.render(&module).unwrap();

        assert!(output.contains("pub struct User"));
        assert!(output.contains("pub id: i64"));
        assert!(output.contains("pub name: String"));
    }

    #[test]
    fn test_render_optional_field() {
        let mut ir_struct = IRStruct::new("User".to_string());
        ir_struct.add_field(IRField::new(
            "email".to_string(),
            IRTypeRef::Option(Box::new(IRTypeRef::Primitive(PrimitiveKind::String))),
        ));

        let mut module = IRModule::new("test".to_string());
        module.add_type(IRType::Struct(ir_struct));

        let renderer = RustRenderer::new(RenderOptions::default());
        let output = renderer.render(&module).unwrap();

        assert!(output.contains("pub email: Option<String>"));
    }

    #[test]
    fn test_render_vec_field() {
        let mut ir_struct = IRStruct::new("User".to_string());
        ir_struct.add_field(IRField::new(
            "tags".to_string(),
            IRTypeRef::Vec(Box::new(IRTypeRef::Primitive(PrimitiveKind::String))),
        ));

        let mut module = IRModule::new("test".to_string());
        module.add_type(IRType::Struct(ir_struct));

        let renderer = RustRenderer::new(RenderOptions::default());
        let output = renderer.render(&module).unwrap();

        assert!(output.contains("pub tags: Vec<String>"));
    }
}
