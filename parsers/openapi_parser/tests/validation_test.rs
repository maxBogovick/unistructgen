use unistructgen_core::{CodeGenerator, Parser};
use unistructgen_openapi_parser::{OpenApiParser, OpenApiParserOptions};
use unistructgen_codegen::{RustRenderer, RenderOptions};

#[test]
fn test_validation_generation() {
    let openapi_spec = r#"
openapi: 3.0.0
info:
  title: Test API
  version: 1.0.0
paths:
  /users:
    get:
      responses:
        '200':
          description: OK
components:
  schemas:
    TestUser:
      type: object
      required:
        - id
        - name
      properties:
        id:
          type: integer
        name:
          type: string
          minLength: 1
          maxLength: 100
        email:
          type: string
          format: email
"#;

    let options = OpenApiParserOptions::builder()
        .generate_validation(true)
        .derive_serde(true)
        .build();

    let mut parser = OpenApiParser::new(options);
    let ir_module = parser.parse(openapi_spec).expect("Failed to parse OpenAPI");

    let renderer = RustRenderer::new(RenderOptions {
        add_header: false,
        add_clippy_allows: false,
    });

    let generated_code = renderer.render(&ir_module).expect("Failed to generate code");

    println!("=== GENERATED CODE ===");
    println!("{}", generated_code);
    println!("=== END GENERATED CODE ===");

    // Check that Validate derive is present
    assert!(generated_code.contains("Validate"), "Generated code should include Validate derive");

    // Check that validation attributes are present
    assert!(generated_code.contains("#[validate("), "Generated code should include validation attributes");

    // Check that min/max length constraints are present
    assert!(generated_code.contains("length("), "Generated code should include length validation");
}
