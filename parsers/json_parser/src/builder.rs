//! Builder pattern for JsonParser configuration
//!
//! Provides a fluent API for configuring JSON parser options.

use crate::{JsonParser, ParserOptions};

/// Builder for creating a JsonParser with fluent API
///
/// This builder provides a convenient way to configure parser options
/// using method chaining.
///
/// # Examples
///
/// ```
/// use unistructgen_json_parser::JsonParserBuilder;
///
/// let parser = JsonParserBuilder::new()
///     .struct_name("User")
///     .derive_serde(true)
///     .derive_default(false)
///     .make_optional(false)
///     .build();
/// ```
#[derive(Debug, Clone)]
pub struct JsonParserBuilder {
    options: ParserOptions,
}

impl JsonParserBuilder {
    /// Create a new builder with default options
    ///
    /// # Examples
    ///
    /// ```
    /// use unistructgen_json_parser::JsonParserBuilder;
    ///
    /// let builder = JsonParserBuilder::new();
    /// ```
    pub fn new() -> Self {
        Self {
            options: ParserOptions::default(),
        }
    }

    /// Set the name of the root struct to generate
    ///
    /// # Arguments
    ///
    /// * `name` - The struct name (will be used as-is, ensure it's PascalCase)
    ///
    /// # Examples
    ///
    /// ```
    /// use unistructgen_json_parser::JsonParserBuilder;
    ///
    /// let parser = JsonParserBuilder::new()
    ///     .struct_name("MyStruct")
    ///     .build();
    /// ```
    pub fn struct_name(mut self, name: impl Into<String>) -> Self {
        self.options.struct_name = name.into();
        self
    }

    /// Enable or disable serde derives (Serialize, Deserialize)
    ///
    /// Default: `true`
    ///
    /// # Examples
    ///
    /// ```
    /// use unistructgen_json_parser::JsonParserBuilder;
    ///
    /// let parser = JsonParserBuilder::new()
    ///     .derive_serde(true)
    ///     .build();
    /// ```
    pub fn derive_serde(mut self, enable: bool) -> Self {
        self.options.derive_serde = enable;
        self
    }

    /// Enable or disable Default derive
    ///
    /// Default: `false`
    ///
    /// # Examples
    ///
    /// ```
    /// use unistructgen_json_parser::JsonParserBuilder;
    ///
    /// let parser = JsonParserBuilder::new()
    ///     .derive_default(true)
    ///     .build();
    /// ```
    pub fn derive_default(mut self, enable: bool) -> Self {
        self.options.derive_default = enable;
        self
    }

    /// Make all fields optional (wrapped in Option<T>)
    ///
    /// This is useful when the JSON schema might have missing fields.
    ///
    /// Default: `false`
    ///
    /// # Examples
    ///
    /// ```
    /// use unistructgen_json_parser::JsonParserBuilder;
    ///
    /// let parser = JsonParserBuilder::new()
    ///     .make_optional(true)
    ///     .build();
    /// ```
    pub fn make_optional(mut self, enable: bool) -> Self {
        self.options.make_fields_optional = enable;
        self
    }

    /// Set multiple derives at once
    ///
    /// Convenience method for configuring common derive combinations.
    ///
    /// # Examples
    ///
    /// ```
    /// use unistructgen_json_parser::JsonParserBuilder;
    ///
    /// let parser = JsonParserBuilder::new()
    ///     .with_derives(true, true) // serde + default
    ///     .build();
    /// ```
    pub fn with_derives(mut self, serde: bool, default: bool) -> Self {
        self.options.derive_serde = serde;
        self.options.derive_default = default;
        self
    }

    /// Get the current options (useful for inspection)
    ///
    /// # Examples
    ///
    /// ```
    /// use unistructgen_json_parser::JsonParserBuilder;
    ///
    /// let builder = JsonParserBuilder::new().struct_name("Test");
    /// let options = builder.options();
    /// assert_eq!(options.struct_name, "Test");
    /// ```
    pub fn options(&self) -> &ParserOptions {
        &self.options
    }

    /// Build the JsonParser with the configured options
    ///
    /// # Examples
    ///
    /// ```
    /// use unistructgen_json_parser::JsonParserBuilder;
    ///
    /// let parser = JsonParserBuilder::new()
    ///     .struct_name("User")
    ///     .derive_serde(true)
    ///     .build();
    /// ```
    pub fn build(self) -> JsonParser {
        JsonParser::new(self.options)
    }
}

impl Default for JsonParserBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl JsonParser {
    /// Create a builder for configuring a JsonParser
    ///
    /// This is a convenience method that returns a new builder.
    ///
    /// # Examples
    ///
    /// ```
    /// use unistructgen_json_parser::JsonParser;
    ///
    /// let parser = JsonParser::builder()
    ///     .struct_name("Config")
    ///     .derive_default(true)
    ///     .build();
    /// ```
    pub fn builder() -> JsonParserBuilder {
        JsonParserBuilder::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_default() {
        let _parser = JsonParserBuilder::new().build();
        let builder = JsonParserBuilder::new();
        let options = builder.options();

        assert_eq!(options.struct_name, "Root");
        assert!(options.derive_serde);
        assert!(!options.derive_default);
        assert!(!options.make_fields_optional);
    }

    #[test]
    fn test_builder_custom() {
        let _parser = JsonParserBuilder::new()
            .struct_name("CustomStruct")
            .derive_serde(false)
            .derive_default(true)
            .make_optional(true)
            .build();

        // Parser created successfully
    }

    #[test]
    fn test_builder_fluent_api() {
        let _parser = JsonParser::builder()
            .struct_name("User")
            .with_derives(true, true)
            .build();

        // Parser created successfully
    }

    #[test]
    fn test_builder_options_inspection() {
        let builder = JsonParserBuilder::new()
            .struct_name("Test")
            .derive_serde(true);

        let options = builder.options();
        assert_eq!(options.struct_name, "Test");
        assert!(options.derive_serde);
    }
}
