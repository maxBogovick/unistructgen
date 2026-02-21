use crate::core::{
    IRField,
    IRModule,
    IRStruct,
    IRType,
    IRTypeRef,
    Parser,
    ParserMetadata,
    PrimitiveKind,
};
use std::collections::HashSet;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EnvParserError {
    #[error("Syntax error at line {line}: {message}")]
    SyntaxError {
        line: usize,
        message: String,
    },
}

pub type Result<T> = std::result::Result<T, EnvParserError>;

#[derive(Debug, Clone)]
pub struct EnvParserOptions {
    pub struct_name: String,
    pub derive_serde: bool,
    pub derive_default: bool,
    pub make_fields_optional: bool,
}

impl Default for EnvParserOptions {
    fn default() -> Self {
        Self {
            struct_name: "Config".to_string(),
            derive_serde: false,
            derive_default: false,
            make_fields_optional: false,
        }
    }
}

pub struct EnvParser {
    options: EnvParserOptions,
}

impl EnvParser {
    pub fn new(options: EnvParserOptions) -> Self {
        Self { options }
    }

    fn sanitize_field_name(name: &str) -> String {
        let name = name.trim();
        let mut result = String::new();
        
        for ch in name.chars() {
            if ch.is_ascii_alphanumeric() {
                result.push(ch.to_ascii_lowercase());
            } else if ch == '_' {
                result.push('_');
            }
        }
        
        // Ensure it doesn't start with a number
        if !result.is_empty() && result.chars().next().unwrap().is_numeric() {
            result.insert(0, '_');
        }

        if result.is_empty() {
             result = "unknown_field".to_string();
        }

        result
    }

    fn infer_type(value: &str) -> IRTypeRef {
        let value = value.trim();
        
        // Remove quotes if present
        let clean_value = if (value.starts_with('"') && value.ends_with('"')) || 
                             (value.starts_with('\'') && value.ends_with('\'')) {
            &value[1..value.len()-1]
        } else {
            value
        };

        if clean_value.eq_ignore_ascii_case("true") || clean_value.eq_ignore_ascii_case("false") {
            return IRTypeRef::Primitive(PrimitiveKind::Bool);
        }

        if let Ok(_) = clean_value.parse::<i64>() {
            return IRTypeRef::Primitive(PrimitiveKind::I64);
        }

        if let Ok(_) = clean_value.parse::<f64>() {
            return IRTypeRef::Primitive(PrimitiveKind::F64);
        }

        IRTypeRef::Primitive(PrimitiveKind::String)
    }
}

impl Parser for EnvParser {
    type Error = EnvParserError;

    fn parse(&mut self, input: &str) -> Result<IRModule> {
        let mut module = IRModule::new(self.options.struct_name.clone());
        let mut ir_struct = IRStruct::new(self.options.struct_name.clone());

        if self.options.derive_serde {
            ir_struct.add_derive("serde::Serialize".to_string());
            ir_struct.add_derive("serde::Deserialize".to_string());
        }

        if self.options.derive_default {
            ir_struct.add_derive("Default".to_string());
        }

        let mut processed_keys = HashSet::new();

        for line in input.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim();
                
                // Skip export prefix if present
                let key = if key.starts_with("export ") {
                    &key[7..]
                } else {
                    key
                };

                let field_name = Self::sanitize_field_name(key);
                
                if processed_keys.contains(&field_name) {
                    continue;
                }
                processed_keys.insert(field_name.clone());

                let field_type = Self::infer_type(value);
                
                let mut field = IRField::new(field_name.clone(), field_type);
                
                // If original key is different (e.g. uppercase), maybe add rename attribute if using serde
                // But for env files, usually we use libraries like `envy` which map uppercase to struct fields automatically or `dotenv`.
                // Here we just generate the struct to hold the data.
                
                if self.options.derive_serde && key != field_name {
                     field.attributes.push(format!("serde(rename = \"{}\")", key));
                }

                if self.options.make_fields_optional {
                    field.optional = true;
                    field.ty = field.ty.make_optional();
                }

                ir_struct.add_field(field);
            } else {
                 // Warning or ignore malformed lines?
                 // For now, ignore
            }
        }

        module.add_type(IRType::Struct(ir_struct));
        Ok(module)
    }

    fn name(&self) -> &'static str {
        "Env"
    }

    fn extensions(&self) -> &[&'static str] {
        &["env"]
    }

    fn validate(&self, _input: &str) -> Result<()> {
        Ok(())
    }

    fn metadata(&self) -> ParserMetadata {
        ParserMetadata::new()
            .with_version(env!("CARGO_PKG_VERSION"))
            .with_description("Parses .env files to Rust structs")
    }
}
