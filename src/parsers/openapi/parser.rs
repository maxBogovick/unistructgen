//! Main OpenAPI parser implementation

use crate::parsers::openapi::client::ClientGenerator;
use crate::parsers::openapi::error::{OpenApiError, Result};
use crate::parsers::openapi::options::OpenApiParserOptions;
use crate::parsers::openapi::schema::SchemaConverter;
use openapiv3::OpenAPI;
use crate::core::{IRModule, Parser, ParserMetadata};

/// OpenAPI/Swagger parser
///
/// Converts OpenAPI 3.0 specifications into UniStructGen's Intermediate Representation (IR).
/// Supports both YAML and JSON formats.
///
/// # Examples
///
/// ```no_run
/// use unistructgen::parsers::openapi::{OpenApiParser, OpenApiParserOptions};
/// use unistructgen::core::Parser;
///
/// let options = OpenApiParserOptions::builder()
///     .generate_client(true)
///     .generate_validation(true)
///     .build();
///
/// let mut parser = OpenApiParser::new(options);
///
/// let yaml = std::fs::read_to_string("openapi.yaml").unwrap();
/// let ir_module = parser.parse(&yaml).unwrap();
///
/// println!("Generated {} types", ir_module.types.len());
/// ```
pub struct OpenApiParser {
    options: OpenApiParserOptions,
}

impl OpenApiParser {
    /// Create a new OpenAPI parser with the given options
    pub fn new(options: OpenApiParserOptions) -> Self {
        Self { options }
    }

    /// Create a new parser with default options
    pub fn with_defaults() -> Self {
        Self::new(OpenApiParserOptions::default())
    }

    /// Parse OpenAPI spec from YAML string
    fn parse_yaml(&self, input: &str) -> Result<OpenAPI> {
        serde_yaml::from_str(input).map_err(OpenApiError::from)
    }

    /// Parse OpenAPI spec from JSON string
    fn parse_json(&self, input: &str) -> Result<OpenAPI> {
        serde_json::from_str(input).map_err(OpenApiError::from)
    }

    /// Parse OpenAPI spec (auto-detect format)
    fn parse_spec(&self, input: &str) -> Result<OpenAPI> {
        // Try JSON first (faster)
        if let Ok(spec) = self.parse_json(input) {
            return Ok(spec);
        }

        // Fall back to YAML
        self.parse_yaml(input)
    }

    /// Validate OpenAPI specification
    fn validate_spec(&self, spec: &OpenAPI) -> Result<()> {
        // Check OpenAPI version
        if !spec.openapi.starts_with("3.0") && !spec.openapi.starts_with("3.1") {
            return Err(OpenApiError::invalid_spec(format!(
                "Unsupported OpenAPI version: {}. Only 3.0 and 3.1 are supported.",
                spec.openapi
            )));
        }

        // Check that spec has components or paths
        if spec.components.is_none() && spec.paths.paths.is_empty() {
            return Err(OpenApiError::invalid_spec(
                "OpenAPI spec must have either components or paths",
            ));
        }

        Ok(())
    }
}

impl Parser for OpenApiParser {
    type Error = OpenApiError;

    fn parse(&mut self, input: &str) -> Result<IRModule> {
        // Parse the OpenAPI specification
        let spec = self.parse_spec(input)?;

        // Validate the spec
        self.validate_spec(&spec)?;

        // Create IR module
        let module_name = if !spec.info.title.is_empty() {
            spec.info.title.clone()
        } else {
            "Api".to_string()
        };
        let mut ir_module = IRModule::new(module_name);

        // Convert schemas to IR types
        let mut converter = SchemaConverter::new(&spec, &self.options);
        let schema_types = converter.convert_all_schemas()?;

        for ir_type in schema_types {
            ir_module.add_type(ir_type);
        }

        // Generate API client types if requested
        if self.options.generate_client {
            let client_gen = ClientGenerator::new(&spec, &self.options);
            let client_types = client_gen.generate_client_types()?;

            for ir_type in client_types {
                ir_module.add_type(ir_type);
            }
        }

        Ok(ir_module)
    }

    fn name(&self) -> &'static str {
        "OpenAPI"
    }

    fn extensions(&self) -> &[&'static str] {
        &["yaml", "yml", "json"]
    }

    fn validate(&self, input: &str) -> Result<()> {
        let spec = self.parse_spec(input)?;
        self.validate_spec(&spec)?;
        Ok(())
    }

    fn metadata(&self) -> ParserMetadata {
        let mut custom = std::collections::HashMap::new();
        custom.insert("name".to_string(), self.name().to_string());
        custom.insert(
            "supported_formats".to_string(),
            self.extensions().join(", "),
        );

        ParserMetadata {
            version: Some(env!("CARGO_PKG_VERSION").to_string()),
            description: Some(
                "Parses OpenAPI 3.0/3.1 specifications and generates Rust types".to_string(),
            ),
            features: vec![
                "Schema to Struct conversion".to_string(),
                "Enum generation from string enums".to_string(),
                "API client trait generation".to_string(),
                "Validation constraint extraction".to_string(),
                "Reference ($ref) resolution".to_string(),
                "Schema composition (allOf, oneOf, anyOf)".to_string(),
            ],
            custom,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SIMPLE_OPENAPI: &str = r#"
openapi: 3.0.0
info:
  title: Test API
  version: 1.0.0
paths: {}
components:
  schemas:
    User:
      type: object
      required:
        - id
        - name
      properties:
        id:
          type: integer
          format: int64
        name:
          type: string
        email:
          type: string
          format: email
    "#;

    #[test]
    fn test_parse_simple_spec() {
        let options = OpenApiParserOptions::default();
        let mut parser = OpenApiParser::new(options);

        let result = parser.parse(SIMPLE_OPENAPI);
        if let Err(ref e) = result {
            eprintln!("Parse error: {:?}", e);
        }
        assert!(result.is_ok());

        let module = result.unwrap();
        assert_eq!(module.types.len(), 1);
    }

    #[test]
    fn test_parser_metadata() {
        let parser = OpenApiParser::with_defaults();
        let metadata = parser.metadata();

        assert_eq!(metadata.custom.get("name"), Some(&"OpenAPI".to_string()));
        assert!(metadata
            .custom
            .get("supported_formats")
            .map(|s| s.contains("yaml"))
            .unwrap_or(false));
        assert!(!metadata.features.is_empty());
    }

    #[test]
    fn test_validate_spec() {
        let parser = OpenApiParser::with_defaults();
        let result = parser.validate(SIMPLE_OPENAPI);
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_version() {
        let invalid_spec = r#"
openapi: 2.0.0
info:
  title: Old API
  version: 1.0.0
paths: {}
        "#;

        let mut parser = OpenApiParser::with_defaults();
        let result = parser.parse(invalid_spec);
        assert!(result.is_err());
    }
}
