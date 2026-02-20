//! Visitor pattern for traversing and modifying IR
//!
//! The visitor pattern allows you to traverse the IR tree and perform operations
//! on each node without modifying the IR structure itself.

use crate::core::{IREnum, IRField, IRModule, IRStruct, IRType, IRTypeRef};

/// Trait for visiting IR nodes
///
/// The visitor pattern provides a clean way to traverse and modify the IR tree.
/// Each `visit_*` method has a default implementation that calls the corresponding
/// `walk_*` function, which recursively visits child nodes.
///
/// # Examples
///
/// ```
/// use unistructgen::core::{IRVisitor, IRModule, IRStruct, walk_struct};
///
/// struct StructCounter {
///     count: usize,
/// }
///
/// impl IRVisitor for StructCounter {
///     fn visit_struct(&mut self, struct_: &mut IRStruct) {
///         self.count += 1;
///         // Don't forget to walk children if needed
///         walk_struct(self, struct_);
///     }
/// }
/// ```
pub trait IRVisitor {
    /// Visit a module
    ///
    /// Default implementation walks all types in the module.
    fn visit_module(&mut self, module: &mut IRModule) {
        walk_module(self, module);
    }

    /// Visit a type (struct or enum)
    ///
    /// Default implementation dispatches to `visit_struct` or `visit_enum`.
    fn visit_type(&mut self, ty: &mut IRType) {
        walk_type(self, ty);
    }

    /// Visit a struct
    ///
    /// Default implementation walks all fields.
    fn visit_struct(&mut self, struct_: &mut IRStruct) {
        walk_struct(self, struct_);
    }

    /// Visit an enum
    ///
    /// Default implementation does nothing (no child nodes to visit).
    fn visit_enum(&mut self, enum_: &mut IREnum) {
        walk_enum(self, enum_);
    }

    /// Visit a field
    ///
    /// Default implementation visits the field's type reference.
    fn visit_field(&mut self, field: &mut IRField) {
        walk_field(self, field);
    }

    /// Visit a type reference
    ///
    /// Default implementation walks nested type references (e.g., inside Option, Vec).
    fn visit_type_ref(&mut self, type_ref: &mut IRTypeRef) {
        walk_type_ref(self, type_ref);
    }
}

/// Walk a module, visiting all its types
pub fn walk_module<V: IRVisitor + ?Sized>(visitor: &mut V, module: &mut IRModule) {
    for ty in &mut module.types {
        visitor.visit_type(ty);
    }
}

/// Walk a type, dispatching to struct or enum visitor
pub fn walk_type<V: IRVisitor + ?Sized>(visitor: &mut V, ty: &mut IRType) {
    match ty {
        IRType::Struct(s) => visitor.visit_struct(s),
        IRType::Enum(e) => visitor.visit_enum(e),
    }
}

/// Walk a struct, visiting all its fields
pub fn walk_struct<V: IRVisitor + ?Sized>(visitor: &mut V, struct_: &mut IRStruct) {
    for field in &mut struct_.fields {
        visitor.visit_field(field);
    }
}

/// Walk an enum (currently no-op, but here for consistency)
pub fn walk_enum<V: IRVisitor + ?Sized>(_visitor: &mut V, _enum_: &mut IREnum) {
    // Enums don't have child nodes to visit yet
    // When we add enum variants with fields, this will visit them
}

/// Walk a field, visiting its type reference
pub fn walk_field<V: IRVisitor + ?Sized>(visitor: &mut V, field: &mut IRField) {
    visitor.visit_type_ref(&mut field.ty);
}

/// Walk a type reference, visiting nested type references
pub fn walk_type_ref<V: IRVisitor + ?Sized>(visitor: &mut V, type_ref: &mut IRTypeRef) {
    match type_ref {
        IRTypeRef::Option(inner) => visitor.visit_type_ref(inner),
        IRTypeRef::Vec(inner) => visitor.visit_type_ref(inner),
        IRTypeRef::Map(key, value) => {
            visitor.visit_type_ref(key);
            visitor.visit_type_ref(value);
        }
        IRTypeRef::Primitive(_) | IRTypeRef::Named(_) => {
            // Leaf nodes, nothing to walk
        }
    }
}

/// Example visitor: Collects all struct names
///
/// # Examples
///
/// ```
/// use unistructgen::core::{IRModule, IRStruct, IRType, IRVisitor, visitor::StructNameCollector};
///
/// let mut module = IRModule::new("Test".to_string());
/// module.add_type(IRType::Struct(IRStruct::new("User".to_string())));
/// module.add_type(IRType::Struct(IRStruct::new("Post".to_string())));
///
/// let mut collector = StructNameCollector::new();
/// collector.visit_module(&mut module);
///
/// assert_eq!(collector.names(), &["User", "Post"]);
/// ```
pub struct StructNameCollector {
    names: Vec<String>,
}

impl StructNameCollector {
    /// Create a new struct name collector
    pub fn new() -> Self {
        Self { names: Vec::new() }
    }

    /// Get the collected struct names
    pub fn names(&self) -> &[String] {
        &self.names
    }

    /// Take ownership of the collected names
    pub fn into_names(self) -> Vec<String> {
        self.names
    }
}

impl Default for StructNameCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl IRVisitor for StructNameCollector {
    fn visit_struct(&mut self, struct_: &mut IRStruct) {
        self.names.push(struct_.name.clone());
        // Don't walk children - we only care about the struct name
    }
}

/// Example visitor: Counts fields across all structs
///
/// # Examples
///
/// ```
/// use unistructgen::core::{IRModule, IRStruct, IRField, IRType, IRTypeRef, PrimitiveKind, IRVisitor, visitor::FieldCounter};
///
/// let mut module = IRModule::new("Test".to_string());
/// let mut s = IRStruct::new("User".to_string());
/// s.add_field(IRField::new("id".to_string(), IRTypeRef::Primitive(PrimitiveKind::I64)));
/// s.add_field(IRField::new("name".to_string(), IRTypeRef::Primitive(PrimitiveKind::String)));
/// module.add_type(IRType::Struct(s));
///
/// let mut counter = FieldCounter::new();
/// counter.visit_module(&mut module);
///
/// assert_eq!(counter.count(), 2);
/// ```
pub struct FieldCounter {
    count: usize,
}

impl FieldCounter {
    /// Create a new field counter
    pub fn new() -> Self {
        Self { count: 0 }
    }

    /// Get the field count
    pub fn count(&self) -> usize {
        self.count
    }
}

impl Default for FieldCounter {
    fn default() -> Self {
        Self::new()
    }
}

impl IRVisitor for FieldCounter {
    fn visit_field(&mut self, field: &mut IRField) {
        self.count += 1;
        walk_field(self, field); // Walk to count nested types if needed
    }
}

/// Example visitor: Collects all primitive types used
///
/// # Examples
///
/// ```
/// use unistructgen::core::{IRModule, IRStruct, IRField, IRType, IRTypeRef, PrimitiveKind, IRVisitor, visitor::PrimitiveTypeCollector};
///
/// let mut module = IRModule::new("Test".to_string());
/// let mut s = IRStruct::new("User".to_string());
/// s.add_field(IRField::new("id".to_string(), IRTypeRef::Primitive(PrimitiveKind::I64)));
/// s.add_field(IRField::new("name".to_string(), IRTypeRef::Primitive(PrimitiveKind::String)));
/// module.add_type(IRType::Struct(s));
///
/// let mut collector = PrimitiveTypeCollector::new();
/// collector.visit_module(&mut module);
///
/// let types = collector.types();
/// assert!(types.contains(&PrimitiveKind::I64));
/// assert!(types.contains(&PrimitiveKind::String));
/// ```
pub struct PrimitiveTypeCollector {
    types: std::collections::HashSet<crate::core::PrimitiveKind>,
}

impl PrimitiveTypeCollector {
    /// Create a new primitive type collector
    pub fn new() -> Self {
        Self {
            types: std::collections::HashSet::new(),
        }
    }

    /// Get the collected primitive types
    pub fn types(&self) -> &std::collections::HashSet<crate::core::PrimitiveKind> {
        &self.types
    }

    /// Take ownership of the collected types
    pub fn into_types(self) -> std::collections::HashSet<crate::core::PrimitiveKind> {
        self.types
    }
}

impl Default for PrimitiveTypeCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl IRVisitor for PrimitiveTypeCollector {
    fn visit_type_ref(&mut self, type_ref: &mut IRTypeRef) {
        if let IRTypeRef::Primitive(kind) = type_ref {
            self.types.insert(*kind);
        }
        walk_type_ref(self, type_ref); // Walk nested types
    }
}

/// Example visitor: Makes all fields in all structs public
///
/// This demonstrates mutating the IR during traversal.
pub struct FieldPublicizer;

impl FieldPublicizer {
    /// Create a new field publicizer
    pub fn new() -> Self {
        Self
    }
}

impl Default for FieldPublicizer {
    fn default() -> Self {
        Self::new()
    }
}

impl IRVisitor for FieldPublicizer {
    fn visit_field(&mut self, field: &mut IRField) {
        // In the future, when we add visibility to fields,
        // this would set field.visibility = Visibility::Public
        walk_field(self, field);
    }
}

/// Example visitor: Validates IR structure
///
/// Checks for common issues like:
/// - Empty struct names
/// - Empty field names
/// - Structs with no fields
///
/// # Examples
///
/// ```
/// use unistructgen::core::{IRModule, IRStruct, IRType, IRVisitor, visitor::IRValidator};
///
/// let mut module = IRModule::new("Test".to_string());
/// let mut empty_struct = IRStruct::new("Empty".to_string());
/// // No fields added
/// module.add_type(IRType::Struct(empty_struct));
///
/// let mut validator = IRValidator::new();
/// validator.visit_module(&mut module);
///
/// assert!(validator.has_errors());
/// assert!(validator.errors().iter().any(|e| e.contains("no fields")));
/// ```
pub struct IRValidator {
    errors: Vec<String>,
}

impl IRValidator {
    /// Create a new IR validator
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    /// Check if any errors were found
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Get the validation errors
    pub fn errors(&self) -> &[String] {
        &self.errors
    }

    /// Take ownership of the errors
    pub fn into_errors(self) -> Vec<String> {
        self.errors
    }
}

impl Default for IRValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl IRVisitor for IRValidator {
    fn visit_struct(&mut self, struct_: &mut IRStruct) {
        // Check struct name
        if struct_.name.is_empty() {
            self.errors.push("Struct has empty name".to_string());
        }

        // Check for fields
        if struct_.fields.is_empty() {
            self.errors
                .push(format!("Struct '{}' has no fields", struct_.name));
        }

        walk_struct(self, struct_);
    }

    fn visit_field(&mut self, field: &mut IRField) {
        // Check field name
        if field.name.is_empty() {
            self.errors.push("Field has empty name".to_string());
        }

        walk_field(self, field);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{IRTypeRef, PrimitiveKind};

    fn create_test_module() -> IRModule {
        let mut module = IRModule::new("Test".to_string());

        let mut user_struct = IRStruct::new("User".to_string());
        user_struct.add_field(IRField::new(
            "id".to_string(),
            IRTypeRef::Primitive(PrimitiveKind::I64),
        ));
        user_struct.add_field(IRField::new(
            "name".to_string(),
            IRTypeRef::Primitive(PrimitiveKind::String),
        ));

        let mut post_struct = IRStruct::new("Post".to_string());
        post_struct.add_field(IRField::new(
            "title".to_string(),
            IRTypeRef::Primitive(PrimitiveKind::String),
        ));

        module.add_type(IRType::Struct(user_struct));
        module.add_type(IRType::Struct(post_struct));

        module
    }

    #[test]
    fn test_struct_name_collector() {
        let mut module = create_test_module();
        let mut collector = StructNameCollector::new();

        collector.visit_module(&mut module);

        assert_eq!(collector.names(), &["User", "Post"]);
    }

    #[test]
    fn test_field_counter() {
        let mut module = create_test_module();
        let mut counter = FieldCounter::new();

        counter.visit_module(&mut module);

        assert_eq!(counter.count(), 3); // 2 in User + 1 in Post
    }

    #[test]
    fn test_primitive_type_collector() {
        let mut module = create_test_module();
        let mut collector = PrimitiveTypeCollector::new();

        collector.visit_module(&mut module);

        let types = collector.types();
        assert!(types.contains(&PrimitiveKind::I64));
        assert!(types.contains(&PrimitiveKind::String));
        assert_eq!(types.len(), 2);
    }

    #[test]
    fn test_validator_empty_struct() {
        let mut module = IRModule::new("Test".to_string());
        let empty_struct = IRStruct::new("Empty".to_string());
        module.add_type(IRType::Struct(empty_struct));

        let mut validator = IRValidator::new();
        validator.visit_module(&mut module);

        assert!(validator.has_errors());
        assert!(validator
            .errors()
            .iter()
            .any(|e| e.contains("no fields")));
    }

    #[test]
    fn test_validator_valid_struct() {
        let mut module = create_test_module();
        let mut validator = IRValidator::new();

        validator.visit_module(&mut module);

        assert!(!validator.has_errors());
    }

    #[test]
    fn test_walk_nested_types() {
        let mut module = IRModule::new("Test".to_string());

        let mut s = IRStruct::new("Nested".to_string());
        // Option<Vec<String>>
        s.add_field(IRField::new(
            "data".to_string(),
            IRTypeRef::Option(Box::new(IRTypeRef::Vec(Box::new(IRTypeRef::Primitive(
                PrimitiveKind::String,
            ))))),
        ));

        module.add_type(IRType::Struct(s));

        let mut collector = PrimitiveTypeCollector::new();
        collector.visit_module(&mut module);

        // Should find String even though it's nested in Option<Vec<>>
        assert!(collector.types().contains(&PrimitiveKind::String));
    }
}
