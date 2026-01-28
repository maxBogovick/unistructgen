use unistructgen_codegen::{JsonSchemaRenderer, RenderOptions, RustRenderer};
use unistructgen_core::{CodeGenerator, FieldConstraints, IRField, IRModule, IRStruct, IRType, IRTypeRef, PrimitiveKind};

fn build_user_module() -> IRModule {
    let mut user = IRStruct::new("User".to_string());

    user.add_field(IRField::new(
        "id".to_string(),
        IRTypeRef::Primitive(PrimitiveKind::I64),
    ));

    let mut name_field = IRField::new(
        "name".to_string(),
        IRTypeRef::Primitive(PrimitiveKind::String),
    );
    name_field.optional = true;
    name_field.ty = name_field.ty.make_optional();
    name_field.constraints = FieldConstraints {
        min_length: Some(1),
        ..Default::default()
    };
    user.add_field(name_field);

    let mut module = IRModule::new("User".to_string());
    module.add_type(IRType::Struct(user));

    module
}

fn normalize(text: &str) -> String {
    text.trim_end().to_string()
}

fn parse_json(text: &str) -> serde_json::Value {
    serde_json::from_str(text).expect("valid json")
}

#[test]
fn rust_renderer_golden() {
    let module = build_user_module();
    let renderer = RustRenderer::new(RenderOptions {
        add_header: false,
        add_clippy_allows: false,
    });

    let output = renderer.generate(&module).expect("render rust");
    let expected = include_str!("fixtures/user_struct.rs");

    assert_eq!(normalize(&output), normalize(expected));
}

#[test]
fn json_schema_renderer_golden() {
    let module = build_user_module();
    let renderer = JsonSchemaRenderer::new();

    let output = renderer.generate(&module).expect("render json schema");
    let expected = include_str!("fixtures/user_schema.json");

    assert_eq!(parse_json(&output), parse_json(expected));
}
