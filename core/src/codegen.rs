//! Code generator trait and related types
//!
//! This module defines the core trait that all code generators must implement.
//! It provides a unified interface for generating code in different programming
//! languages from the Intermediate Representation (IR).

use crate::IRModule;
use std::error::Error as StdError;

/// Result type alias for code generation operations
pub type CodegenResult<T> = std::result::Result<T, Box<dyn StdError + Send + Sync>>;

/// Core trait for all code generators
///
/// Implement this trait to add support for generating code in new languages.
/// The generator is responsible for converting an IR module into syntactically
/// correct code in the target language.
///
/// # Type Parameters
///
/// * `Error` - The error type produced by this generator. Must implement
///   `std::error::Error + Send + Sync + 'static` for composability.
///
/// # Examples
///
/// ```ignore
/// use unistructgen_core::{CodeGenerator, IRModule};
///
/// struct MyGenerator {
///     options: MyOptions,
/// }
///
/// impl CodeGenerator for MyGenerator {
///     type Error = MyGeneratorError;
///
///     fn generate(&self, module: &IRModule) -> Result<String, Self::Error> {
///         // Generate code from IR
///         todo!()
///     }
///
///     fn language(&self) -> &'static str {
///         "MyLanguage"
///     }
///
///     fn file_extension(&self) -> &str {
///         "my"
///     }
/// }
/// ```
pub trait CodeGenerator {
    /// The error type this generator produces
    type Error: StdError + Send + Sync + 'static;

    /// Generate code from an IR module
    ///
    /// # Arguments
    ///
    /// * `module` - The IR module to generate code from
    ///
    /// # Returns
    ///
    /// Returns `Ok(String)` containing the generated code on success.
    ///
    /// # Errors
    ///
    /// Returns `Self::Error` if code generation fails. The error should
    /// contain detailed information about what went wrong.
    fn generate(&self, module: &IRModule) -> Result<String, Self::Error>;

    /// Get the target language name
    ///
    /// Used for diagnostics and user-facing messages.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// assert_eq!(rust_gen.language(), "Rust");
    /// assert_eq!(typescript_gen.language(), "TypeScript");
    /// ```
    fn language(&self) -> &'static str;

    /// Get the file extension for generated code
    ///
    /// Used when writing generated code to files.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// assert_eq!(rust_gen.file_extension(), "rs");
    /// assert_eq!(typescript_gen.file_extension(), "ts");
    /// ```
    fn file_extension(&self) -> &str;

    /// Validate IR before generation (optional)
    ///
    /// Provides a way to check if the IR is compatible with this generator
    /// before attempting full code generation. Default implementation
    /// always returns `Ok(())`.
    ///
    /// # Arguments
    ///
    /// * `module` - The IR module to validate
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if IR is valid, `Err` otherwise.
    fn validate(&self, _module: &IRModule) -> Result<(), Self::Error> {
        Ok(())
    }

    /// Format generated code (optional)
    ///
    /// Apply language-specific formatting to the generated code.
    /// Default implementation returns the code unchanged.
    ///
    /// # Arguments
    ///
    /// * `code` - The generated code to format
    ///
    /// # Returns
    ///
    /// Returns formatted code on success.
    fn format(&self, code: String) -> Result<String, Self::Error> {
        Ok(code)
    }

    /// Get metadata about the generator (optional)
    ///
    /// Returns additional information about the generator's capabilities,
    /// version, etc. Default implementation returns empty metadata.
    fn metadata(&self) -> GeneratorMetadata {
        GeneratorMetadata::default()
    }
}

/// Metadata about a code generator
///
/// Contains additional information about generator capabilities and configuration.
#[derive(Debug, Clone, Default)]
pub struct GeneratorMetadata {
    /// Generator version
    pub version: Option<String>,
    /// Description of what this generator produces
    pub description: Option<String>,
    /// Minimum language version required (e.g., "1.70" for Rust)
    pub min_language_version: Option<String>,
    /// List of supported features
    pub features: Vec<String>,
    /// Additional custom metadata
    pub custom: std::collections::HashMap<String, String>,
}

impl GeneratorMetadata {
    /// Create new empty metadata
    pub fn new() -> Self {
        Self::default()
    }

    /// Set version
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Set description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set minimum language version
    pub fn with_min_language_version(mut self, version: impl Into<String>) -> Self {
        self.min_language_version = Some(version.into());
        self
    }

    /// Add a feature
    pub fn with_feature(mut self, feature: impl Into<String>) -> Self {
        self.features.push(feature.into());
        self
    }

    /// Add custom metadata
    pub fn with_custom(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.custom.insert(key.into(), value.into());
        self
    }
}

/// Extension trait for CodeGenerator to provide convenience methods
pub trait CodeGeneratorExt: CodeGenerator {
    /// Generate and format code in one step
    fn generate_formatted(&self, module: &IRModule) -> Result<String, Self::Error> {
        let code = self.generate(module)?;
        self.format(code)
    }

    /// Generate code with validation first
    fn generate_validated(&self, module: &IRModule) -> Result<String, Self::Error> {
        self.validate(module)?;
        self.generate(module)
    }

    /// Generate, validate, and format code
    fn generate_complete(&self, module: &IRModule) -> Result<String, Self::Error> {
        self.validate(module)?;
        let code = self.generate(module)?;
        self.format(code)
    }

    /// Generate code and convert to a specific error type
    fn generate_or<E>(&self, module: &IRModule) -> Result<String, E>
    where
        E: From<Self::Error>,
    {
        self.generate(module).map_err(E::from)
    }
}

// Blanket implementation for all CodeGenerator types
impl<G: CodeGenerator> CodeGeneratorExt for G {}

/// Helper error type for multi-generator operations
#[derive(Debug)]
pub struct MultiGeneratorError {
    pub generator_name: String,
    pub message: String,
}

impl std::fmt::Display for MultiGeneratorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Generator '{}' failed: {}", self.generator_name, self.message)
    }
}

impl StdError for MultiGeneratorError {}

/// Helper for chaining multiple generators
///
/// Allows generating code in multiple languages from the same IR.
/// This is useful for projects that need to generate code in multiple
/// target languages from a single schema.
///
/// # Examples
///
/// ```ignore
/// use unistructgen_core::{MultiGenerator, IRModule};
///
/// let multi = MultiGenerator::new()
///     .add("rust", rust_generator)
///     .add("typescript", ts_generator);
///
/// let results = multi.generate_all(&module)?;
/// for (name, code) in results {
///     println!("Generated {} code", name);
/// }
/// ```
pub struct MultiGenerator {
    generators: Vec<GeneratorEntry>,
}

struct GeneratorEntry {
    name: String,
    generator: Box<dyn GeneratorWrapper>,
}

trait GeneratorWrapper {
    fn generate_wrapped(&self, module: &IRModule) -> Result<String, MultiGeneratorError>;
    fn language(&self) -> &'static str;
    fn file_extension(&self) -> &str;
}

struct GenWrap<G>(G);

impl<G> GeneratorWrapper for GenWrap<G>
where
    G: CodeGenerator,
{
    fn generate_wrapped(&self, module: &IRModule) -> Result<String, MultiGeneratorError> {
        self.0.generate(module).map_err(|e| MultiGeneratorError {
            generator_name: self.0.language().to_string(),
            message: e.to_string(),
        })
    }

    fn language(&self) -> &'static str {
        self.0.language()
    }

    fn file_extension(&self) -> &str {
        self.0.file_extension()
    }
}

impl MultiGenerator {
    /// Create a new multi-generator
    pub fn new() -> Self {
        Self {
            generators: Vec::new(),
        }
    }

    /// Add a generator
    pub fn add<G>(mut self, name: impl Into<String>, generator: G) -> Self
    where
        G: CodeGenerator + 'static,
    {
        self.generators.push(GeneratorEntry {
            name: name.into(),
            generator: Box::new(GenWrap(generator)),
        });
        self
    }

    /// Generate code with all registered generators
    pub fn generate_all(
        &self,
        module: &IRModule,
    ) -> Result<Vec<(String, String)>, MultiGeneratorError> {
        let mut results = Vec::new();
        for entry in &self.generators {
            let code = entry.generator.generate_wrapped(module)?;
            results.push((entry.name.clone(), code));
        }
        Ok(results)
    }

    /// Get list of registered generator names
    pub fn generator_names(&self) -> Vec<&str> {
        self.generators.iter().map(|e| e.name.as_str()).collect()
    }
}

impl Default for MultiGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{IRModule, IRStruct, IRType};

    // Mock generator for testing
    struct MockGenerator;

    #[derive(Debug)]
    struct MockError;

    impl std::fmt::Display for MockError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Mock error")
        }
    }

    impl StdError for MockError {}

    impl CodeGenerator for MockGenerator {
        type Error = MockError;

        fn generate(&self, module: &IRModule) -> Result<String, Self::Error> {
            Ok(format!("struct {};", module.name))
        }

        fn language(&self) -> &'static str {
            "Mock"
        }

        fn file_extension(&self) -> &str {
            "mock"
        }
    }

    #[test]
    fn test_generator_trait() {
        let generator = MockGenerator;
        let mut module = IRModule::new("Test".to_string());
        let test_struct = IRStruct::new("Test".to_string());
        module.add_type(IRType::Struct(test_struct));

        let result = generator.generate(&module);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "struct Test;");
    }

    #[test]
    fn test_generator_metadata() {
        let generator = MockGenerator;
        assert_eq!(generator.language(), "Mock");
        assert_eq!(generator.file_extension(), "mock");
    }

    #[test]
    fn test_generator_metadata_builder() {
        let metadata = GeneratorMetadata::new()
            .with_version("1.0.0")
            .with_description("Test generator")
            .with_min_language_version("2021")
            .with_feature("generics")
            .with_custom("author", "test");

        assert_eq!(metadata.version, Some("1.0.0".to_string()));
        assert_eq!(metadata.description, Some("Test generator".to_string()));
        assert_eq!(metadata.min_language_version, Some("2021".to_string()));
        assert_eq!(metadata.features, vec!["generics"]);
        assert_eq!(metadata.custom.get("author"), Some(&"test".to_string()));
    }

    #[test]
    fn test_generator_ext() {
        let generator = MockGenerator;
        let mut module = IRModule::new("Test".to_string());
        let test_struct = IRStruct::new("Test".to_string());
        module.add_type(IRType::Struct(test_struct));

        let result = generator.generate_validated(&module);
        assert!(result.is_ok());

        let result = generator.generate_formatted(&module);
        assert!(result.is_ok());

        let result = generator.generate_complete(&module);
        assert!(result.is_ok());
    }

    #[test]
    fn test_multi_generator() {
        let multi = MultiGenerator::new()
            .add("mock1", MockGenerator)
            .add("mock2", MockGenerator);

        let mut module = IRModule::new("Test".to_string());
        let test_struct = IRStruct::new("Test".to_string());
        module.add_type(IRType::Struct(test_struct));

        let results = multi.generate_all(&module).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0, "mock1");
        assert_eq!(results[1].0, "mock2");
    }
}
