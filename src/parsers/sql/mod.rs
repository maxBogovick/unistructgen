use sqlparser::dialect::GenericDialect;
use sqlparser::parser::Parser as SqlParserImpl;
use sqlparser::ast::{Statement, DataType, ColumnDef};
use thiserror::Error;
use crate::core::{
    IRField,
    IRModule,
    IRStruct,
    IRType,
    IRTypeRef,
    PrimitiveKind,
    Parser,
    ParserMetadata,
};

#[derive(Error, Debug)]
pub enum SqlParserError {
    #[error("SQL parsing error: {0}")]
    ParseError(#[from] sqlparser::parser::ParserError),
    #[error("Unsupported SQL statement: {0}")]
    UnsupportedStatement(String),
}

pub type Result<T> = std::result::Result<T, SqlParserError>;

#[derive(Debug, Clone)]
pub struct SqlParserOptions {
    pub derive_serde: bool,
    pub derive_default: bool,
    pub make_fields_optional: bool,
}

impl Default for SqlParserOptions {
    fn default() -> Self {
        Self {
            derive_serde: true,
            derive_default: false,
            make_fields_optional: false,
        }
    }
}

pub struct SqlParser {
    options: SqlParserOptions,
}

impl SqlParser {
    pub fn new(options: SqlParserOptions) -> Self {
        Self { options }
    }

    fn map_sql_type(data_type: &DataType) -> IRTypeRef {
        match data_type {
            DataType::Integer(_) | DataType::Int(_) | DataType::SmallInt(_) | DataType::TinyInt(_) => IRTypeRef::Primitive(PrimitiveKind::I32),
            DataType::BigInt(_) => IRTypeRef::Primitive(PrimitiveKind::I64),
            DataType::UnsignedInt(_) | DataType::UnsignedInteger(_) => IRTypeRef::Primitive(PrimitiveKind::U32),
            DataType::UnsignedBigInt(_) => IRTypeRef::Primitive(PrimitiveKind::U64),
            DataType::Float(_) | DataType::Real | DataType::Double | DataType::DoublePrecision => IRTypeRef::Primitive(PrimitiveKind::F64),
            DataType::Decimal(_) | DataType::Numeric(_) => IRTypeRef::Primitive(PrimitiveKind::F64),
            DataType::Boolean => IRTypeRef::Primitive(PrimitiveKind::Bool),
            DataType::Char(_) | DataType::Varchar(_) | DataType::Text | DataType::String(_) => IRTypeRef::Primitive(PrimitiveKind::String),
            DataType::Timestamp(_, _) | DataType::Date | DataType::Time(_, _) => IRTypeRef::Primitive(PrimitiveKind::DateTime),
            DataType::Uuid => IRTypeRef::Primitive(PrimitiveKind::Uuid),
            DataType::JSON => IRTypeRef::Primitive(PrimitiveKind::Json),
            DataType::Custom(name, _modifiers) if name.to_string().eq_ignore_ascii_case("SERIAL") => {
                IRTypeRef::Primitive(PrimitiveKind::I32)
            }
            DataType::Custom(name, _modifiers) if name.to_string().eq_ignore_ascii_case("BIGSERIAL") => {
                IRTypeRef::Primitive(PrimitiveKind::I64)
            }
            _ => IRTypeRef::Primitive(PrimitiveKind::String), // Fallback
        }
    }

    fn process_create_table(&self, name: &str, columns: &[ColumnDef]) -> IRStruct {
        // Convert table name to PascalCase for struct name
        let struct_name = self.to_pascal_case(name);
        let mut ir_struct = IRStruct::new(struct_name);

        if self.options.derive_serde {
            ir_struct.add_derive("serde::Serialize".to_string());
            ir_struct.add_derive("serde::Deserialize".to_string());
        }

        if self.options.derive_default {
            ir_struct.add_derive("Default".to_string());
        }

        for col in columns {
            let field_name = col.name.value.to_lowercase();
            let field_type = Self::map_sql_type(&col.data_type);
            
            // Check if column is nullable (not null is usually explicit)
            // Default assumption in SQL: nullable unless NOT NULL specified
            let mut is_optional = true;
            for option in &col.options {
                match option.option {
                    sqlparser::ast::ColumnOption::NotNull => {
                        is_optional = false;
                    }
                    sqlparser::ast::ColumnOption::Unique { is_primary: true, .. } => {
                        is_optional = false;
                    }
                    _ => {}
                }
            }
            
            // Override with global option
            if self.options.make_fields_optional {
                is_optional = true;
            }

            let mut field = IRField::new(field_name.clone(), field_type);
            
            if is_optional {
                field.optional = true;
                field.ty = field.ty.make_optional();
            }

            // Handle rename if needed (e.g. if field name matches Rust keyword)
            if self.is_rust_keyword(&field_name) {
                 field.name = format!("{}_", field_name);
                 if self.options.derive_serde {
                     field.attributes.push(format!("serde(rename = \"{}\")", field_name));
                 }
            }

            ir_struct.add_field(field);
        }

        ir_struct
    }

    fn to_pascal_case(&self, s: &str) -> String {
        let mut result = String::new();
        let mut capitalize = true;
        for c in s.chars() {
            if c == '_' || c == '-' {
                capitalize = true;
            } else if capitalize {
                result.push(c.to_ascii_uppercase());
                capitalize = false;
            } else {
                result.push(c.to_ascii_lowercase());
            }
        }
        result
    }

    fn is_rust_keyword(&self, name: &str) -> bool {
        matches!(
            name,
            "as" | "break" | "const" | "continue" | "crate" | "else" | "enum" | "extern" | "false" | "fn" | "for" | "if" | "impl" | "in" | "let" | "loop" | "match" | "mod" | "move" | "mut" | "pub" | "ref" | "return" | "self" | "Self" | "static" | "struct" | "super" | "trait" | "true" | "type" | "unsafe" | "use" | "where" | "while" | "async" | "await" | "dyn"
        )
    }
}

impl Parser for SqlParser {
    type Error = SqlParserError;

    fn parse(&mut self, input: &str) -> Result<IRModule> {
        let dialect = GenericDialect {};
        let ast = SqlParserImpl::parse_sql(&dialect, input)?;
        
        let mut module = IRModule::new("Schema".to_string());

        for statement in ast {
            if let Statement::CreateTable { name, columns, .. } = statement {
                // name is ObjectName (Vec<Ident>)
                let table_name = name.0.last().unwrap().value.clone();
                let ir_struct = self.process_create_table(&table_name, &columns);
                module.add_type(IRType::Struct(ir_struct));
            }
        }

        Ok(module)
    }

    fn name(&self) -> &'static str {
        "SQL"
    }

    fn extensions(&self) -> &[&'static str] {
        &["sql"]
    }

    fn validate(&self, input: &str) -> Result<()> {
        let dialect = GenericDialect {};
        SqlParserImpl::parse_sql(&dialect, input)?;
        Ok(())
    }

    fn metadata(&self) -> ParserMetadata {
        ParserMetadata::new()
            .with_version(env!("CARGO_PKG_VERSION"))
            .with_description("Parses SQL CREATE TABLE statements to Rust structs")
    }
}
