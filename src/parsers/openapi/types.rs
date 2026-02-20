//! Type conversion utilities for OpenAPI schemas

use crate::parsers::openapi::error::{OpenApiError, Result};
use openapiv3::{Schema, SchemaKind, Type};
use crate::core::{IRTypeRef, PrimitiveKind};

/// Convert OpenAPI type to IR type reference
pub fn openapi_type_to_ir(
    schema: &Schema,
    type_name_hint: Option<&str>,
) -> Result<IRTypeRef> {
    match &schema.schema_kind {
        SchemaKind::Type(Type::String(string_type)) => {
            // Check for format-specific types
            let format_str = format!("{:?}", string_type.format);
            if format_str.contains("DateTime") {
                Ok(IRTypeRef::Primitive(PrimitiveKind::DateTime))
            } else if format_str.contains("Uuid") {
                Ok(IRTypeRef::Primitive(PrimitiveKind::Uuid))
            } else if !string_type.enumeration.is_empty() {
                // Enum type - will be handled by schema converter
                if let Some(name) = type_name_hint {
                    Ok(IRTypeRef::Named(to_pascal_case(name)))
                } else {
                    Ok(IRTypeRef::Primitive(PrimitiveKind::String))
                }
            } else {
                Ok(IRTypeRef::Primitive(PrimitiveKind::String))
            }
        }

        SchemaKind::Type(Type::Number(number_type)) => {
            let format_str = format!("{:?}", number_type.format);
            if format_str.contains("Float") {
                Ok(IRTypeRef::Primitive(PrimitiveKind::F32))
            } else {
                Ok(IRTypeRef::Primitive(PrimitiveKind::F64))
            }
        }

        SchemaKind::Type(Type::Integer(int_type)) => {
            let format_str = format!("{:?}", int_type.format);
            if format_str.contains("Int32") {
                Ok(IRTypeRef::Primitive(PrimitiveKind::I32))
            } else {
                Ok(IRTypeRef::Primitive(PrimitiveKind::I64))
            }
        }

        SchemaKind::Type(Type::Boolean(_)) => {
            Ok(IRTypeRef::Primitive(PrimitiveKind::Bool))
        }

        SchemaKind::Type(Type::Array(array_type)) => {
            if let Some(ref items) = array_type.items {
                let item_type = match items {
                    openapiv3::ReferenceOr::Reference { reference } => {
                        // Extract type name from reference
                        let type_name = extract_type_name_from_ref(&reference);
                        IRTypeRef::Named(type_name)
                    }
                    openapiv3::ReferenceOr::Item(schema) => {
                        openapi_type_to_ir(schema.as_ref(), type_name_hint)?
                    }
                };
                Ok(IRTypeRef::Vec(Box::new(item_type)))
            } else {
                // Array without items - use generic JSON value
                Ok(IRTypeRef::Vec(Box::new(IRTypeRef::Primitive(
                    PrimitiveKind::Json,
                ))))
            }
        }

        SchemaKind::Type(Type::Object(obj_type)) => {
            // Check if it's a map/dictionary
            if obj_type.properties.is_empty() && obj_type.additional_properties.is_some() {
                // This is a map
                let value_type = match obj_type.additional_properties.as_ref().unwrap() {
                    openapiv3::AdditionalProperties::Any(true) => {
                        IRTypeRef::Primitive(PrimitiveKind::Json)
                    }
                    openapiv3::AdditionalProperties::Schema(schema_ref) => {
                        match schema_ref.as_ref() {
                            openapiv3::ReferenceOr::Reference { reference } => {
                                IRTypeRef::Named(extract_type_name_from_ref(reference))
                            }
                            openapiv3::ReferenceOr::Item(schema) => {
                                openapi_type_to_ir(schema, None)?
                            }
                        }
                    }
                    _ => IRTypeRef::Primitive(PrimitiveKind::Json),
                };

                Ok(IRTypeRef::Map(
                    Box::new(IRTypeRef::Primitive(PrimitiveKind::String)),
                    Box::new(value_type),
                ))
            } else if let Some(name) = type_name_hint {
                // Named object type
                Ok(IRTypeRef::Named(to_pascal_case(name)))
            } else {
                // Anonymous object - use JSON value
                Ok(IRTypeRef::Primitive(PrimitiveKind::Json))
            }
        }

        SchemaKind::OneOf { .. } | SchemaKind::AnyOf { .. } | SchemaKind::AllOf { .. } => {
            // Schema composition - will be handled specially
            if let Some(name) = type_name_hint {
                Ok(IRTypeRef::Named(to_pascal_case(name)))
            } else {
                Ok(IRTypeRef::Primitive(PrimitiveKind::Json))
            }
        }

        SchemaKind::Any(_) => {
            // Untyped schema - use JSON value
            Ok(IRTypeRef::Primitive(PrimitiveKind::Json))
        }

        SchemaKind::Not { .. } => {
            Err(OpenApiError::unsupported_type("not schemas are not supported"))
        }
    }
}

/// Extract type name from OpenAPI reference
/// Example: "#/components/schemas/User" -> "User"
pub fn extract_type_name_from_ref(reference: &str) -> String {
    reference
        .rsplit('/')
        .next()
        .unwrap_or(reference)
        .to_string()
}

/// Convert snake_case or kebab-case to PascalCase
pub fn to_pascal_case(s: &str) -> String {
    s.split(|c| c == '_' || c == '-' || c == ' ')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().chain(chars).collect(),
                None => String::new(),
            }
        })
        .collect()
}

/// Convert PascalCase or camelCase to snake_case
pub fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    let chars: Vec<char> = s.chars().collect();

    for (i, &c) in chars.iter().enumerate() {
        if c.is_uppercase() {
            // Add underscore if:
            // 1. Not the first character AND
            // 2. Previous char is lowercase OR
            // 3. Next char exists and is lowercase (for handling acronyms like "APIKey" -> "api_key")
            if i > 0 {
                let prev_is_lower = chars.get(i - 1).map_or(false, |c| c.is_lowercase());
                let next_is_lower = chars.get(i + 1).map_or(false, |c| c.is_lowercase());

                if prev_is_lower || next_is_lower {
                    result.push('_');
                }
            }
            result.push(c.to_lowercase().next().unwrap());
        } else {
            result.push(c);
        }
    }

    result
}

/// Sanitize field name to be a valid Rust identifier
pub fn sanitize_field_name(name: &str) -> String {
    // Convert to snake_case first
    let snake = to_snake_case(name);

    // Replace invalid characters
    let sanitized: String = snake
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();

    // Check if it starts with a number
    if sanitized.chars().next().map_or(false, |c| c.is_numeric()) {
        format!("_{}", sanitized)
    } else if is_rust_keyword(&sanitized) {
        // Append underscore to Rust keywords
        format!("{}_", sanitized)
    } else {
        sanitized
    }
}

/// Check if a string is a Rust keyword
pub fn is_rust_keyword(s: &str) -> bool {
    matches!(
        s,
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
            | "abstract"
            | "become"
            | "box"
            | "do"
            | "final"
            | "macro"
            | "override"
            | "priv"
            | "typeof"
            | "unsized"
            | "virtual"
            | "yield"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_pascal_case() {
        assert_eq!(to_pascal_case("user_profile"), "UserProfile");
        assert_eq!(to_pascal_case("api-key"), "ApiKey");
        assert_eq!(to_pascal_case("simple"), "Simple");
        assert_eq!(to_pascal_case("my_long_type_name"), "MyLongTypeName");
    }

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("UserProfile"), "user_profile");
        assert_eq!(to_snake_case("APIKey"), "api_key");
        assert_eq!(to_snake_case("simple"), "simple");
        assert_eq!(to_snake_case("myLongTypeName"), "my_long_type_name");
    }

    #[test]
    fn test_sanitize_field_name() {
        assert_eq!(sanitize_field_name("type"), "type_");
        assert_eq!(sanitize_field_name("123field"), "_123field");
        assert_eq!(sanitize_field_name("user-name"), "user_name");
        assert_eq!(sanitize_field_name("valid_field"), "valid_field");
    }

    #[test]
    fn test_extract_type_name_from_ref() {
        assert_eq!(
            extract_type_name_from_ref("#/components/schemas/User"),
            "User"
        );
        assert_eq!(
            extract_type_name_from_ref("#/components/schemas/ApiKey"),
            "ApiKey"
        );
        assert_eq!(extract_type_name_from_ref("User"), "User");
    }

    #[test]
    fn test_is_rust_keyword() {
        assert!(is_rust_keyword("type"));
        assert!(is_rust_keyword("async"));
        assert!(is_rust_keyword("await"));
        assert!(!is_rust_keyword("user"));
        assert!(!is_rust_keyword("field"));
    }
}
