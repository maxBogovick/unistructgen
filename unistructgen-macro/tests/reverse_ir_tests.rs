use unistructgen_macro::{generate_struct_from_json, IntoIR};
use unistructgen::core::CodeGenerator;
use unistructgen::codegen::JsonSchemaRenderer;
use unistructgen::core::ir::IRModule;
use unistructgen::parsers::openapi::{OpenApiParser, OpenApiParserOptions};
use unistructgen::core::Parser;

// --- Test 1: Schema -> IR -> Rust -> IR -> Schema ---

generate_struct_from_json! {
    name = "UserReverse",
    json = r#"{"id": 123, "username": "jdoe", "is_active": true}"#,
    serde = true,
    reverse_ir = true
}

#[test]
fn test_schema_to_rust_to_schema() {
    // 1. We have the generated struct `UserReverse` from JSON above.
    // It should implement IntoIR.
    
    // 2. Convert back to IR (Reverse IR)
    let definition = <UserReverse as unistructgen::core::IntoIR>::ir_definition()
        .expect("Failed to get IR definition from generated struct");
    
    // 3. Wrap in module
    let mut module = IRModule::new("UserReverseModule".to_string());
    module.add_type(definition);
    
    // 4. Generate Schema
    let renderer = JsonSchemaRenderer::new().fragment();
    let schema_json = renderer.generate(&module).expect("Failed to generate JSON Schema");
    
    println!("Generated Schema:\n{}", schema_json);
    
    // 5. Verify the schema matches expectations
    let schema: serde_json::Value = serde_json::from_str(&schema_json).expect("Invalid JSON");
    
    // Access properties inside $defs since JsonSchemaRenderer wraps them
    let defs = schema.get("$defs").expect("Missing $defs");
    let user_def = defs.get("UserReverse").expect("Missing UserReverse def");
    let props = user_def.get("properties").expect("Missing properties");
    
    assert!(props.get("id").is_some());
    assert!(props.get("username").is_some());
    assert!(props.get("is_active").is_some());
    
    assert_eq!(props["id"]["type"], "integer");
    assert_eq!(props["username"]["type"], "string");
    assert_eq!(props["is_active"]["type"], "boolean");
}

// --- Test 2: Rust -> Schema -> IR -> Rust ---

#[derive(IntoIR, Clone, Debug, PartialEq)]
struct Product {
    #[field(min_value = 0.0, doc = "Price of the product")]
    price: f64,
    
    #[field(min_length = 3, max_length = 50)]
    name: String,
    
    in_stock: bool,
}

#[test]
fn test_rust_to_schema_to_rust() {
    // 1. Rust -> IR
    let definition = <Product as unistructgen::core::IntoIR>::ir_definition()
        .expect("Failed to get IR");
    let mut module = IRModule::new("ProductModule".to_string());
    module.add_type(definition);
    
    // 2. IR -> JSON Schema
    let renderer = JsonSchemaRenderer::new().fragment(); // fragment to embed in OpenAPI
    let schema_str = renderer.generate(&module).expect("Failed to generate schema");
    
    // Parse the generated schema to extract the definition
    let schema_json: serde_json::Value = serde_json::from_str(&schema_str).expect("Invalid JSON Schema");
    let product_def = schema_json["$defs"]["Product"].clone();
    let product_json_str = serde_json::to_string_pretty(&product_def).expect("Failed to serialize definition");
    
    // 3. Wrap in OpenAPI spec (Schema -> IR via OpenAPI Parser)
    // We construct a fake OpenAPI spec that defines a component with this schema
    let openapi_yaml = format!("openapi: 3.0.0
info:
  title: Test API
  version: 1.0.0
paths: {{}}
components:
  schemas:
    Product:
{}", indent_json(&product_json_str));

    // Helper to indent the JSON string to fit into YAML
    fn indent_json(json: &str) -> String {
        json.lines().map(|l| format!("      {}", l)).collect::<Vec<_>>().join("\n")
    }

    // 4. Parse OpenAPI -> IR
    let options = OpenApiParserOptions::builder().build();
    let mut parser = OpenApiParser::new(options);
    let ir_back = parser.parse(&openapi_yaml).expect("Failed to parse generated OpenAPI");
    
    // 5. Verify IR structure
    assert!(!ir_back.types.is_empty());
    let struct_def = match &ir_back.types[0] {
        unistructgen::core::IRType::Struct(s) => s,
        _ => panic!("Expected struct"),
    };
    
    assert_eq!(struct_def.name, "Product");
    
    // Check fields
    let price_field = struct_def.fields.iter().find(|f| f.name == "price").expect("Missing price");
    let name_field = struct_def.fields.iter().find(|f| f.name == "name").expect("Missing name");
    
    // Check types
    assert!(matches!(price_field.ty, unistructgen::core::IRTypeRef::Primitive(unistructgen::core::PrimitiveKind::F64) | unistructgen::core::IRTypeRef::Primitive(unistructgen::core::PrimitiveKind::Decimal)));
    assert!(matches!(name_field.ty, unistructgen::core::IRTypeRef::Primitive(unistructgen::core::PrimitiveKind::String)));
}
