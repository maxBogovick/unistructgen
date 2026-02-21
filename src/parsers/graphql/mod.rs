use crate::core::{
    IRField, IRModule, IRStruct, IRType, IRTypeRef, Parser, ParserMetadata, PrimitiveKind,
};
use graphql_parser::schema::{parse_schema, Definition, Type, TypeDefinition};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GraphqlParserError {
    #[error("GraphQL parsing error: {0}")]
    ParseError(#[from] graphql_parser::schema::ParseError),
    #[error("Unsupported definition: {0}")]
    UnsupportedDefinition(String),
}

pub type Result<T> = std::result::Result<T, GraphqlParserError>;

#[derive(Debug, Clone)]
pub struct GraphqlParserOptions {
    pub derive_serde: bool,
    pub derive_default: bool,
    pub make_fields_optional: bool, // Force all fields optional?
}

impl Default for GraphqlParserOptions {
    fn default() -> Self {
        Self {
            derive_serde: true,
            derive_default: false,
            make_fields_optional: false,
        }
    }
}

pub struct GraphqlParser {
    options: GraphqlParserOptions,
}

impl GraphqlParser {
    pub fn new(options: GraphqlParserOptions) -> Self {
        Self { options }
    }

    // Returns (RustType, is_optional)
    fn map_graphql_type(&self, gql_type: &Type<String>) -> (IRTypeRef, bool) {
        match gql_type {
            Type::NamedType(name) => {
                let ty = match name.as_str() {
                    "Int" => IRTypeRef::Primitive(PrimitiveKind::I32),
                    "Float" => IRTypeRef::Primitive(PrimitiveKind::F64),
                    "String" => IRTypeRef::Primitive(PrimitiveKind::String),
                    "Boolean" => IRTypeRef::Primitive(PrimitiveKind::Bool),
                    "ID" => IRTypeRef::Primitive(PrimitiveKind::String), // IDs are typically strings
                    _ => IRTypeRef::Named(name.clone()),
                };
                (ty, true) // Named types are nullable by default in GraphQL
            }
            Type::ListType(inner) => {
                let (inner_ty, inner_optional) = self.map_graphql_type(inner);
                let vec_inner = if inner_optional {
                    IRTypeRef::Option(Box::new(inner_ty))
                } else {
                    inner_ty
                };
                (IRTypeRef::Vec(Box::new(vec_inner)), true) // Lists are nullable by default
            }
            Type::NonNullType(inner) => {
                let (ty, _) = self.map_graphql_type(inner);
                (ty, false) // Explicitly non-null
            }
        }
    }

    fn to_snake_case(str: &str) -> String {
        let mut result = String::new();
        let mut prev_is_upper = false;
        
        for (i, c) in str.chars().enumerate() {
            if c.is_uppercase() {
                if i > 0 && !prev_is_upper {
                    result.push('_');
                }
                result.push(c.to_ascii_lowercase());
                prev_is_upper = true;
            } else {
                result.push(c);
                prev_is_upper = false;
            }
        }
        result
    }

    fn process_type_definition(&self, def: &TypeDefinition<String>) -> Option<IRStruct> {
        match def {
            TypeDefinition::Object(obj) => {
                let mut ir_struct = IRStruct::new(obj.name.clone());

                if self.options.derive_serde {
                    ir_struct.add_derive("serde::Serialize".to_string());
                    ir_struct.add_derive("serde::Deserialize".to_string());
                }

                if self.options.derive_default {
                    ir_struct.add_derive("Default".to_string());
                }

                for field_def in &obj.fields {
                    let (mut field_type, mut is_optional) = self.map_graphql_type(&field_def.field_type);
                    
                    if self.options.make_fields_optional {
                        is_optional = true;
                    }

                    if is_optional {
                         field_type = field_type.make_optional();
                    }

                    let field_name = Self::to_snake_case(&field_def.name);
                    let mut field = IRField::new(field_name.clone(), field_type);
                    
                    if field_name != field_def.name && self.options.derive_serde {
                        field.attributes.push(format!("serde(rename = \"{}\")", field_def.name));
                    }

                    field.optional = is_optional;
                    
                    ir_struct.add_field(field);
                }
                
                Some(ir_struct)
            }
             TypeDefinition::InputObject(obj) => {
                 // Also handle Input Objects as structs
                let mut ir_struct = IRStruct::new(obj.name.clone());

                if self.options.derive_serde {
                    ir_struct.add_derive("serde::Serialize".to_string());
                    ir_struct.add_derive("serde::Deserialize".to_string());
                }

                if self.options.derive_default {
                    ir_struct.add_derive("Default".to_string());
                }

                for field_def in &obj.fields {
                    let (mut field_type, mut is_optional) = self.map_graphql_type(&field_def.value_type);
                    
                    if self.options.make_fields_optional {
                        is_optional = true;
                    }

                    if is_optional {
                         field_type = field_type.make_optional();
                    }

                    let field_name = Self::to_snake_case(&field_def.name);
                    let mut field = IRField::new(field_name.clone(), field_type);

                    if field_name != field_def.name && self.options.derive_serde {
                        field.attributes.push(format!("serde(rename = \"{}\")", field_def.name));
                    }

                    field.optional = is_optional;
                    
                    ir_struct.add_field(field);
                }
                
                Some(ir_struct)
             }
            _ => None, // Enums, Interfaces, Unions, Scalars etc. ignored for now
        }
    }
}

impl Parser for GraphqlParser {
    type Error = GraphqlParserError;

    fn parse(&mut self, input: &str) -> Result<IRModule> {
        let ast = parse_schema::<String>(input)?;
        let mut module = IRModule::new("Schema".to_string());

        for def in ast.definitions {
            if let Definition::TypeDefinition(type_def) = def {
                if let Some(ir_struct) = self.process_type_definition(&type_def) {
                    module.add_type(IRType::Struct(ir_struct));
                }
            }
        }

        Ok(module)
    }

    fn name(&self) -> &'static str {
        "GraphQL"
    }

    fn extensions(&self) -> &[&'static str] {
        &["graphql", "gql"]
    }

    fn validate(&self, input: &str) -> Result<()> {
        parse_schema::<String>(input)?;
        Ok(())
    }

    fn metadata(&self) -> ParserMetadata {
         ParserMetadata::new()
            .with_version(env!("CARGO_PKG_VERSION"))
            .with_description("Parses GraphQL schemas to Rust structs")
    }
}
