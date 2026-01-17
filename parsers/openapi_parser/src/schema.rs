//! Schema conversion from OpenAPI to IR

use crate::error::{OpenApiError, Result};
use crate::options::OpenApiParserOptions;
use crate::types::{
    extract_type_name_from_ref, openapi_type_to_ir, sanitize_field_name, to_pascal_case,
};
use crate::validation::extract_validation_constraints;
use openapiv3::{
    OpenAPI, ReferenceOr, Schema, SchemaKind, Type,
};
use std::collections::HashSet;
use unistructgen_core::{
    IREnum, IREnumVariant, IRField, IRStruct, IRType, IRTypeRef, PrimitiveKind,
};

/// Schema converter that maintains context and handles references
pub struct SchemaConverter<'a> {
    /// The OpenAPI specification
    spec: &'a OpenAPI,
    /// Parser options
    options: &'a OpenApiParserOptions,
    /// Generated type names to avoid duplicates
    generated_types: HashSet<String>,
    /// Current depth for recursion prevention
    current_depth: usize,
    /// Reference resolution stack for cycle detection
    reference_stack: Vec<String>,
    /// Inline enum types generated from properties
    inline_enum_types: Vec<IRType>,
}

impl<'a> SchemaConverter<'a> {
    /// Create a new schema converter
    pub fn new(spec: &'a OpenAPI, options: &'a OpenApiParserOptions) -> Self {
        Self {
            spec,
            options,
            generated_types: HashSet::new(),
            current_depth: 0,
            reference_stack: Vec::new(),
            inline_enum_types: Vec::new(),
        }
    }

    /// Convert all schemas from components to IR types
    pub fn convert_all_schemas(&mut self) -> Result<Vec<IRType>> {
        let mut types = Vec::new();

        if let Some(components) = &self.spec.components {
            for (name, schema_ref) in &components.schemas {
                let schema = match schema_ref {
                    ReferenceOr::Item(schema) => schema,
                    ReferenceOr::Reference { .. } => {
                        // Skip references at the top level
                        continue;
                    }
                };

                let ir_type = self.convert_schema(name, schema)?;
                if let Some(ty) = ir_type {
                    types.push(ty);
                }
            }
        }

        // Add all inline enum types generated from properties
        types.extend(self.inline_enum_types.drain(..));

        Ok(types)
    }

    /// Convert a single schema to IR type
    pub fn convert_schema(&mut self, name: &str, schema: &Schema) -> Result<Option<IRType>> {
        // Check depth limit
        if self.current_depth >= self.options.max_depth {
            return Err(OpenApiError::invalid_spec(format!(
                "Maximum schema depth ({}) exceeded for '{}'",
                self.options.max_depth, name
            )));
        }

        self.current_depth += 1;
        let result = self.convert_schema_impl(name, schema);
        self.current_depth -= 1;

        result
    }

    fn convert_schema_impl(&mut self, name: &str, schema: &Schema) -> Result<Option<IRType>> {
        match &schema.schema_kind {
            SchemaKind::Type(Type::Object(obj_type)) => {
                let struct_name = self.options.format_type_name(&to_pascal_case(name));

                // Check if already generated
                if self.generated_types.contains(&struct_name) {
                    return Ok(None);
                }
                self.generated_types.insert(struct_name.clone());

                let mut ir_struct = IRStruct::new(struct_name);

                // Add documentation
                if self.options.generate_docs {
                    if let Some(desc) = &schema.schema_data.description {
                        ir_struct.doc = Some(desc.clone());
                    }
                }

                // Add derives
                if self.options.derive_serde {
                    ir_struct.add_derive("serde::Serialize".to_string());
                    ir_struct.add_derive("serde::Deserialize".to_string());
                }
                if self.options.derive_default {
                    ir_struct.add_derive("Default".to_string());
                }
                if self.options.generate_validation {
                    ir_struct.add_derive("validator::Validate".to_string());
                }

                // Convert properties to fields
                let required_fields: HashSet<_> =
                    obj_type.required.iter().map(|s| s.as_str()).collect();

                for (field_name, property_ref) in &obj_type.properties {
                    let property = match property_ref {
                        ReferenceOr::Item(schema) => schema,
                        ReferenceOr::Reference { reference: _ } => {
                            // We need to resolve this reference manually
                            // For now, skip or use a simplified approach
                            continue;
                        }
                    };
                    let field =
                        self.convert_property(field_name, property, &required_fields)?;
                    ir_struct.add_field(field);
                }

                Ok(Some(IRType::Struct(ir_struct)))
            }

            SchemaKind::Type(Type::String(string_type)) if !string_type.enumeration.is_empty() => {
                // Enum type
                let enum_name = self.options.format_type_name(&to_pascal_case(name));

                if self.generated_types.contains(&enum_name) {
                    return Ok(None);
                }
                self.generated_types.insert(enum_name.clone());

                let mut ir_enum = IREnum {
                    name: enum_name,
                    variants: Vec::new(),
                    derives: vec![
                        "Debug".to_string(),
                        "Clone".to_string(),
                        "PartialEq".to_string(),
                        "Eq".to_string(),
                        "Hash".to_string(),
                    ],
                    doc: schema.schema_data.description.clone(),
                };

                if self.options.derive_serde {
                    ir_enum.derives.push("serde::Serialize".to_string());
                    ir_enum.derives.push("serde::Deserialize".to_string());
                }

                for variant_value in &string_type.enumeration {
                    if let Some(variant_str) = variant_value {
                        let pascal_name = to_pascal_case(variant_str);
                        let variant = IREnumVariant {
                            name: pascal_name.clone(),
                            source_value: if pascal_name != *variant_str {
                                Some(variant_str.clone())
                            } else {
                                None
                            },
                            doc: None,
                        };
                        ir_enum.variants.push(variant);
                    }
                }

                Ok(Some(IRType::Enum(ir_enum)))
            }

            SchemaKind::AllOf { all_of } => {
                // Merge all schemas
                self.convert_all_of(name, all_of)
            }

            SchemaKind::OneOf { one_of } => {
                // Create enum with variants
                self.convert_one_of(name, one_of)
            }

            SchemaKind::AnyOf { any_of } => {
                // Similar to oneOf
                self.convert_any_of(name, any_of)
            }

            _ => {
                // Other types (primitives) don't generate standalone types
                Ok(None)
            }
        }
    }

    /// Convert a property to an IR field
    fn convert_property(
        &mut self,
        name: &str,
        schema: &Schema,
        required_fields: &HashSet<&str>,
    ) -> Result<IRField> {
        let field_name = sanitize_field_name(name);
        let is_required = required_fields.contains(name);

        // Determine field type
        let mut field_type = match &schema.schema_kind {
            SchemaKind::Type(Type::Object(_)) => {
                // Nested object - generate a new type
                let nested_name = to_pascal_case(name);
                if let Some(IRType::Struct(nested_struct)) =
                    self.convert_schema(&nested_name, schema)?
                {
                    IRTypeRef::Named(nested_struct.name)
                } else {
                    IRTypeRef::Primitive(PrimitiveKind::Json)
                }
            }
            SchemaKind::Type(Type::String(string_type)) if !string_type.enumeration.is_empty() => {
                // Inline enum - generate enum type
                let enum_name = to_pascal_case(name);

                // Check if not already generated
                if !self.generated_types.contains(&enum_name) {
                    self.generated_types.insert(enum_name.clone());

                    let mut ir_enum = IREnum {
                        name: enum_name.clone(),
                        variants: Vec::new(),
                        derives: vec![
                            "Debug".to_string(),
                            "Clone".to_string(),
                            "PartialEq".to_string(),
                            "Eq".to_string(),
                            "Hash".to_string(),
                        ],
                        doc: schema.schema_data.description.clone(),
                    };

                    if self.options.derive_serde {
                        ir_enum.derives.push("serde::Serialize".to_string());
                        ir_enum.derives.push("serde::Deserialize".to_string());
                    }

                    for variant_value in &string_type.enumeration {
                        if let Some(variant_str) = variant_value {
                            let pascal_name = to_pascal_case(variant_str);
                            let variant = IREnumVariant {
                                name: pascal_name.clone(),
                                source_value: if pascal_name != *variant_str {
                                    Some(variant_str.clone())
                                } else {
                                    None
                                },
                                doc: None,
                            };
                            ir_enum.variants.push(variant);
                        }
                    }

                    self.inline_enum_types.push(IRType::Enum(ir_enum));
                }

                IRTypeRef::Named(enum_name)
            }
            _ => openapi_type_to_ir(schema, Some(name))?,
        };

        // Make optional if not required or if option is set
        if !is_required || self.options.make_fields_optional {
            field_type = field_type.make_optional();
        }

        let mut field = IRField::new(field_name.clone(), field_type);

        // Set source name for serde rename if different
        if field_name != name {
            field.source_name = Some(name.to_string());
            field.attributes.push(format!("#[serde(rename = \"{}\")]", name));
        }

        // Add documentation
        if self.options.generate_docs {
            if let Some(desc) = &schema.schema_data.description {
                field.doc = Some(desc.clone());
            }
        }

        // Extract validation constraints
        if self.options.generate_validation {
            field.constraints = extract_validation_constraints(schema);
        }

        field.optional = !is_required;

        Ok(field)
    }

    /// Resolve a schema reference to the actual schema
    fn resolve_schema_ref(&self, schema_ref: &'a ReferenceOr<Schema>) -> Result<&'a Schema> {
        match schema_ref {
            ReferenceOr::Item(schema) => Ok(schema),
            ReferenceOr::Reference { reference } => {
                // Check for circular references
                if self.reference_stack.contains(reference) {
                    return Err(OpenApiError::circular_reference(reference.clone()));
                }

                // Extract schema name from reference
                let schema_name = extract_type_name_from_ref(reference);

                // Look up in components
                let components = self.spec.components.as_ref().ok_or_else(|| {
                    OpenApiError::reference_resolution(
                        reference.clone(),
                        "no components in spec".to_string(),
                    )
                })?;

                let found_schema_ref = components.schemas.get(&schema_name).ok_or_else(|| {
                    OpenApiError::reference_resolution(
                        reference.clone(),
                        format!("schema '{}' not found in components", schema_name),
                    )
                })?;

                match found_schema_ref {
                    ReferenceOr::Item(schema) => Ok(schema),
                    ReferenceOr::Reference { .. } => Err(OpenApiError::reference_resolution(
                        reference.clone(),
                        "nested references not supported".to_string(),
                    )),
                }
            }
        }
    }

    /// Convert allOf schema composition
    fn convert_all_of(
        &mut self,
        name: &str,
        schemas: &[ReferenceOr<Schema>],
    ) -> Result<Option<IRType>> {
        let struct_name = self.options.format_type_name(&to_pascal_case(name));

        if self.generated_types.contains(&struct_name) {
            return Ok(None);
        }
        self.generated_types.insert(struct_name.clone());

        let mut ir_struct = IRStruct::new(struct_name);

        // Merge all schemas
        // First, collect all field info to avoid borrow checker issues
        let mut fields_to_process = Vec::new();

        for schema_ref in schemas {
            let schema = self.resolve_schema_ref(schema_ref)?;

            if let SchemaKind::Type(Type::Object(obj_type)) = &schema.schema_kind {
                let required: HashSet<_> = obj_type.required.iter().map(|s| s.as_str()).collect();

                for (field_name, property_ref) in &obj_type.properties {
                    let property = match property_ref {
                        ReferenceOr::Item(schema) => schema,
                        ReferenceOr::Reference { .. } => continue,
                    };
                    // Clone the necessary data to avoid holding the borrow
                    fields_to_process.push((
                        field_name.clone(),
                        property.clone(),
                        required.clone(),
                    ));
                }
            }
        }

        // Now process the fields with mutable access to self
        for (field_name, property, required) in fields_to_process {
            let required_set: HashSet<&str> = required.iter().map(|s| s.as_ref()).collect();
            let field = self.convert_property(&field_name, &property, &required_set)?;
            ir_struct.add_field(field);
        }

        // Add standard derives
        if self.options.derive_serde {
            ir_struct.add_derive("serde::Serialize".to_string());
            ir_struct.add_derive("serde::Deserialize".to_string());
        }

        Ok(Some(IRType::Struct(ir_struct)))
    }

    /// Convert oneOf schema composition (creates enum)
    fn convert_one_of(
        &mut self,
        name: &str,
        schemas: &[ReferenceOr<Schema>],
    ) -> Result<Option<IRType>> {
        let enum_name = self.options.format_type_name(&to_pascal_case(name));

        if self.generated_types.contains(&enum_name) {
            return Ok(None);
        }
        self.generated_types.insert(enum_name.clone());

        let mut ir_enum = IREnum {
            name: enum_name,
            variants: Vec::new(),
            derives: vec![
                "Debug".to_string(),
                "Clone".to_string(),
                "PartialEq".to_string(),
            ],
            doc: None,
        };

        if self.options.derive_serde {
            ir_enum.derives.push("serde::Serialize".to_string());
            ir_enum.derives.push("serde::Deserialize".to_string());
        }

        // Create variant for each schema
        for (idx, _schema_ref) in schemas.iter().enumerate() {
            let variant_name = format!("Variant{}", idx + 1);
            let variant = IREnumVariant {
                name: variant_name,
                source_value: None,
                doc: None,
            };
            ir_enum.variants.push(variant);
        }

        Ok(Some(IRType::Enum(ir_enum)))
    }

    /// Convert anyOf schema composition
    fn convert_any_of(
        &mut self,
        name: &str,
        schemas: &[ReferenceOr<Schema>],
    ) -> Result<Option<IRType>> {
        // Similar to oneOf for now
        self.convert_one_of(name, schemas)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_field_name() {
        assert_eq!(sanitize_field_name("userName"), "user_name");
        assert_eq!(sanitize_field_name("type"), "type_");
        assert_eq!(sanitize_field_name("123field"), "_123field");
    }
}
