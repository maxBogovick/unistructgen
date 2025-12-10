use serde_json::Value;
use std::collections::HashSet;
use thiserror::Error;
use unistructgen_core::{
    IRField, IRModule, IRStruct, IRType, IRTypeRef, PrimitiveKind,
};

#[derive(Error, Debug)]
pub enum JsonParserError {
    #[error("JSON parse error: {0}")]
    ParseError(#[from] serde_json::Error),

    #[error("Invalid JSON structure: {0}")]
    InvalidStructure(String),

    #[error("Type inference failed: {0}")]
    TypeInferenceFailed(String),
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

pub struct JsonParser {
    options: ParserOptions,
    /// Generated type names to avoid collisions
    type_names: HashSet<String>,
    /// Accumulated types (nested structs)
    accumulated_types: Vec<IRStruct>,
}

impl JsonParser {
    pub fn new(options: ParserOptions) -> Self {
        Self {
            options,
            type_names: HashSet::new(),
            accumulated_types: Vec::new(),
        }
    }

    pub fn parse(&mut self, input: &str) -> Result<IRModule> {
        let value: Value = serde_json::from_str(input)?;

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
                return Err(JsonParserError::InvalidStructure(
                    "Root JSON must be an object, not an array".to_string(),
                ));
            }
            _ => {
                return Err(JsonParserError::InvalidStructure(
                    "Root JSON must be an object".to_string(),
                ));
            }
        }

        Ok(module)
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

        if let IRType::Struct(s) = &result.types[0] {
            assert_eq!(s.name, "User");
            assert_eq!(s.fields.len(), 3);
        } else {
            panic!("Expected struct");
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
