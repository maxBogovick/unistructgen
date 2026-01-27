use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

/// A type-safe container for dependency injection.
/// Stores arbitrary types indexed by their TypeId.
#[derive(Default, Clone)]
pub struct Context {
    map: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    /// Insert a dependency into the context.
    pub fn insert<T: Send + Sync + 'static>(&mut self, value: T) {
        self.map.insert(TypeId::of::<T>(), Arc::new(value));
    }

    /// Get a dependency reference from the context.
    pub fn get<T: Send + Sync + 'static>(&self) -> Option<&T> {
        self.map
            .get(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast_ref())
    }

    /// Require a dependency, returning an error if missing.
    /// Useful for tools to fail gracefully.
    pub fn require<T: Send + Sync + 'static>(&self) -> Result<&T, String> {
        self.get::<T>()
            .ok_or_else(|| format!("Missing dependency in context: {}", std::any::type_name::<T>()))
    }
}
