use crate::ir::{FieldConstraints, IRField, IRStruct, IRType, IRTypeRef, PrimitiveKind};

/// Trait to convert Rust types into UniStructGen IR.
///
/// This allows "Reverse IR": deriving IR from existing Rust code.
pub trait IntoIR {
    /// Returns the type reference for this type (e.g., "String", "i32", or "MyStruct").
    /// For complex types (Structs/Enums), this returns `IRTypeRef::Named`.
    fn ir_type_ref() -> IRTypeRef;

    /// Returns the full IR definition (Struct or Enum) if applicable.
    /// Returns `None` for primitive types, `Option`, `Vec`, etc.
    fn ir_definition() -> Option<IRType> {
        None
    }
}

// --- Primitive Implementations ---

macro_rules! impl_primitive {
    ($ty:ty, $kind:expr) => {
        impl IntoIR for $ty {
            fn ir_type_ref() -> IRTypeRef {
                IRTypeRef::Primitive($kind)
            }
        }
    };
}

impl_primitive!(String, PrimitiveKind::String);
impl_primitive!(i8, PrimitiveKind::I8);
impl_primitive!(i16, PrimitiveKind::I16);
impl_primitive!(i32, PrimitiveKind::I32);
impl_primitive!(i64, PrimitiveKind::I64);
impl_primitive!(i128, PrimitiveKind::I128);
impl_primitive!(u8, PrimitiveKind::U8);
impl_primitive!(u16, PrimitiveKind::U16);
impl_primitive!(u32, PrimitiveKind::U32);
impl_primitive!(u64, PrimitiveKind::U64);
impl_primitive!(u128, PrimitiveKind::U128);
impl_primitive!(f32, PrimitiveKind::F32);
impl_primitive!(f64, PrimitiveKind::F64);
impl_primitive!(bool, PrimitiveKind::Bool);
impl_primitive!(char, PrimitiveKind::Char);

// --- Special Types (heuristic based on common usage) ---

// Assuming uuid::Uuid and chrono::DateTime might be available or we treat them as primitives in IR
// Since `core` doesn't strictly depend on `uuid` or `chrono` unless features are enabled,
// we might not be able to impl them directly here unless we use a wrapper or the user uses `#[field(type="...")]`.
// However, the IR supports them. For now, we only implement standard library types.

// --- Container Implementations ---

impl<T: IntoIR> IntoIR for Option<T> {
    fn ir_type_ref() -> IRTypeRef {
        IRTypeRef::Option(Box::new(T::ir_type_ref()))
    }
}

impl<T: IntoIR> IntoIR for Vec<T> {
    fn ir_type_ref() -> IRTypeRef {
        IRTypeRef::Vec(Box::new(T::ir_type_ref()))
    }
}

// TODO: Implement for HashMap once we have Map support fully consistent
