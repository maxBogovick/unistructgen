use serde::{Deserialize, Serialize};

/// Intermediate Representation Module
/// Contains all types that will be generated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IRModule {
    pub name: String,
    pub types: Vec<IRType>,
}

/// Type definition in the IR
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IRType {
    Struct(IRStruct),
    Enum(IREnum),
}

/// Struct definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IRStruct {
    pub name: String,
    pub fields: Vec<IRField>,
    pub derives: Vec<String>,
    pub doc: Option<String>,
    pub attributes: Vec<String>,
}

/// Enum definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IREnum {
    pub name: String,
    pub variants: Vec<IREnumVariant>,
    pub derives: Vec<String>,
    pub doc: Option<String>,
}

/// Enum variant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IREnumVariant {
    pub name: String,
    /// Original value from the source (e.g., "open" vs "Open")
    pub source_value: Option<String>,
    pub doc: Option<String>,
}

/// Field in a struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IRField {
    pub name: String,
    /// Original name in the source (e.g., JSON key)
    pub source_name: Option<String>,
    pub ty: IRTypeRef,
    pub optional: bool,
    pub default: Option<String>,
    pub constraints: FieldConstraints,
    pub attributes: Vec<String>,
    pub doc: Option<String>,
}

/// Type reference
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum IRTypeRef {
    /// Primitive types (String, i32, f64, bool, etc.)
    Primitive(PrimitiveKind),
    /// Option<T>
    Option(Box<IRTypeRef>),
    /// Vec<T>
    Vec(Box<IRTypeRef>),
    /// Named type (struct/enum reference)
    Named(String),
    /// Map/HashMap<K, V>
    Map(Box<IRTypeRef>, Box<IRTypeRef>),
}

/// Primitive type kinds
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PrimitiveKind {
    String,
    I8,
    I16,
    I32,
    I64,
    I128,
    U8,
    U16,
    U32,
    U64,
    U128,
    F32,
    F64,
    Bool,
    Char,
    // Special types
    DateTime,
    Uuid,
    Decimal,
    Json,
}

impl PrimitiveKind {
    pub fn rust_type_name(&self) -> &'static str {
        match self {
            PrimitiveKind::String => "String",
            PrimitiveKind::I8 => "i8",
            PrimitiveKind::I16 => "i16",
            PrimitiveKind::I32 => "i32",
            PrimitiveKind::I64 => "i64",
            PrimitiveKind::I128 => "i128",
            PrimitiveKind::U8 => "u8",
            PrimitiveKind::U16 => "u16",
            PrimitiveKind::U32 => "u32",
            PrimitiveKind::U64 => "u64",
            PrimitiveKind::U128 => "u128",
            PrimitiveKind::F32 => "f32",
            PrimitiveKind::F64 => "f64",
            PrimitiveKind::Bool => "bool",
            PrimitiveKind::Char => "char",
            PrimitiveKind::DateTime => "chrono::DateTime<chrono::Utc>",
            PrimitiveKind::Uuid => "uuid::Uuid",
            PrimitiveKind::Decimal => "rust_decimal::Decimal",
            PrimitiveKind::Json => "serde_json::Value",
        }
    }
}

/// Field constraints for validation
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FieldConstraints {
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
    pub pattern: Option<String>,
    pub format: Option<String>,
}

impl IRModule {
    pub fn new(name: String) -> Self {
        Self {
            name,
            types: Vec::new(),
        }
    }

    pub fn add_type(&mut self, ty: IRType) {
        self.types.push(ty);
    }
}

impl IRStruct {
    pub fn new(name: String) -> Self {
        Self {
            name,
            fields: Vec::new(),
            derives: vec![
                "Debug".to_string(),
                "Clone".to_string(),
                "PartialEq".to_string(),
            ],
            doc: None,
            attributes: Vec::new(),
        }
    }

    pub fn add_field(&mut self, field: IRField) {
        self.fields.push(field);
    }

    pub fn add_derive(&mut self, derive: String) {
        if !self.derives.contains(&derive) {
            self.derives.push(derive);
        }
    }
}

impl IRField {
    pub fn new(name: String, ty: IRTypeRef) -> Self {
        Self {
            name,
            source_name: None,
            ty,
            optional: false,
            default: None,
            constraints: FieldConstraints::default(),
            attributes: Vec::new(),
            doc: None,
        }
    }
}

impl IRTypeRef {
    pub fn is_primitive(&self) -> bool {
        matches!(self, IRTypeRef::Primitive(_))
    }

    pub fn is_optional(&self) -> bool {
        matches!(self, IRTypeRef::Option(_))
    }

    pub fn make_optional(self) -> Self {
        match self {
            IRTypeRef::Option(_) => self,
            other => IRTypeRef::Option(Box::new(other)),
        }
    }
}
