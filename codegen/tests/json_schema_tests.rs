use unistructgen_core::{
    CodeGenerator, IRField, IRModule, IRStruct, IRType, IRTypeRef, PrimitiveKind, IREnum, IREnumVariant,
};
use unistructgen_codegen::JsonSchemaRenderer;
use serde_json::Value;

#[test]
fn test_json_schema_simple_struct() {
    let mut ir_struct = IRStruct::new("User".to_string());
    ir_struct.doc = Some("A user of the system".to_string());
    ir_struct.add_field(IRField::new(
        "id".to_string(),
        IRTypeRef::Primitive(PrimitiveKind::I64),
    ));
    
    let mut name_field = IRField::new(
        "name".to_string(),
        IRTypeRef::Primitive(PrimitiveKind::String),
    );
    name_field.doc = Some("The user's full name".to_string());
    ir_struct.add_field(name_field);

    let mut module = IRModule::new("test".to_string());
    module.add_type(IRType::Struct(ir_struct));

    let renderer = JsonSchemaRenderer::default();
    let schema_str = renderer.generate(&module).expect("Failed to generate schema");
    
    let schema: Value = serde_json::from_str(&schema_str).expect("Invalid JSON");

    // Check root type
    assert_eq!(schema["$defs"]["User"]["type"], "object");
    assert_eq!(schema["$defs"]["User"]["description"], "A user of the system");
    
    // Check required fields (default is required)
    let required = schema["$defs"]["User"]["required"].as_array().unwrap();
    assert!(required.contains(&serde_json::json!("id")));
    assert!(required.contains(&serde_json::json!("name")));

    // Check properties
    let props = &schema["$defs"]["User"]["properties"];
    assert_eq!(props["id"]["type"], "integer");
    assert_eq!(props["name"]["type"], "string");
    assert_eq!(props["name"]["description"], "The user's full name");
}

#[test]
fn test_json_schema_nested_types() {
    // Address struct
    let mut address = IRStruct::new("Address".to_string());
    address.add_field(IRField::new("city".to_string(), IRTypeRef::Primitive(PrimitiveKind::String)));
    
    // User struct with Address
    let mut user = IRStruct::new("User".to_string());
    user.add_field(IRField::new(
        "address".to_string(), 
        IRTypeRef::Named("Address".to_string())
    ));

    let mut module = IRModule::new("test".to_string());
    module.add_type(IRType::Struct(address));
    module.add_type(IRType::Struct(user));

    let renderer = JsonSchemaRenderer::default();
    let schema_str = renderer.generate(&module).expect("Failed to generate");
    let schema: Value = serde_json::from_str(&schema_str).unwrap();

    // Check ref
    assert_eq!(
        schema["$defs"]["User"]["properties"]["address"]["$ref"],
        "#/$defs/Address"
    );
}

#[test]
fn test_json_schema_arrays_and_options() {
    let mut data = IRStruct::new("Data".to_string());
    
    // Optional field
    let mut opt_field = IRField::new(
        "optional_val".to_string(),
        IRTypeRef::Option(Box::new(IRTypeRef::Primitive(PrimitiveKind::I32)))
    );
    // Explicitly set optional flag in IRField (important for logic)
    opt_field.optional = true;
    data.add_field(opt_field);

    // Array field
    data.add_field(IRField::new(
        "tags".to_string(),
        IRTypeRef::Vec(Box::new(IRTypeRef::Primitive(PrimitiveKind::String)))
    ));

    let mut module = IRModule::new("test".to_string());
    module.add_type(IRType::Struct(data));

    let renderer = JsonSchemaRenderer::default();
    let schema_str = renderer.generate(&module).unwrap();
    let schema: Value = serde_json::from_str(&schema_str).unwrap();

    let props = &schema["$defs"]["Data"]["properties"];
    
    // Optional field should NOT be in required array
    let required = schema["$defs"]["Data"].get("required");
    if let Some(req) = required {
        let req_arr = req.as_array().unwrap();
        assert!(!req_arr.contains(&serde_json::json!("optional_val")));
        assert!(req_arr.contains(&serde_json::json!("tags")));
    }

    // Array check
    assert_eq!(props["tags"]["type"], "array");
    assert_eq!(props["tags"]["items"]["type"], "string");
}

#[test]
fn test_json_schema_enum() {
    let mut status = IREnum {
        name: "Status".to_string(),
        variants: vec![
            IREnumVariant { name: "Active".to_string(), source_value: None, doc: None },
            IREnumVariant { name: "Inactive".to_string(), source_value: None, doc: None },
        ],
        derives: vec![],
        doc: Some("User status".to_string()),
    };

    let mut module = IRModule::new("test".to_string());
    module.add_type(IRType::Enum(status));

    let renderer = JsonSchemaRenderer::default();
    let schema_str = renderer.generate(&module).unwrap();
    let schema: Value = serde_json::from_str(&schema_str).unwrap();

    let enum_def = &schema["$defs"]["Status"];
    assert_eq!(enum_def["type"], "string");
    assert_eq!(enum_def["description"], "User status");
    
    let variants = enum_def["enum"].as_array().unwrap();
    assert!(variants.contains(&serde_json::json!("Active")));
    assert!(variants.contains(&serde_json::json!("Inactive")));
}
