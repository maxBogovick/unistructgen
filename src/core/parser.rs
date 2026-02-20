//! Parser trait and related types
//!
//! This module defines the core trait that all parsers must implement.
//! It provides a unified interface for parsing different input formats
//! (JSON, Markdown, SQL, etc.) into the Intermediate Representation (IR).

use crate::core::IRModule;
use std::error::Error as StdError;

/// Result type alias for parser operations
pub type ParserResult<T> = std::result::Result<T, Box<dyn StdError + Send + Sync>>;

/// Core trait for all parsers
///
/// Implement this trait to add support for new input formats.
/// The parser is responsible for converting input text into an IR module
/// that can be processed by code generators.
///
/// # Type Parameters
///
/// * `Error` - The error type produced by this parser. Must implement
///   `std::error::Error + Send + Sync + 'static` for composability.
///
/// # Examples
///
/// ```ignore
/// use unistructgen::core::{Parser, IRModule};
///
/// struct MyParser {
///     options: MyOptions,
/// }
///
/// impl Parser for MyParser {
///     type Error = MyParserError;
///
///     fn parse(&mut self, input: &str) -> Result<IRModule, Self::Error> {
///         // Parse input and return IR
///         todo!()
///     }
///
///     fn name(&self) -> &'static str {
///         "MyFormat"
///     }
///
///     fn extensions(&self) -> &[&'static str] {
///         &["my", "myformat"]
///     }
/// }
/// ```
pub trait Parser {
    /// The error type this parser produces
    type Error: StdError + Send + Sync + 'static;

    /// Parse input text and return an IR module
    ///
    /// # Arguments
    ///
    /// * `input` - The input text to parse
    ///
    /// # Returns
    ///
    /// Returns `Ok(IRModule)` on success, containing the parsed types.
    ///
    /// # Errors
    ///
    /// Returns `Self::Error` if parsing fails. The error should contain
    /// detailed information about what went wrong and where.
    fn parse(&mut self, input: &str) -> Result<IRModule, Self::Error>;

    /// Get the parser's human-readable name
    ///
    /// Used for diagnostics and error messages.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// assert_eq!(json_parser.name(), "JSON");
    /// ```
    fn name(&self) -> &'static str;

    /// Get the file extensions this parser supports
    ///
    /// Used for automatic format detection based on file extension.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// assert_eq!(json_parser.extensions(), &["json"]);
    /// assert_eq!(markdown_parser.extensions(), &["md", "markdown"]);
    /// ```
    fn extensions(&self) -> &[&'static str];

    /// Validate input without full parsing (optional)
    ///
    /// Provides a quick way to check if input is valid without
    /// performing full parsing. Default implementation always returns `Ok(())`.
    ///
    /// # Arguments
    ///
    /// * `input` - The input text to validate
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if input appears valid, `Err` otherwise.
    fn validate(&self, _input: &str) -> Result<(), Self::Error> {
        Ok(())
    }

    /// Get metadata about the parser (optional)
    ///
    /// Returns additional information about the parser's capabilities,
    /// version, etc. Default implementation returns empty metadata.
    fn metadata(&self) -> ParserMetadata {
        ParserMetadata::default()
    }
}

/// Metadata about a parser
///
/// Contains additional information about parser capabilities and configuration.
#[derive(Debug, Clone, Default)]
pub struct ParserMetadata {
    /// Parser version
    pub version: Option<String>,
    /// Description of what this parser does
    pub description: Option<String>,
    /// List of supported features
    pub features: Vec<String>,
    /// Additional custom metadata
    pub custom: std::collections::HashMap<String, String>,
}

impl ParserMetadata {
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

/// Helper trait for converting errors to parser results
///
/// Allows using `?` operator with different error types.
pub trait IntoParserError {
    /// Convert into a boxed error
    fn into_parser_error(self) -> Box<dyn StdError + Send + Sync>;
}

impl<E> IntoParserError for E
where
    E: StdError + Send + Sync + 'static,
{
    fn into_parser_error(self) -> Box<dyn StdError + Send + Sync> {
        Box::new(self)
    }
}

/// Extension trait for Parser to provide convenience methods
pub trait ParserExt: Parser {
    /// Parse input and convert to a specific error type
    fn parse_or<E>(&mut self, input: &str) -> Result<IRModule, E>
    where
        E: From<Self::Error>,
    {
        self.parse(input).map_err(E::from)
    }

    /// Parse input with validation first
    fn parse_validated(&mut self, input: &str) -> Result<IRModule, Self::Error> {
        self.validate(input)?;
        self.parse(input)
    }
}

// Blanket implementation for all Parser types
impl<P: Parser> ParserExt for P {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{IRModule, IRStruct, IRType};

    // Mock parser for testing
    struct MockParser;

    #[derive(Debug)]
    struct MockError;

    impl std::fmt::Display for MockError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Mock error")
        }
    }

    impl StdError for MockError {}

    impl Parser for MockParser {
        type Error = MockError;

        fn parse(&mut self, _input: &str) -> Result<IRModule, Self::Error> {
            let mut module = IRModule::new("Test".to_string());
            let test_struct = IRStruct::new("Test".to_string());
            module.add_type(IRType::Struct(test_struct));
            Ok(module)
        }

        fn name(&self) -> &'static str {
            "Mock"
        }

        fn extensions(&self) -> &[&'static str] {
            &["mock"]
        }
    }

    #[test]
    fn test_parser_trait() {
        let mut parser = MockParser;
        let result = parser.parse("test input");
        assert!(result.is_ok());

        let module = result.unwrap();
        assert_eq!(module.name, "Test");
        assert_eq!(module.types.len(), 1);
    }

    #[test]
    fn test_parser_metadata() {
        let parser = MockParser;
        assert_eq!(parser.name(), "Mock");
        assert_eq!(parser.extensions(), &["mock"]);
    }

    #[test]
    fn test_parser_metadata_builder() {
        let metadata = ParserMetadata::new()
            .with_version("1.0.0")
            .with_description("Test parser")
            .with_feature("nested-objects")
            .with_custom("author", "test");

        assert_eq!(metadata.version, Some("1.0.0".to_string()));
        assert_eq!(metadata.description, Some("Test parser".to_string()));
        assert_eq!(metadata.features, vec!["nested-objects"]);
        assert_eq!(metadata.custom.get("author"), Some(&"test".to_string()));
    }

    #[test]
    fn test_parser_ext() {
        let mut parser = MockParser;
        let result = parser.parse_validated("test");
        assert!(result.is_ok());
    }
}
