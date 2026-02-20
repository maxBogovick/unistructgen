use unistructgen_core::{IRType, IRTypeRef, Parser, PrimitiveKind};
use unistructgen_markdown_parser::{MarkdownParser, MarkdownParserOptions};

#[test]
fn test_markdown_parser_simple_table() {
    let markdown = r#"
# User Table

| Field Name | Type | Description | Required |
|------------|------|-------------|----------|
| id         | uuid | Unique ID   | yes      |
| name       | string | User name | yes      |
| age        | integer | Age      | no       |
    "#;

    let options = MarkdownParserOptions {
        struct_name: "User".to_string(),
        ..Default::default()
    };

    let mut parser = MarkdownParser::new(options);
    let module = parser.parse(markdown).expect("Failed to parse markdown");

    assert_eq!(module.types.len(), 1);
    
    match &module.types[0] {
        IRType::Struct(s) => {
            assert_eq!(s.name, "User");
            assert_eq!(s.fields.len(), 3);

            let id_field = &s.fields[0];
            assert_eq!(id_field.name, "id");
            assert!(matches!(id_field.ty, IRTypeRef::Primitive(PrimitiveKind::Uuid)));
            assert_eq!(id_field.doc, Some("Unique ID".to_string()));
            assert!(!id_field.optional);

            let age_field = &s.fields[2];
            assert_eq!(age_field.name, "age");
            assert!(matches!(age_field.ty, IRTypeRef::Option(_)));
            assert!(age_field.optional);
        }
        _ => assert!(false, "Expected struct"),
    }
}

#[test]
fn test_markdown_parser_complex_types() {
    let markdown = r#"
| Property | Type |
|----------|------|
| tags     | Vec<String> |
| metadata | json |
| scores   | integer[] |
    "#;

    let mut parser = MarkdownParser::new(Default::default());
    let module = parser.parse(markdown).expect("Failed to parse markdown");

    match &module.types[0] {
        IRType::Struct(s) => {
            let tags = &s.fields[0];
            assert_eq!(tags.name, "tags");
            // Vec<String>
            if let IRTypeRef::Vec(inner) = &tags.ty {
                assert!(matches!(**inner, IRTypeRef::Primitive(PrimitiveKind::String)));
            } else {
                assert!(false, "Expected Vec<String>");
            }

            let scores = &s.fields[2];
             // integer[] -> Vec<i32>
             if let IRTypeRef::Vec(inner) = &scores.ty {
                assert!(matches!(**inner, IRTypeRef::Primitive(PrimitiveKind::I32)));
            } else {
                assert!(false, "Expected Vec<i32>");
            }
        }
        _ => assert!(false, "Expected struct"),
    }
}
