//! API client trait generation from OpenAPI paths

use crate::parsers::openapi::error::Result;
use crate::parsers::openapi::options::OpenApiParserOptions;
use crate::parsers::openapi::types::{extract_type_name_from_ref, sanitize_field_name, to_pascal_case};
use openapiv3::{OpenAPI, Operation, PathItem, ReferenceOr};
use crate::core::{IRField, IRStruct, IRType, IRTypeRef, PrimitiveKind};

/// API client generator
pub struct ClientGenerator<'a> {
    spec: &'a OpenAPI,
    options: &'a OpenApiParserOptions,
}

impl<'a> ClientGenerator<'a> {
    /// Create a new client generator
    pub fn new(spec: &'a OpenAPI, options: &'a OpenApiParserOptions) -> Self {
        Self { spec, options }
    }

    /// Generate API client trait and method types
    pub fn generate_client_types(&self) -> Result<Vec<IRType>> {
        let mut types = Vec::new();

        // Generate request/response types for each operation
        for (path, path_item) in &self.spec.paths.paths {
            let path_item = match path_item {
                ReferenceOr::Item(item) => item,
                ReferenceOr::Reference { .. } => continue,
            };

            self.generate_path_types(path, path_item, &mut types)?;
        }

        Ok(types)
    }

    /// Generate types for a single path
    fn generate_path_types(
        &self,
        path: &str,
        path_item: &PathItem,
        types: &mut Vec<IRType>,
    ) -> Result<()> {
        // Process each HTTP method
        if let Some(op) = &path_item.get {
            self.generate_operation_types(path, "Get", op, types)?;
        }
        if let Some(op) = &path_item.post {
            self.generate_operation_types(path, "Post", op, types)?;
        }
        if let Some(op) = &path_item.put {
            self.generate_operation_types(path, "Put", op, types)?;
        }
        if let Some(op) = &path_item.delete {
            self.generate_operation_types(path, "Delete", op, types)?;
        }
        if let Some(op) = &path_item.patch {
            self.generate_operation_types(path, "Patch", op, types)?;
        }

        Ok(())
    }

    /// Generate types for a single operation
    fn generate_operation_types(
        &self,
        path: &str,
        method: &str,
        operation: &Operation,
        types: &mut Vec<IRType>,
    ) -> Result<()> {
        // Determine operation name
        let operation_name = if let Some(operation_id) = &operation.operation_id {
            to_pascal_case(operation_id)
        } else {
            // Generate from path and method
            let path_parts: Vec<_> = path
                .split('/')
                .filter(|s| !s.is_empty() && !s.starts_with('{'))
                .collect();
            format!("{}{}", method, path_parts.join(""))
        };

        // Generate request type if there are parameters or request body
        if !operation.parameters.is_empty() || operation.request_body.is_some() {
            let request_type = self.generate_request_type(&operation_name, operation)?;
            if let Some(ty) = request_type {
                types.push(ty);
            }
        }

        // Generate response types
        for (status_code, response_ref) in &operation.responses.responses {
            let response = match response_ref {
                ReferenceOr::Item(resp) => resp,
                ReferenceOr::Reference { .. } => continue,
            };

            let _response_name = format!("{}{}Response", operation_name, status_code);

            // Check if response has content
            if let Some(media_type) = response.content.get("application/json") {
                if let Some(schema_ref) = &media_type.schema {
                    // Extract type from schema reference
                    match schema_ref {
                        ReferenceOr::Reference { .. } => {
                            // Type already defined in components
                            continue;
                        }
                        ReferenceOr::Item(_schema) => {
                            // Inline schema - would need to generate type
                            // For now, skip inline response schemas
                            continue;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Generate request type from operation parameters and body
    fn generate_request_type(
        &self,
        operation_name: &str,
        operation: &Operation,
    ) -> Result<Option<IRType>> {
        let mut ir_struct = IRStruct::new(format!("{}Request", operation_name));

        // Add documentation
        if self.options.generate_docs {
            if let Some(summary) = &operation.summary {
                ir_struct.doc = Some(format!("Request parameters for {}", summary));
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

        // Process parameters
        for param_ref in &operation.parameters {
            let param = match param_ref {
                ReferenceOr::Item(p) => p,
                ReferenceOr::Reference { .. } => continue,
            };

            match param {
                openapiv3::Parameter::Query { parameter_data, .. }
                | openapiv3::Parameter::Path { parameter_data, .. }
                | openapiv3::Parameter::Header { parameter_data, .. } => {
                    let field_name = sanitize_field_name(&parameter_data.name);

                    // Determine type from schema
                    let field_type = match &parameter_data.format {
                        openapiv3::ParameterSchemaOrContent::Schema(schema_ref) => {
                            match schema_ref {
                                ReferenceOr::Reference { reference } => {
                                    IRTypeRef::Named(extract_type_name_from_ref(reference))
                                }
                                ReferenceOr::Item(_schema) => {
                                    // Default to string for inline schemas
                                    IRTypeRef::Primitive(PrimitiveKind::String)
                                }
                            }
                        }
                        openapiv3::ParameterSchemaOrContent::Content(_) => {
                            IRTypeRef::Primitive(PrimitiveKind::String)
                        }
                    };

                    let mut field = IRField::new(field_name.clone(), field_type);

                    // Make optional if not required
                    if !parameter_data.required {
                        field.ty = field.ty.make_optional();
                        field.optional = true;
                    }

                    // Add documentation
                    if self.options.generate_docs {
                        if let Some(desc) = &parameter_data.description {
                            field.doc = Some(desc.clone());
                        }
                    }

                    // Add serde rename if needed
                    if field_name != parameter_data.name {
                        field.source_name = Some(parameter_data.name.clone());
                        field.attributes.push(format!(
                            "#[serde(rename = \"{}\")]",
                            parameter_data.name
                        ));
                    }

                    ir_struct.add_field(field);
                }
                _ => {}
            }
        }

        // If no fields, don't generate the type
        if ir_struct.fields.is_empty() {
            return Ok(None);
        }

        Ok(Some(IRType::Struct(ir_struct)))
    }

    /// Generate API client trait definition (as documentation)
    pub fn generate_client_trait_doc(&self) -> String {
        let mut output = String::new();

        output.push_str("// API Client Trait\n");
        output.push_str("// This trait can be implemented to create an API client\n\n");
        output.push_str("#[async_trait::async_trait]\n");
        output.push_str("pub trait ApiClient {\n");

        for (path, path_item) in &self.spec.paths.paths {
            let path_item = match path_item {
                ReferenceOr::Item(item) => item,
                ReferenceOr::Reference { .. } => continue,
            };

            self.generate_client_methods(path, path_item, &mut output);
        }

        output.push_str("}\n");
        output
    }

    fn generate_client_methods(&self, path: &str, path_item: &PathItem, output: &mut String) {
        if let Some(op) = &path_item.get {
            self.generate_client_method(path, "get", op, output);
        }
        if let Some(op) = &path_item.post {
            self.generate_client_method(path, "post", op, output);
        }
        if let Some(op) = &path_item.put {
            self.generate_client_method(path, "put", op, output);
        }
        if let Some(op) = &path_item.delete {
            self.generate_client_method(path, "delete", op, output);
        }
    }

    fn generate_client_method(
        &self,
        path: &str,
        method: &str,
        operation: &Operation,
        output: &mut String,
    ) {
        let operation_name = if let Some(operation_id) = &operation.operation_id {
            sanitize_field_name(operation_id)
        } else {
            format!("{}_{}", method, path.replace(['/', '{', '}'], "_"))
        };

        output.push_str(&format!("    async fn {}(", operation_name));
        output.push_str("&self");

        // Add parameters
        for param_ref in &operation.parameters {
            if let ReferenceOr::Item(param) = param_ref {
                match param {
                    openapiv3::Parameter::Path { parameter_data, .. } => {
                        let param_name = sanitize_field_name(&parameter_data.name);
                        output.push_str(&format!(", {}: &str", param_name));
                    }
                    _ => {}
                }
            }
        }

        output.push_str(") -> Result<serde_json::Value, Box<dyn std::error::Error>>;\n\n");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operation_name_generation() {
        let name = to_pascal_case("get_users");
        assert_eq!(name, "GetUsers");
    }
}
