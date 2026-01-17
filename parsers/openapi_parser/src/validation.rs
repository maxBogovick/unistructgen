//! Validation constraint extraction from OpenAPI schemas

use openapiv3::{Schema, SchemaKind, Type};
use unistructgen_core::FieldConstraints;

/// Extract validation constraints from an OpenAPI schema
pub fn extract_validation_constraints(schema: &Schema) -> FieldConstraints {
    let mut constraints = FieldConstraints::default();

    match &schema.schema_kind {
        SchemaKind::Type(Type::String(string_type)) => {
            // String length constraints
            if let Some(min_len) = string_type.min_length {
                constraints.min_length = Some(min_len);
            }
            if let Some(max_len) = string_type.max_length {
                constraints.max_length = Some(max_len);
            }

            // Pattern constraint
            if let Some(pattern) = &string_type.pattern {
                constraints.pattern = Some(pattern.clone());
            }

            // Format constraint - now not optional
            constraints.format = Some(format!("{:?}", string_type.format));
        }

        SchemaKind::Type(Type::Number(number_type)) => {
            // Numeric range constraints
            if let Some(min) = number_type.minimum {
                constraints.min_value = Some(min);
            }
            if let Some(max) = number_type.maximum {
                constraints.max_value = Some(max);
            }
        }

        SchemaKind::Type(Type::Integer(int_type)) => {
            // Integer range constraints
            if let Some(min) = int_type.minimum {
                constraints.min_value = Some(min as f64);
            }
            if let Some(max) = int_type.maximum {
                constraints.max_value = Some(max as f64);
            }
        }

        SchemaKind::Type(Type::Array(array_type)) => {
            // Array length constraints
            if let Some(min_items) = array_type.min_items {
                constraints.min_length = Some(min_items);
            }
            if let Some(max_items) = array_type.max_items {
                constraints.max_length = Some(max_items);
            }
        }

        _ => {}
    }

    constraints
}

/// Generate validator attribute string from constraints
pub fn generate_validator_attribute(constraints: &FieldConstraints) -> Option<String> {
    let mut attrs = Vec::new();

    // Length validation
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

    // Range validation
    if constraints.min_value.is_some() || constraints.max_value.is_some() {
        let mut range_parts = Vec::new();
        if let Some(min) = constraints.min_value {
            range_parts.push(format!("min = {}", min));
        }
        if let Some(max) = constraints.max_value {
            range_parts.push(format!("max = {}", max));
        }
        attrs.push(format!("range({})", range_parts.join(", ")));
    }

    // Format validation
    if let Some(format) = &constraints.format {
        match format.as_str() {
            "email" => attrs.push("email".to_string()),
            "url" | "uri" => attrs.push("url".to_string()),
            _ => {}
        }
    }

    // Pattern validation
    if let Some(pattern) = &constraints.pattern {
        attrs.push(format!("regex = \"{}\"", pattern));
    }

    if attrs.is_empty() {
        None
    } else {
        Some(format!("#[validate({})]", attrs.join(", ")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_validator_attribute() {
        let mut constraints = FieldConstraints::default();
        constraints.min_length = Some(5);
        constraints.max_length = Some(50);

        let attr = generate_validator_attribute(&constraints);
        assert_eq!(attr, Some("#[validate(length(min = 5, max = 50))]".to_string()));
    }

    #[test]
    fn test_generate_validator_attribute_email() {
        let mut constraints = FieldConstraints::default();
        constraints.format = Some("email".to_string());

        let attr = generate_validator_attribute(&constraints);
        assert_eq!(attr, Some("#[validate(email)]".to_string()));
    }

    #[test]
    fn test_generate_validator_attribute_range() {
        let mut constraints = FieldConstraints::default();
        constraints.min_value = Some(0.0);
        constraints.max_value = Some(150.0);

        let attr = generate_validator_attribute(&constraints);
        assert_eq!(attr, Some("#[validate(range(min = 0, max = 150))]".to_string()));
    }
}
