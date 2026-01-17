//! Integration tests for OpenAPI parser

use unistructgen_core::Parser;
use unistructgen_openapi_parser::{OpenApiParser, OpenApiParserOptions};

const PETSTORE_SPEC: &str = r#"
openapi: 3.0.0
info:
  title: Pet Store API
  version: 1.0.0
  description: A sample API for a pet store
paths: {}
components:
  schemas:
    Pet:
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
        tag:
          type: string
        status:
          type: string
          enum:
            - available
            - pending
            - sold

    NewPet:
      type: object
      required:
        - name
      properties:
        name:
          type: string
          minLength: 1
          maxLength: 100
        tag:
          type: string

    Error:
      type: object
      required:
        - code
        - message
      properties:
        code:
          type: integer
          format: int32
        message:
          type: string
"#;

#[test]
fn test_parse_petstore_spec() {
    let options = OpenApiParserOptions::default();
    let mut parser = OpenApiParser::new(options);

    let result = parser.parse(PETSTORE_SPEC);
    assert!(result.is_ok(), "Failed to parse petstore spec: {:?}", result.err());

    let module = result.unwrap();
    assert!(!module.types.is_empty(), "No types were generated");

    // Should have Pet, NewPet, and Error types
    assert!(module.types.len() >= 3, "Expected at least 3 types, got {}", module.types.len());
}

#[test]
fn test_enum_generation() {
    let spec = r#"
openapi: 3.0.0
info:
  title: Test API
  version: 1.0.0
paths: {}
components:
  schemas:
    Status:
      type: string
      enum:
        - active
        - inactive
        - pending
    "#;

    let options = OpenApiParserOptions::default();
    let mut parser = OpenApiParser::new(options);

    let result = parser.parse(spec);
    assert!(result.is_ok());

    let module = result.unwrap();
    assert_eq!(module.types.len(), 1);
}

#[test]
fn test_nested_objects() {
    let spec = r#"
openapi: 3.0.0
info:
  title: Test API
  version: 1.0.0
paths: {}
components:
  schemas:
    User:
      type: object
      properties:
        id:
          type: integer
        profile:
          type: object
          properties:
            name:
              type: string
            email:
              type: string
              format: email
    "#;

    let options = OpenApiParserOptions::default();
    let mut parser = OpenApiParser::new(options);

    let result = parser.parse(spec);
    assert!(result.is_ok());
}

#[test]
fn test_array_types() {
    let spec = r#"
openapi: 3.0.0
info:
  title: Test API
  version: 1.0.0
paths: {}
components:
  schemas:
    PetList:
      type: object
      properties:
        pets:
          type: array
          items:
            type: string
        ids:
          type: array
          items:
            type: integer
    "#;

    let options = OpenApiParserOptions::default();
    let mut parser = OpenApiParser::new(options);

    let result = parser.parse(spec);
    assert!(result.is_ok());
}

#[test]
fn test_validation_constraints() {
    let spec = r#"
openapi: 3.0.0
info:
  title: Test API
  version: 1.0.0
paths: {}
components:
  schemas:
    ValidatedUser:
      type: object
      properties:
        username:
          type: string
          minLength: 3
          maxLength: 20
        age:
          type: integer
          minimum: 0
          maximum: 150
        email:
          type: string
          format: email
    "#;

    let options = OpenApiParserOptions::builder()
        .generate_validation(true)
        .build();
    let mut parser = OpenApiParser::new(options);

    let result = parser.parse(spec);
    assert!(result.is_ok());

    let module = result.unwrap();
    // Validate that constraints were extracted
    // (actual constraint checking would require inspecting the IR)
    assert!(!module.types.is_empty());
}

#[test]
fn test_all_of_composition() {
    let spec = r#"
openapi: 3.0.0
info:
  title: Test API
  version: 1.0.0
paths: {}
components:
  schemas:
    BaseEntity:
      type: object
      properties:
        id:
          type: integer
        createdAt:
          type: string
          format: date-time

    User:
      allOf:
        - type: object
          properties:
            name:
              type: string
        - type: object
          properties:
            email:
              type: string
    "#;

    let options = OpenApiParserOptions::default();
    let mut parser = OpenApiParser::new(options);

    let result = parser.parse(spec);
    assert!(result.is_ok());
}

#[test]
fn test_reference_resolution() {
    let spec = r#"
openapi: 3.0.0
info:
  title: Test API
  version: 1.0.0
paths: {}
components:
  schemas:
    Address:
      type: object
      properties:
        street:
          type: string
        city:
          type: string

    User:
      type: object
      properties:
        name:
          type: string
        address:
          $ref: '#/components/schemas/Address'
    "#;

    let options = OpenApiParserOptions::default();
    let mut parser = OpenApiParser::new(options);

    let result = parser.parse(spec);
    assert!(result.is_ok());
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
fn test_invalid_openapi_version() {
    let spec = r#"
openapi: 2.0.0
info:
  title: Old API
  version: 1.0.0
paths: {}
    "#;

    let mut parser = OpenApiParser::with_defaults();
    let result = parser.parse(spec);
    assert!(result.is_err());
}

#[test]
fn test_empty_spec() {
    let spec = r#"
openapi: 3.0.0
info:
  title: Empty API
  version: 1.0.0
paths: {}
    "#;

    let mut parser = OpenApiParser::with_defaults();
    let result = parser.parse(spec);
    // Should error because no components or paths
    assert!(result.is_err());
}

#[test]
fn test_with_validation_disabled() {
    let options = OpenApiParserOptions::builder()
        .generate_validation(false)
        .build();

    let mut parser = OpenApiParser::new(options);
    let result = parser.parse(PETSTORE_SPEC);
    assert!(result.is_ok());
}

#[test]
fn test_with_client_generation() {
    let spec = r#"
openapi: 3.0.0
info:
  title: API with Paths
  version: 1.0.0
paths:
  /pets:
    get:
      operationId: listPets
      responses:
        '200':
          description: Success
components:
  schemas:
    Pet:
      type: object
      properties:
        id:
          type: integer
        name:
          type: string
    "#;

    let options = OpenApiParserOptions::builder()
        .generate_client(true)
        .build();

    let mut parser = OpenApiParser::new(options);
    let result = parser.parse(spec);
    assert!(result.is_ok());
}
