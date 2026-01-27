use serde_json::{json, Map, Value};
use unistructgen_core::{
    CodeGenerator, GeneratorMetadata, IRField, IRModule, IRStruct, IRType, IRTypeRef, PrimitiveKind,
    IREnum,
};
use thiserror::Error;

/// Errors that can occur during JSON Schema generation
#[derive(Debug, Error)]
pub enum JsonSchemaError {
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Unsupported type for JSON Schema: {0}")]
    UnsupportedType(String),
}

/// Generator for JSON Schema (Draft 2020-12)
///
/// This generator produces a JSON Schema compatible with OpenAI's `response_format`
/// and other AI tools that require structured output validation.
///
/// # Example
///
/// ```ignore
/// use unistructgen_codegen::JsonSchemaRenderer;
/// use unistructgen_core::CodeGenerator;
///
/// let generator = JsonSchemaRenderer::default();
/// let schema_json = generator.generate(&ir_module)?;
/// ```
#[derive(Debug, Default)]
pub struct JsonSchemaRenderer {
    /// If true, the generated schema will not include the $schema keyword,
    /// making it suitable for embedding in other schemas.
    pub fragment_mode: bool,
}

impl JsonSchemaRenderer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable fragment mode (no $schema, no generic wrapper if possible)
    pub fn fragment(mut self) -> Self {
        self.fragment_mode = true;
        self
    }
}

impl CodeGenerator for JsonSchemaRenderer {
    type Error = JsonSchemaError;

    fn generate(&self, module: &IRModule) -> Result<String, Self::Error> {
        let mut defs = Map::new();
        let mut root_ref = None;

        // Process all types in the module to build definitions
        for ty in &module.types {
            let (name, schema) = match ty {
                IRType::Struct(s) => (s.name.clone(), render_struct(s)?),
                IRType::Enum(e) => (e.name.clone(), render_enum(e)?),
            };

            defs.insert(name.clone(), schema);
            
            // Heuristic: The last defined type or a type matching the module name is often the root
            // For now, we'll just track the last one, but ideally the IRModule would specify a root.
            root_ref = Some(name);
        }

        // If the module name matches a type, that's definitely the root
        if defs.contains_key(&module.name) {
            root_ref = Some(module.name.clone());
        }

        let mut root_schema = Map::new();
        
        if !self.fragment_mode {
            root_schema.insert("$schema".to_string(), json!("https://json-schema.org/draft/2020-12/schema"));
        }

        root_schema.insert("$defs".to_string(), Value::Object(defs));

        // If we identified a root type, point the main schema to it
        if let Some(root_name) = root_ref {
            root_schema.insert("$ref".to_string(), json!(format!("#/$defs/{}", root_name)));
        } else {
            // Fallback if no types: empty object
            root_schema.insert("type".to_string(), json!("object"));
        }

        Ok(serde_json::to_string_pretty(&root_schema)?)
    }

    fn language(&self) -> &'static str {
        "JSON Schema"
    }

    fn file_extension(&self) -> &str {
        "json"
    }

    fn metadata(&self) -> GeneratorMetadata {
        GeneratorMetadata::new()
            .with_version("0.1.0")
            .with_description("Generates JSON Schema (Draft 2020-12) for AI Structured Outputs")
            .with_feature("recursive-types")
            .with_feature("validation")
    }
}

fn render_struct(s: &IRStruct) -> Result<Value, JsonSchemaError> {
    let mut schema = Map::new();
    schema.insert("type".to_string(), json!("object"));
    schema.insert("additionalProperties".to_string(), json!(false));
    
    if let Some(doc) = &s.doc {
        schema.insert("description".to_string(), json!(doc));
    }

    let mut properties = Map::new();
    let mut required = Vec::new();

    for field in &s.fields {
        let field_name = field.source_name.as_ref().unwrap_or(&field.name).clone();
        let field_schema = render_field(field)?;
        
        properties.insert(field_name.clone(), field_schema);

        if !field.optional {
            required.push(json!(field_name));
        }
    }

    schema.insert("properties".to_string(), Value::Object(properties));
    
    if !required.is_empty() {
        schema.insert("required".to_string(), Value::Array(required));
    }

    Ok(Value::Object(schema))
}

fn render_enum(e: &IREnum) -> Result<Value, JsonSchemaError> {
    // Check if it's a simple string enum (no fields in variants)
    // IR currently doesn't strictly distinguish variant types in the struct def, 
    // but typical JSON enums are just strings.
    // If future IR supports complex enums (sum types), this needs 'oneOf'.
    
    // Assuming simple string enum for now as per current IR usage
    let mut schema = Map::new();
    schema.insert("type".to_string(), json!("string"));
    
    if let Some(doc) = &e.doc {
        schema.insert("description".to_string(), json!(doc));
    }

    let mut enum_values = Vec::new();
    for variant in &e.variants {
        let val = variant.source_value.as_ref().unwrap_or(&variant.name).clone();
        enum_values.push(json!(val));
    }
    
    schema.insert("enum".to_string(), Value::Array(enum_values));

    Ok(Value::Object(schema))
}

fn render_field(field: &IRField) -> Result<Value, JsonSchemaError> {
    let mut schema = render_type_ref(&field.ty)?;
    
    // Add field-specific documentation and constraints if it's a simple type object
    if let Some(obj) = schema.as_object_mut() {
        if let Some(doc) = &field.doc {
            obj.insert("description".to_string(), json!(doc));
        }
        
        // Apply constraints
        if let Some(min) = field.constraints.min_length {
            obj.insert("minLength".to_string(), json!(min));
        }
        if let Some(max) = field.constraints.max_length {
            obj.insert("maxLength".to_string(), json!(max));
        }
        if let Some(min) = field.constraints.min_value {
            obj.insert("minimum".to_string(), json!(min));
        }
        if let Some(max) = field.constraints.max_value {
            obj.insert("maximum".to_string(), json!(max));
        }
        if let Some(pattern) = &field.constraints.pattern {
            obj.insert("pattern".to_string(), json!(pattern));
        }
        if let Some(format) = &field.constraints.format {
            obj.insert("format".to_string(), json!(format));
        }
    }

    Ok(schema)
}

fn render_type_ref(ty: &IRTypeRef) -> Result<Value, JsonSchemaError> {
    match ty {
        IRTypeRef::Primitive(p) => render_primitive(p),
        IRTypeRef::Named(name) => {
            Ok(json!({ "$ref": format!("#/$defs/{}", name) }))
        }
        IRTypeRef::Option(inner) => {
            // For JSON schema, optionality is usually handled by 'required'.
            // However, explicit nullability can be defined.
            // We'll return the inner type, and let the struct renderer handle 'required'.
            // If explicit null is needed:
            // let mut inner_schema = render_type_ref(inner)?;
            // ... logic to add "null" to type ...
            render_type_ref(inner)
        }
        IRTypeRef::Vec(inner) => {
            Ok(json!({
                "type": "array",
                "items": render_type_ref(inner)?
            }))
        }
        IRTypeRef::Map(_key, value) => {
            // JSON keys are always strings
            Ok(json!({
                "type": "object",
                "additionalProperties": render_type_ref(value)?
            }))
        }
    }
}

fn render_primitive(p: &PrimitiveKind) -> Result<Value, JsonSchemaError> {
    let (ty, format) = match p {
        PrimitiveKind::String => ("string", None),
        PrimitiveKind::I32 | PrimitiveKind::I64 | PrimitiveKind::I128 |
        PrimitiveKind::U32 | PrimitiveKind::U64 | PrimitiveKind::U128 => ("integer", None),
        PrimitiveKind::I8 | PrimitiveKind::I16 |
        PrimitiveKind::U8 | PrimitiveKind::U16 => ("integer", None),
        PrimitiveKind::F32 | PrimitiveKind::F64 | PrimitiveKind::Decimal => ("number", None),
        PrimitiveKind::Bool => ("boolean", None),
        PrimitiveKind::Char => ("string", Some("char")), // Custom format
        PrimitiveKind::DateTime => ("string", Some("date-time")),
        PrimitiveKind::Uuid => ("string", Some("uuid")),
        PrimitiveKind::Json => ("object", None), // Generic object
    };

    let mut map = Map::new();
    map.insert("type".to_string(), json!(ty));
    if let Some(fmt) = format {
        map.insert("format".to_string(), json!(fmt));
    }
    
    Ok(Value::Object(map))
}
