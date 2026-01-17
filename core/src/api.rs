//! Unified API for struct generation
//!
//! This module provides a convenient, ergonomic API for generating Rust structs
//! from various data sources (JSON, Markdown, OpenAPI) in one place.
//!
//! # Quick Start
//!
//! ```ignore
//! use unistructgen_core::api::*;
//!
//! // Quick generation from JSON
//! let code = from_json(r#"{"id": 1, "name": "Alice"}"#)
//!     .struct_name("User")
//!     .with_serde()
//!     .generate()?;
//!
//! // Using the builder for more control
//! let code = StructGen::new()
//!     .name("User")
//!     .field("id", FieldType::I64)
//!     .field("name", FieldType::String)
//!     .field_optional("email", FieldType::String)
//!     .with_serde()
//!     .with_default()
//!     .generate()?;
//! ```

use crate::{
    IRModule, IRStruct, IRType, IRField, IRTypeRef, IREnum, IREnumVariant,
    PrimitiveKind, FieldConstraints,
};

/// Result type for API operations
pub type ApiResult<T> = Result<T, ApiError>;

/// Errors that can occur during API operations
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Generation error: {0}")]
    Generation(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Configuration error: {0}")]
    Config(String),
}

// ============================================================================
// Field Type Enum - Simple type specification
// ============================================================================

/// Simplified field type specification for the builder API
#[derive(Debug, Clone)]
pub enum FieldType {
    // Primitives
    String,
    I8, I16, I32, I64, I128,
    U8, U16, U32, U64, U128,
    F32, F64,
    Bool,
    Char,

    // Special types
    DateTime,
    Uuid,
    Decimal,
    Json,

    // Compound types
    Option(Box<FieldType>),
    Vec(Box<FieldType>),
    Map(Box<FieldType>, Box<FieldType>),

    /// Reference to another struct/enum by name
    Named(std::string::String),
}

impl FieldType {
    /// Create an optional type
    pub fn optional(inner: FieldType) -> Self {
        FieldType::Option(Box::new(inner))
    }

    /// Create a vector type
    pub fn vec(inner: FieldType) -> Self {
        FieldType::Vec(Box::new(inner))
    }

    /// Create a map type
    pub fn map(key: FieldType, value: FieldType) -> Self {
        FieldType::Map(Box::new(key), Box::new(value))
    }

    /// Create a named reference type
    pub fn named(name: impl Into<std::string::String>) -> Self {
        FieldType::Named(name.into())
    }

    /// Convert to IR type reference
    pub fn to_ir(&self) -> IRTypeRef {
        match self {
            FieldType::String => IRTypeRef::Primitive(PrimitiveKind::String),
            FieldType::I8 => IRTypeRef::Primitive(PrimitiveKind::I8),
            FieldType::I16 => IRTypeRef::Primitive(PrimitiveKind::I16),
            FieldType::I32 => IRTypeRef::Primitive(PrimitiveKind::I32),
            FieldType::I64 => IRTypeRef::Primitive(PrimitiveKind::I64),
            FieldType::I128 => IRTypeRef::Primitive(PrimitiveKind::I128),
            FieldType::U8 => IRTypeRef::Primitive(PrimitiveKind::U8),
            FieldType::U16 => IRTypeRef::Primitive(PrimitiveKind::U16),
            FieldType::U32 => IRTypeRef::Primitive(PrimitiveKind::U32),
            FieldType::U64 => IRTypeRef::Primitive(PrimitiveKind::U64),
            FieldType::U128 => IRTypeRef::Primitive(PrimitiveKind::U128),
            FieldType::F32 => IRTypeRef::Primitive(PrimitiveKind::F32),
            FieldType::F64 => IRTypeRef::Primitive(PrimitiveKind::F64),
            FieldType::Bool => IRTypeRef::Primitive(PrimitiveKind::Bool),
            FieldType::Char => IRTypeRef::Primitive(PrimitiveKind::Char),
            FieldType::DateTime => IRTypeRef::Primitive(PrimitiveKind::DateTime),
            FieldType::Uuid => IRTypeRef::Primitive(PrimitiveKind::Uuid),
            FieldType::Decimal => IRTypeRef::Primitive(PrimitiveKind::Decimal),
            FieldType::Json => IRTypeRef::Primitive(PrimitiveKind::Json),
            FieldType::Option(inner) => IRTypeRef::Option(Box::new(inner.to_ir())),
            FieldType::Vec(inner) => IRTypeRef::Vec(Box::new(inner.to_ir())),
            FieldType::Map(k, v) => IRTypeRef::Map(Box::new(k.to_ir()), Box::new(v.to_ir())),
            FieldType::Named(name) => IRTypeRef::Named(name.clone()),
        }
    }
}

// ============================================================================
// FieldBuilder - Build fields with constraints
// ============================================================================

/// Builder for creating struct fields with all options
#[derive(Debug, Clone)]
pub struct FieldBuilder {
    name: std::string::String,
    ty: FieldType,
    optional: bool,
    default: Option<std::string::String>,
    doc: Option<std::string::String>,
    attributes: Vec<std::string::String>,
    constraints: FieldConstraints,
    source_name: Option<std::string::String>,
}

impl FieldBuilder {
    /// Create a new field builder
    pub fn new(name: impl Into<std::string::String>, ty: FieldType) -> Self {
        Self {
            name: name.into(),
            ty,
            optional: false,
            default: None,
            doc: None,
            attributes: Vec::new(),
            constraints: FieldConstraints::default(),
            source_name: None,
        }
    }

    /// Mark field as optional (wraps in Option<T>)
    pub fn optional(mut self) -> Self {
        self.optional = true;
        self
    }

    /// Set default value
    pub fn default_value(mut self, value: impl Into<std::string::String>) -> Self {
        self.default = Some(value.into());
        self
    }

    /// Add documentation comment
    pub fn doc(mut self, doc: impl Into<std::string::String>) -> Self {
        self.doc = Some(doc.into());
        self
    }

    /// Add custom attribute
    pub fn attr(mut self, attr: impl Into<std::string::String>) -> Self {
        self.attributes.push(attr.into());
        self
    }

    /// Set serde rename attribute
    pub fn rename(mut self, name: impl Into<std::string::String>) -> Self {
        let rename_name = name.into();
        self.source_name = Some(rename_name.clone());
        self.attributes.push(format!("serde(rename = \"{}\")", rename_name));
        self
    }

    /// Set minimum length constraint
    pub fn min_length(mut self, len: usize) -> Self {
        self.constraints.min_length = Some(len);
        self
    }

    /// Set maximum length constraint
    pub fn max_length(mut self, len: usize) -> Self {
        self.constraints.max_length = Some(len);
        self
    }

    /// Set length range constraint
    pub fn length(mut self, min: usize, max: usize) -> Self {
        self.constraints.min_length = Some(min);
        self.constraints.max_length = Some(max);
        self
    }

    /// Set minimum value constraint
    pub fn min_value(mut self, val: f64) -> Self {
        self.constraints.min_value = Some(val);
        self
    }

    /// Set maximum value constraint
    pub fn max_value(mut self, val: f64) -> Self {
        self.constraints.max_value = Some(val);
        self
    }

    /// Set value range constraint
    pub fn range(mut self, min: f64, max: f64) -> Self {
        self.constraints.min_value = Some(min);
        self.constraints.max_value = Some(max);
        self
    }

    /// Set regex pattern constraint
    pub fn pattern(mut self, pattern: impl Into<std::string::String>) -> Self {
        self.constraints.pattern = Some(pattern.into());
        self
    }

    /// Set format constraint (email, url, etc.)
    pub fn format(mut self, format: impl Into<std::string::String>) -> Self {
        self.constraints.format = Some(format.into());
        self
    }

    /// Build into IRField
    pub fn build(self) -> IRField {
        let mut ty = self.ty.to_ir();
        if self.optional {
            ty = ty.make_optional();
        }

        IRField {
            name: self.name,
            source_name: self.source_name,
            ty,
            optional: self.optional,
            default: self.default,
            constraints: self.constraints,
            attributes: self.attributes,
            doc: self.doc,
        }
    }
}

// ============================================================================
// StructGen - Main struct builder
// ============================================================================

/// Main builder for generating Rust structs
///
/// # Examples
///
/// ```ignore
/// use unistructgen_core::api::{StructGen, FieldType};
///
/// let code = StructGen::new()
///     .name("User")
///     .doc("Represents a user in the system")
///     .field("id", FieldType::I64)
///     .field("username", FieldType::String)
///     .field_optional("email", FieldType::String)
///     .field_with(|f| f
///         .doc("User's age")
///         .range(0.0, 150.0)
///     , "age", FieldType::I32)
///     .with_serde()
///     .with_debug()
///     .with_clone()
///     .generate()?;
/// ```
#[derive(Debug, Clone)]
pub struct StructGen {
    name: std::string::String,
    fields: Vec<IRField>,
    derives: Vec<std::string::String>,
    attributes: Vec<std::string::String>,
    doc: Option<std::string::String>,
    nested_types: Vec<IRType>,
}

impl StructGen {
    /// Create a new struct generator
    pub fn new() -> Self {
        Self {
            name: "Generated".to_string(),
            fields: Vec::new(),
            derives: vec![
                "Debug".to_string(),
                "Clone".to_string(),
                "PartialEq".to_string(),
            ],
            attributes: Vec::new(),
            doc: None,
            nested_types: Vec::new(),
        }
    }

    /// Set struct name
    pub fn name(mut self, name: impl Into<std::string::String>) -> Self {
        self.name = name.into();
        self
    }

    /// Add documentation comment
    pub fn doc(mut self, doc: impl Into<std::string::String>) -> Self {
        self.doc = Some(doc.into());
        self
    }

    /// Add a simple field
    pub fn field(mut self, name: impl Into<std::string::String>, ty: FieldType) -> Self {
        self.fields.push(FieldBuilder::new(name, ty).build());
        self
    }

    /// Add an optional field
    pub fn field_optional(mut self, name: impl Into<std::string::String>, ty: FieldType) -> Self {
        self.fields.push(FieldBuilder::new(name, ty).optional().build());
        self
    }

    /// Add a field with full builder customization
    pub fn field_with<F>(mut self, builder_fn: F, name: impl Into<std::string::String>, ty: FieldType) -> Self
    where
        F: FnOnce(FieldBuilder) -> FieldBuilder,
    {
        let builder = FieldBuilder::new(name, ty);
        self.fields.push(builder_fn(builder).build());
        self
    }

    /// Add a pre-built field
    pub fn field_raw(mut self, field: IRField) -> Self {
        self.fields.push(field);
        self
    }

    /// Add a custom derive
    pub fn derive(mut self, derive: impl Into<std::string::String>) -> Self {
        let d = derive.into();
        if !self.derives.contains(&d) {
            self.derives.push(d);
        }
        self
    }

    /// Add serde derives (Serialize, Deserialize)
    pub fn with_serde(self) -> Self {
        self.derive("serde::Serialize").derive("serde::Deserialize")
    }

    /// Add Debug derive
    pub fn with_debug(self) -> Self {
        self.derive("Debug")
    }

    /// Add Clone derive
    pub fn with_clone(self) -> Self {
        self.derive("Clone")
    }

    /// Add Default derive
    pub fn with_default(self) -> Self {
        self.derive("Default")
    }

    /// Add PartialEq derive
    pub fn with_partial_eq(self) -> Self {
        self.derive("PartialEq")
    }

    /// Add Eq derive (also adds PartialEq if missing)
    pub fn with_eq(self) -> Self {
        self.derive("PartialEq").derive("Eq")
    }

    /// Add Hash derive
    pub fn with_hash(self) -> Self {
        self.derive("Hash")
    }

    /// Add all common derives (Debug, Clone, PartialEq, Eq, Hash, Default)
    pub fn with_common_derives(self) -> Self {
        self.with_debug()
            .with_clone()
            .with_eq()
            .with_hash()
            .with_default()
    }

    /// Add custom attribute
    pub fn attr(mut self, attr: impl Into<std::string::String>) -> Self {
        self.attributes.push(attr.into());
        self
    }

    /// Add a nested struct type
    pub fn nested_struct(mut self, nested: StructGen) -> Self {
        self.nested_types.push(IRType::Struct(nested.into_ir_struct()));
        self
    }

    /// Add a nested enum type
    pub fn nested_enum(mut self, nested: EnumGen) -> Self {
        self.nested_types.push(IRType::Enum(nested.into_ir_enum()));
        self
    }

    /// Build the IR struct (without generating code)
    pub fn build_ir_struct(&self) -> IRStruct {
        IRStruct {
            name: self.name.clone(),
            fields: self.fields.clone(),
            derives: self.derives.clone(),
            doc: self.doc.clone(),
            attributes: self.attributes.clone(),
        }
    }

    /// Consume and build the IR struct
    pub fn into_ir_struct(self) -> IRStruct {
        IRStruct {
            name: self.name,
            fields: self.fields,
            derives: self.derives,
            doc: self.doc,
            attributes: self.attributes,
        }
    }

    /// Build the IR module containing this struct and nested types
    pub fn build_ir_module(self) -> IRModule {
        let StructGen {
            name,
            fields,
            derives,
            attributes,
            doc,
            nested_types,
        } = self;

        let ir_struct = IRStruct {
            name: name.clone(),
            fields,
            derives,
            doc,
            attributes,
        };

        let mut module = IRModule::new(name);

        // Add nested types first
        for nested in nested_types {
            module.add_type(nested);
        }

        module.add_type(IRType::Struct(ir_struct));
        module
    }

    /// Generate Rust code string
    pub fn generate(self) -> ApiResult<std::string::String> {
        let module = self.build_ir_module();
        render_module(&module)
    }

    /// Generate with custom render options
    pub fn generate_with_options(self, options: RenderOptions) -> ApiResult<std::string::String> {
        let module = self.build_ir_module();
        render_module_with_options(&module, options)
    }
}

impl Default for StructGen {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// EnumGen - Enum builder
// ============================================================================

/// Builder for generating Rust enums
///
/// # Examples
///
/// ```ignore
/// use unistructgen_core::api::EnumGen;
///
/// let code = EnumGen::new()
///     .name("Status")
///     .variant("Active")
///     .variant("Inactive")
///     .variant_with_rename("Pending", "pending")
///     .with_serde()
///     .generate()?;
/// ```
#[derive(Debug, Clone)]
pub struct EnumGen {
    name: std::string::String,
    variants: Vec<IREnumVariant>,
    derives: Vec<std::string::String>,
    doc: Option<std::string::String>,
}

impl EnumGen {
    /// Create a new enum generator
    pub fn new() -> Self {
        Self {
            name: "Generated".to_string(),
            variants: Vec::new(),
            derives: vec![
                "Debug".to_string(),
                "Clone".to_string(),
                "PartialEq".to_string(),
            ],
            doc: None,
        }
    }

    /// Set enum name
    pub fn name(mut self, name: impl Into<std::string::String>) -> Self {
        self.name = name.into();
        self
    }

    /// Add documentation comment
    pub fn doc(mut self, doc: impl Into<std::string::String>) -> Self {
        self.doc = Some(doc.into());
        self
    }

    /// Add a variant
    pub fn variant(mut self, name: impl Into<std::string::String>) -> Self {
        self.variants.push(IREnumVariant {
            name: name.into(),
            source_value: None,
            doc: None,
        });
        self
    }

    /// Add a variant with documentation
    pub fn variant_with_doc(mut self, name: impl Into<std::string::String>, doc: impl Into<std::string::String>) -> Self {
        self.variants.push(IREnumVariant {
            name: name.into(),
            source_value: None,
            doc: Some(doc.into()),
        });
        self
    }

    /// Add a variant with serde rename
    pub fn variant_with_rename(mut self, name: impl Into<std::string::String>, source: impl Into<std::string::String>) -> Self {
        self.variants.push(IREnumVariant {
            name: name.into(),
            source_value: Some(source.into()),
            doc: None,
        });
        self
    }

    /// Add a custom derive
    pub fn derive(mut self, derive: impl Into<std::string::String>) -> Self {
        let d = derive.into();
        if !self.derives.contains(&d) {
            self.derives.push(d);
        }
        self
    }

    /// Add serde derives
    pub fn with_serde(self) -> Self {
        self.derive("serde::Serialize").derive("serde::Deserialize")
    }

    /// Build the IR enum (by reference)
    pub fn build_ir_enum(&self) -> IREnum {
        IREnum {
            name: self.name.clone(),
            variants: self.variants.clone(),
            derives: self.derives.clone(),
            doc: self.doc.clone(),
        }
    }

    /// Consume and build the IR enum
    pub fn into_ir_enum(self) -> IREnum {
        IREnum {
            name: self.name,
            variants: self.variants,
            derives: self.derives,
            doc: self.doc,
        }
    }

    /// Build the IR module containing this enum
    pub fn build_ir_module(self) -> IRModule {
        let enum_name = self.name.clone();
        let ir_enum = self.into_ir_enum();

        let mut module = IRModule::new(enum_name);
        module.add_type(IRType::Enum(ir_enum));
        module
    }

    /// Generate Rust code string
    pub fn generate(self) -> ApiResult<std::string::String> {
        let module = self.build_ir_module();
        render_module(&module)
    }
}

impl Default for EnumGen {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// ModuleGen - Multiple types builder
// ============================================================================

/// Builder for generating a module with multiple structs and enums
///
/// # Examples
///
/// ```ignore
/// use unistructgen_core::api::{ModuleGen, StructGen, EnumGen, FieldType};
///
/// let code = ModuleGen::new("models")
///     .add_struct(StructGen::new()
///         .name("User")
///         .field("id", FieldType::I64)
///         .field("status", FieldType::named("UserStatus"))
///     )
///     .add_enum(EnumGen::new()
///         .name("UserStatus")
///         .variant("Active")
///         .variant("Inactive")
///     )
///     .generate()?;
/// ```
#[derive(Debug, Clone)]
pub struct ModuleGen {
    name: std::string::String,
    types: Vec<IRType>,
}

impl ModuleGen {
    /// Create a new module generator
    pub fn new(name: impl Into<std::string::String>) -> Self {
        Self {
            name: name.into(),
            types: Vec::new(),
        }
    }

    /// Add a struct to the module
    pub fn add_struct(mut self, s: StructGen) -> Self {
        self.types.push(IRType::Struct(s.into_ir_struct()));
        self
    }

    /// Add an enum to the module
    pub fn add_enum(mut self, e: EnumGen) -> Self {
        self.types.push(IRType::Enum(e.into_ir_enum()));
        self
    }

    /// Add a raw IR type
    pub fn add_type(mut self, ty: IRType) -> Self {
        self.types.push(ty);
        self
    }

    /// Build the IR module
    pub fn build(self) -> IRModule {
        IRModule {
            name: self.name,
            types: self.types,
        }
    }

    /// Generate Rust code string
    pub fn generate(self) -> ApiResult<std::string::String> {
        let module = self.build();
        render_module(&module)
    }
}

// ============================================================================
// Render Options
// ============================================================================

/// Options for code rendering
#[derive(Debug, Clone)]
pub struct RenderOptions {
    /// Add header comment with generator info
    pub add_header: bool,
    /// Add clippy allow attributes
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

impl RenderOptions {
    /// Create options without header
    pub fn no_header() -> Self {
        Self {
            add_header: false,
            add_clippy_allows: true,
        }
    }

    /// Create minimal options (no header, no clippy allows)
    pub fn minimal() -> Self {
        Self {
            add_header: false,
            add_clippy_allows: false,
        }
    }
}

// ============================================================================
// Quick Generation Functions
// ============================================================================

/// Quick JSON to Rust struct generation builder
///
/// # Examples
///
/// ```ignore
/// use unistructgen_core::api::from_json;
///
/// let code = from_json(r#"{"id": 1, "name": "Alice"}"#)
///     .struct_name("User")
///     .with_serde()
///     .generate()?;
/// ```
pub fn from_json(json: &str) -> JsonGenBuilder {
    JsonGenBuilder::new(json)
}

/// Builder for JSON-based struct generation
#[derive(Debug, Clone)]
pub struct JsonGenBuilder {
    json: std::string::String,
    struct_name: std::string::String,
    derive_serde: bool,
    derive_default: bool,
    make_optional: bool,
}

impl JsonGenBuilder {
    /// Create a new JSON generation builder
    pub fn new(json: &str) -> Self {
        Self {
            json: json.to_string(),
            struct_name: "Root".to_string(),
            derive_serde: false,
            derive_default: false,
            make_optional: false,
        }
    }

    /// Set the struct name
    pub fn struct_name(mut self, name: impl Into<std::string::String>) -> Self {
        self.struct_name = name.into();
        self
    }

    /// Enable serde derives
    pub fn with_serde(mut self) -> Self {
        self.derive_serde = true;
        self
    }

    /// Enable Default derive
    pub fn with_default(mut self) -> Self {
        self.derive_default = true;
        self
    }

    /// Make all fields optional
    pub fn make_optional(mut self) -> Self {
        self.make_optional = true;
        self
    }

    /// Build the IR module (requires json_parser feature)
    #[cfg(feature = "json_parser")]
    pub fn build_module(self) -> ApiResult<IRModule> {
        use unistructgen_json_parser::{JsonParser, ParserOptions};
        use crate::Parser;

        let mut parser = JsonParser::new(ParserOptions {
            struct_name: self.struct_name,
            derive_serde: self.derive_serde,
            derive_default: self.derive_default,
            make_fields_optional: self.make_optional,
        });

        parser.parse(&self.json)
            .map_err(|e| ApiError::Parse(e.to_string()))
    }

    /// Generate Rust code (requires json_parser feature)
    #[cfg(feature = "json_parser")]
    pub fn generate(self) -> ApiResult<std::string::String> {
        let module = self.build_module()?;
        render_module(&module)
    }

    /// Get configuration for external parser
    pub fn get_config(&self) -> JsonParserConfig {
        JsonParserConfig {
            struct_name: self.struct_name.clone(),
            derive_serde: self.derive_serde,
            derive_default: self.derive_default,
            make_optional: self.make_optional,
        }
    }
}

/// Configuration for JSON parser
#[derive(Debug, Clone)]
pub struct JsonParserConfig {
    pub struct_name: std::string::String,
    pub derive_serde: bool,
    pub derive_default: bool,
    pub make_optional: bool,
}

// ============================================================================
// Rendering utilities
// ============================================================================

/// Render an IR module to Rust code
pub fn render_module(module: &IRModule) -> ApiResult<std::string::String> {
    render_module_with_options(module, RenderOptions::default())
}

/// Render an IR module with custom options
pub fn render_module_with_options(module: &IRModule, options: RenderOptions) -> ApiResult<std::string::String> {
    use std::fmt::Write;

    let mut output = std::string::String::new();

    if options.add_header {
        writeln!(output, "// Generated by unistructgen").map_err(|e| ApiError::Generation(e.to_string()))?;
        writeln!(output, "// Do not edit this file manually").map_err(|e| ApiError::Generation(e.to_string()))?;
        writeln!(output).map_err(|e| ApiError::Generation(e.to_string()))?;
    }

    if options.add_clippy_allows {
        writeln!(output, "#![allow(dead_code)]").map_err(|e| ApiError::Generation(e.to_string()))?;
        writeln!(output, "#![allow(unused_imports)]").map_err(|e| ApiError::Generation(e.to_string()))?;
        writeln!(output).map_err(|e| ApiError::Generation(e.to_string()))?;
    }

    for ty in &module.types {
        match ty {
            IRType::Struct(s) => {
                render_struct(&mut output, s)?;
                writeln!(output).map_err(|e| ApiError::Generation(e.to_string()))?;
            }
            IRType::Enum(e) => {
                render_enum(&mut output, e)?;
                writeln!(output).map_err(|e| ApiError::Generation(e.to_string()))?;
            }
        }
    }

    Ok(output)
}

fn render_struct(output: &mut std::string::String, s: &IRStruct) -> ApiResult<()> {
    use std::fmt::Write;

    // Doc comment
    if let Some(doc) = &s.doc {
        writeln!(output, "/// {}", doc).map_err(|e| ApiError::Generation(e.to_string()))?;
    }

    // Derives
    if !s.derives.is_empty() {
        write!(output, "#[derive(").map_err(|e| ApiError::Generation(e.to_string()))?;
        for (i, derive) in s.derives.iter().enumerate() {
            if i > 0 {
                write!(output, ", ").map_err(|e| ApiError::Generation(e.to_string()))?;
            }
            write!(output, "{}", derive).map_err(|e| ApiError::Generation(e.to_string()))?;
        }
        writeln!(output, ")]").map_err(|e| ApiError::Generation(e.to_string()))?;
    }

    // Attributes
    for attr in &s.attributes {
        writeln!(output, "#[{}]", attr).map_err(|e| ApiError::Generation(e.to_string()))?;
    }

    writeln!(output, "pub struct {} {{", s.name).map_err(|e| ApiError::Generation(e.to_string()))?;

    for field in &s.fields {
        render_field(output, field)?;
    }

    writeln!(output, "}}").map_err(|e| ApiError::Generation(e.to_string()))?;

    Ok(())
}

fn render_field(output: &mut std::string::String, field: &IRField) -> ApiResult<()> {
    use std::fmt::Write;

    // Doc comment
    if let Some(doc) = &field.doc {
        writeln!(output, "    /// {}", doc).map_err(|e| ApiError::Generation(e.to_string()))?;
    }

    // Attributes
    for attr in &field.attributes {
        writeln!(output, "    #[{}]", attr).map_err(|e| ApiError::Generation(e.to_string()))?;
    }

    // Validation attributes
    let validation_attrs = generate_validation_attrs(&field.constraints);
    if !validation_attrs.is_empty() {
        writeln!(output, "    #[validate({})]", validation_attrs.join(", "))
            .map_err(|e| ApiError::Generation(e.to_string()))?;
    }

    writeln!(output, "    pub {}: {},", field.name, render_type(&field.ty))
        .map_err(|e| ApiError::Generation(e.to_string()))?;

    Ok(())
}

fn render_enum(output: &mut std::string::String, e: &IREnum) -> ApiResult<()> {
    use std::fmt::Write;

    // Doc comment
    if let Some(doc) = &e.doc {
        writeln!(output, "/// {}", doc).map_err(|e| ApiError::Generation(e.to_string()))?;
    }

    // Derives
    if !e.derives.is_empty() {
        write!(output, "#[derive(").map_err(|err| ApiError::Generation(err.to_string()))?;
        for (i, derive) in e.derives.iter().enumerate() {
            if i > 0 {
                write!(output, ", ").map_err(|err| ApiError::Generation(err.to_string()))?;
            }
            write!(output, "{}", derive).map_err(|err| ApiError::Generation(err.to_string()))?;
        }
        writeln!(output, ")]").map_err(|err| ApiError::Generation(err.to_string()))?;
    }

    writeln!(output, "pub enum {} {{", e.name).map_err(|err| ApiError::Generation(err.to_string()))?;

    for variant in &e.variants {
        if let Some(doc) = &variant.doc {
            writeln!(output, "    /// {}", doc).map_err(|err| ApiError::Generation(err.to_string()))?;
        }
        if let Some(source) = &variant.source_value {
            writeln!(output, "    #[serde(rename = \"{}\")]", source).map_err(|err| ApiError::Generation(err.to_string()))?;
        }
        writeln!(output, "    {},", variant.name).map_err(|err| ApiError::Generation(err.to_string()))?;
    }

    writeln!(output, "}}").map_err(|err| ApiError::Generation(err.to_string()))?;

    Ok(())
}

fn render_type(ty: &IRTypeRef) -> std::string::String {
    match ty {
        IRTypeRef::Primitive(p) => p.rust_type_name().to_string(),
        IRTypeRef::Option(inner) => format!("Option<{}>", render_type(inner)),
        IRTypeRef::Vec(inner) => format!("Vec<{}>", render_type(inner)),
        IRTypeRef::Named(name) => name.clone(),
        IRTypeRef::Map(k, v) => format!("std::collections::HashMap<{}, {}>", render_type(k), render_type(v)),
    }
}

fn generate_validation_attrs(constraints: &FieldConstraints) -> Vec<std::string::String> {
    let mut attrs = Vec::new();

    if constraints.min_length.is_some() || constraints.max_length.is_some() {
        let mut parts = Vec::new();
        if let Some(min) = constraints.min_length {
            parts.push(format!("min = {}", min));
        }
        if let Some(max) = constraints.max_length {
            parts.push(format!("max = {}", max));
        }
        attrs.push(format!("length({})", parts.join(", ")));
    }

    if constraints.min_value.is_some() || constraints.max_value.is_some() {
        let mut parts = Vec::new();
        if let Some(min) = constraints.min_value {
            if min.fract() == 0.0 {
                parts.push(format!("min = {}", min as i64));
            } else {
                parts.push(format!("min = {}", min));
            }
        }
        if let Some(max) = constraints.max_value {
            if max.fract() == 0.0 {
                parts.push(format!("max = {}", max as i64));
            } else {
                parts.push(format!("max = {}", max));
            }
        }
        attrs.push(format!("range({})", parts.join(", ")));
    }

    if let Some(pattern) = &constraints.pattern {
        attrs.push(format!("regex = \"{}\"", pattern.replace('\"', "\\\"")));
    }

    if let Some(format) = &constraints.format {
        match format.as_str() {
            "email" => attrs.push("email".to_string()),
            "url" => attrs.push("url".to_string()),
            _ => {}
        }
    }

    attrs
}

// ============================================================================
// Prelude - Common imports
// ============================================================================

/// Prelude module with commonly used types
pub mod prelude {
    pub use super::{
        StructGen, EnumGen, ModuleGen,
        FieldBuilder, FieldType,
        RenderOptions,
        ApiResult, ApiError,
        from_json, JsonGenBuilder,
        render_module, render_module_with_options,
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_struct_gen_basic() {
        let code = StructGen::new()
            .name("User")
            .field("id", FieldType::I64)
            .field("name", FieldType::String)
            .generate()
            .unwrap();

        assert!(code.contains("pub struct User"));
        assert!(code.contains("pub id: i64"));
        assert!(code.contains("pub name: String"));
    }

    #[test]
    fn test_struct_gen_with_serde() {
        let code = StructGen::new()
            .name("User")
            .field("id", FieldType::I64)
            .with_serde()
            .generate()
            .unwrap();

        assert!(code.contains("serde::Serialize"));
        assert!(code.contains("serde::Deserialize"));
    }

    #[test]
    fn test_struct_gen_optional_field() {
        let code = StructGen::new()
            .name("User")
            .field_optional("email", FieldType::String)
            .generate()
            .unwrap();

        assert!(code.contains("pub email: Option<String>"));
    }

    #[test]
    fn test_enum_gen_basic() {
        let code = EnumGen::new()
            .name("Status")
            .variant("Active")
            .variant("Inactive")
            .generate()
            .unwrap();

        assert!(code.contains("pub enum Status"));
        assert!(code.contains("Active,"));
        assert!(code.contains("Inactive,"));
    }

    #[test]
    fn test_field_builder_with_constraints() {
        let code = StructGen::new()
            .name("User")
            .field_with(
                |f| f.doc("User's age").range(0.0, 150.0),
                "age",
                FieldType::I32
            )
            .generate()
            .unwrap();

        assert!(code.contains("/// User's age"));
        assert!(code.contains("#[validate(range(min = 0, max = 150))]"));
    }

    #[test]
    fn test_module_gen() {
        let code = ModuleGen::new("models")
            .add_struct(StructGen::new()
                .name("User")
                .field("id", FieldType::I64))
            .add_enum(EnumGen::new()
                .name("Status")
                .variant("Active"))
            .generate()
            .unwrap();

        assert!(code.contains("pub struct User"));
        assert!(code.contains("pub enum Status"));
    }

    #[test]
    fn test_render_options_minimal() {
        let code = StructGen::new()
            .name("Test")
            .field("x", FieldType::I32)
            .generate_with_options(RenderOptions::minimal())
            .unwrap();

        assert!(!code.contains("Generated by"));
        assert!(!code.contains("#![allow"));
    }
}
