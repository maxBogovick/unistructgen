pub mod chunker;

use pulldown_cmark::{Event, Parser as CmarkParser, Tag, Options as CmarkOptions};
use thiserror::Error;
use crate::core::{
    IRField, IRModule, IRStruct, IRType, IRTypeRef, Parser, ParserMetadata, PrimitiveKind,
};

#[derive(Error, Debug)]
pub enum MarkdownParserError {
    #[error("No tables found in markdown input")]
    NoTablesFound,
    
    #[error("Table mapping error: {0}")]
    MappingError(String),
    
    #[error("Invalid column structure: {0}")]
    InvalidColumns(String),
}

pub type Result<T> = std::result::Result<T, MarkdownParserError>;

#[derive(Debug, Clone)]
pub struct MarkdownParserOptions {
    pub struct_name: String,
    pub derive_serde: bool,
    pub derive_default: bool,
    pub make_fields_optional: bool,
}

impl Default for MarkdownParserOptions {
    fn default() -> Self {
        Self {
            struct_name: "Root".to_string(),
            derive_serde: true,
            derive_default: false,
            make_fields_optional: false,
        }
    }
}

pub struct MarkdownParser {
    options: MarkdownParserOptions,
}

impl MarkdownParser {
    pub fn new(options: MarkdownParserOptions) -> Self {
        Self { options }
    }

    fn parse_type(&self, type_str: &str) -> IRTypeRef {
        let type_lower = type_str.to_lowercase();
        let type_clean = type_lower.trim();

        if type_clean.starts_with("vec<") || type_clean.starts_with("array<") || type_clean.ends_with("[]") {
            // Basic array handling
            let inner = if type_clean.ends_with("[]") {
                &type_clean[..type_clean.len()-2]
            } else if let Some(start) = type_clean.find('<') {
                if let Some(end) = type_clean.rfind('>') {
                    &type_clean[start+1..end]
                } else {
                    "string"
                }
            } else {
                "string"
            };
            return IRTypeRef::Vec(Box::new(self.parse_type(inner)));
        }

        match type_clean {
            "string" | "text" | "varchar" | "str" => IRTypeRef::Primitive(PrimitiveKind::String),
            "integer" | "int" | "i32" | "number" => IRTypeRef::Primitive(PrimitiveKind::I32),
            "long" | "i64" | "bigint" => IRTypeRef::Primitive(PrimitiveKind::I64),
            "float" | "double" | "f64" | "decimal" => IRTypeRef::Primitive(PrimitiveKind::F64),
            "boolean" | "bool" => IRTypeRef::Primitive(PrimitiveKind::Bool),
            "date" | "datetime" | "timestamp" => IRTypeRef::Primitive(PrimitiveKind::DateTime),
            "uuid" | "guid" => IRTypeRef::Primitive(PrimitiveKind::Uuid),
            "json" | "object" | "map" => IRTypeRef::Primitive(PrimitiveKind::Json),
            _ => IRTypeRef::Named(Self::to_pascal_case(type_clean)),
        }
    }

    fn to_pascal_case(s: &str) -> String {
        let mut result = String::new();
        let mut capitalize_next = true;
        for c in s.chars() {
            if c.is_alphanumeric() {
                if capitalize_next {
                    result.push(c.to_ascii_uppercase());
                    capitalize_next = false;
                } else {
                    result.push(c);
                }
            } else {
                capitalize_next = true;
            }
        }
        if result.is_empty() {
            "Unknown".to_string()
        } else {
            result
        }
    }
    
    fn sanitize_field_name(s: &str) -> String {
        let mut result = String::new();
        for c in s.chars() {
            if c.is_alphanumeric() || c == '_' {
                result.push(c.to_ascii_lowercase());
            } else if c == ' ' || c == '-' {
                result.push('_');
            }
        }
        
        if result.chars().next().map_or(false, |c| c.is_numeric()) {
            result.insert(0, '_');
        }
        
        // Handle keywords
        match result.as_str() {
            "type" | "struct" | "enum" | "fn" | "let" | "const" => format!("{}_field", result),
            _ => result,
        }
    }
}

impl Parser for MarkdownParser {
    type Error = MarkdownParserError;

    fn parse(&mut self, input: &str) -> std::result::Result<IRModule, Self::Error> {
        let mut module = IRModule::new(self.options.struct_name.clone());
        let mut options = CmarkOptions::empty();
        options.insert(CmarkOptions::ENABLE_TABLES);
        let parser = CmarkParser::new_ext(input, options);

        let current_struct_name = self.options.struct_name.clone();
        let mut inside_table = false;
        let mut table_headers: Vec<String> = Vec::new();
        let mut table_rows: Vec<Vec<String>> = Vec::new();
        let mut current_row: Vec<String> = Vec::new();
        let mut current_cell = String::new();
        
        // Simple state machine
        for event in parser {
            match event {
                Event::Start(Tag::Heading(_, _, _)) => {
                    // Reset or capture potential struct name from heading
                    // For now, we stick to the main name or heuristics could be added here
                }
                Event::Start(Tag::Table(_)) => {
                    inside_table = true;
                    table_headers.clear();
                    table_rows.clear();
                }
                Event::End(Tag::Table(_)) => {
                    inside_table = false;
                    
                    if table_headers.is_empty() {
                        continue;
                    }

                    // Process the table we just finished
                    let mut ir_struct = IRStruct::new(current_struct_name.clone());
                    
                    if self.options.derive_serde {
                        ir_struct.add_derive("serde::Serialize".to_string());
                        ir_struct.add_derive("serde::Deserialize".to_string());
                    }
                    if self.options.derive_default {
                        ir_struct.add_derive("Default".to_string());
                    }

                    // Identify column indices
                    let mut name_idx = None;
                    let mut type_idx = None;
                    let mut desc_idx = None;
                    let mut required_idx = None;

                    for (i, header) in table_headers.iter().enumerate() {
                        let h = header.to_lowercase();
                        if h.contains("name") || h.contains("field") || h.contains("property") {
                            name_idx = Some(i);
                        } else if h.contains("type") {
                            type_idx = Some(i);
                        } else if h.contains("desc") || h.contains("comment") {
                            desc_idx = Some(i);
                        } else if h.contains("required") || h.contains("optional") {
                            required_idx = Some(i);
                        }
                    }

                    if let (Some(n_idx), Some(t_idx)) = (name_idx, type_idx) {
                        for row in &table_rows {
                            if row.len() <= n_idx || row.len() <= t_idx {
                                continue;
                            }
                            
                            let raw_name = &row[n_idx];
                            let raw_type = &row[t_idx];
                            
                            let field_name = Self::sanitize_field_name(raw_name);
                            let mut field_type = self.parse_type(raw_type);
                            
                            let mut is_optional = self.options.make_fields_optional;
                            
                            // Check explicit required/optional column
                            if let Some(r_idx) = required_idx {
                                if row.len() > r_idx {
                                    let req_text = row[r_idx].to_lowercase();
                                    if req_text.contains("no") || req_text.contains("false") || req_text.contains("optional") {
                                        is_optional = true;
                                    }
                                }
                            }
                            
                            // Check description for hints
                            let doc = if let Some(d_idx) = desc_idx {
                                if row.len() > d_idx {
                                    let text = &row[d_idx];
                                    if text.to_lowercase().contains("optional") {
                                        is_optional = true;
                                    }
                                    Some(text.clone())
                                } else {
                                    None
                                }
                            } else {
                                None
                            };

                            if is_optional {
                                field_type = field_type.make_optional();
                            }

                            let mut field = IRField::new(field_name, field_type);
                            field.optional = is_optional;
                            
                            field.doc = doc;
                            if raw_name != &field.name {
                                field.source_name = Some(raw_name.clone());
                                if self.options.derive_serde {
                                     field.attributes.push(format!("serde(rename = \"{}\")", raw_name));
                                }
                            }
                            
                            ir_struct.add_field(field);
                        }
                        
                        module.add_type(IRType::Struct(ir_struct));
                        // If we found a valid table, we might stop or continue to find others
                        // For this version, let's allow multiple tables to define multiple structs?
                        // But we need unique names. 
                        // Let's just generate one for now and break, or use the heading
                         break; 
                    }
                }
                Event::Start(Tag::TableHead) => {
                    current_row.clear();
                }
                Event::End(Tag::TableHead) => {
                     table_headers = current_row.clone();
                }
                Event::Start(Tag::TableRow) => {
                    current_row.clear();
                }
                Event::End(Tag::TableRow) => {
                    if inside_table {
                         // If we are here, it's a body row because headers are handled in TableHead
                         table_rows.push(current_row.clone());
                    }
                }
                Event::Start(Tag::TableCell) => {
                    current_cell.clear();
                }
                Event::End(Tag::TableCell) => {
                    current_row.push(current_cell.trim().to_string());
                }
                Event::Text(text) => {
                    current_cell.push_str(&text);
                }
                Event::Code(text) => {
                    current_cell.push_str(&text);
                }
                Event::Html(text) => {
                    current_cell.push_str(&text);
                }
                _ => {}
            }
        }

        if module.types.is_empty() {
            return Err(MarkdownParserError::NoTablesFound);
        }

        Ok(module)
    }

    fn name(&self) -> &'static str {
        "Markdown"
    }

    fn extensions(&self) -> &[&'static str] {
        &["md", "markdown"]
    }

    fn validate(&self, _input: &str) -> std::result::Result<(), Self::Error> {
        Ok(())
    }

    fn metadata(&self) -> ParserMetadata {
        ParserMetadata::new()
            .with_version(env!("CARGO_PKG_VERSION"))
            .with_description("Parses Markdown tables into structs")
    }
}
