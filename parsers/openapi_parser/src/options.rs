//! Parser configuration options

use std::collections::HashSet;

/// Options for configuring the OpenAPI parser
#[derive(Debug, Clone)]
pub struct OpenApiParserOptions {
    /// Generate API client traits
    pub generate_client: bool,

    /// Generate validation derives and attributes
    pub generate_validation: bool,

    /// Add serde derives to generated structs
    pub derive_serde: bool,

    /// Add Default derive to generated structs
    pub derive_default: bool,

    /// Make all fields optional (wrap in Option<T>)
    pub make_fields_optional: bool,

    /// Maximum depth for nested schemas (prevents infinite recursion)
    pub max_depth: usize,

    /// Tag names to filter (only include these tags)
    pub include_tags: Option<HashSet<String>>,

    /// Tag names to exclude
    pub exclude_tags: Option<HashSet<String>>,

    /// Generate builder pattern for request types
    pub generate_builders: bool,

    /// Add documentation comments from OpenAPI descriptions
    pub generate_docs: bool,

    /// Prefix for generated type names
    pub type_prefix: Option<String>,

    /// Suffix for generated type names
    pub type_suffix: Option<String>,

    /// Module name for generated code
    pub module_name: String,
}

impl Default for OpenApiParserOptions {
    fn default() -> Self {
        Self {
            generate_client: true,
            generate_validation: true,
            derive_serde: true,
            derive_default: false,
            make_fields_optional: false,
            max_depth: 10,
            include_tags: None,
            exclude_tags: None,
            generate_builders: false,
            generate_docs: true,
            type_prefix: None,
            type_suffix: None,
            module_name: "api".to_string(),
        }
    }
}

impl OpenApiParserOptions {
    /// Create a new builder for parser options
    pub fn builder() -> OpenApiParserOptionsBuilder {
        OpenApiParserOptionsBuilder::default()
    }

    /// Check if a tag should be included
    pub fn should_include_tag(&self, tag: &str) -> bool {
        // If include_tags is specified, only include those
        if let Some(ref include) = self.include_tags {
            if !include.contains(tag) {
                return false;
            }
        }

        // Check exclude list
        if let Some(ref exclude) = self.exclude_tags {
            if exclude.contains(tag) {
                return false;
            }
        }

        true
    }

    /// Apply prefix and suffix to type name
    pub fn format_type_name(&self, name: &str) -> String {
        let mut result = String::new();

        if let Some(ref prefix) = self.type_prefix {
            result.push_str(prefix);
        }

        result.push_str(name);

        if let Some(ref suffix) = self.type_suffix {
            result.push_str(suffix);
        }

        result
    }
}

/// Builder for OpenApiParserOptions
#[derive(Debug, Default)]
pub struct OpenApiParserOptionsBuilder {
    generate_client: Option<bool>,
    generate_validation: Option<bool>,
    derive_serde: Option<bool>,
    derive_default: Option<bool>,
    make_fields_optional: Option<bool>,
    max_depth: Option<usize>,
    include_tags: Option<HashSet<String>>,
    exclude_tags: Option<HashSet<String>>,
    generate_builders: Option<bool>,
    generate_docs: Option<bool>,
    type_prefix: Option<String>,
    type_suffix: Option<String>,
    module_name: Option<String>,
}

impl OpenApiParserOptionsBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set whether to generate API client traits
    pub fn generate_client(mut self, value: bool) -> Self {
        self.generate_client = Some(value);
        self
    }

    /// Set whether to generate validation
    pub fn generate_validation(mut self, value: bool) -> Self {
        self.generate_validation = Some(value);
        self
    }

    /// Set whether to derive serde traits
    pub fn derive_serde(mut self, value: bool) -> Self {
        self.derive_serde = Some(value);
        self
    }

    /// Set whether to derive Default trait
    pub fn derive_default(mut self, value: bool) -> Self {
        self.derive_default = Some(value);
        self
    }

    /// Set whether to make all fields optional
    pub fn make_fields_optional(mut self, value: bool) -> Self {
        self.make_fields_optional = Some(value);
        self
    }

    /// Set maximum depth for nested schemas
    pub fn max_depth(mut self, value: usize) -> Self {
        self.max_depth = Some(value);
        self
    }

    /// Set tags to include
    pub fn include_tags(mut self, tags: HashSet<String>) -> Self {
        self.include_tags = Some(tags);
        self
    }

    /// Set tags to exclude
    pub fn exclude_tags(mut self, tags: HashSet<String>) -> Self {
        self.exclude_tags = Some(tags);
        self
    }

    /// Set whether to generate builder patterns
    pub fn generate_builders(mut self, value: bool) -> Self {
        self.generate_builders = Some(value);
        self
    }

    /// Set whether to generate documentation
    pub fn generate_docs(mut self, value: bool) -> Self {
        self.generate_docs = Some(value);
        self
    }

    /// Set type name prefix
    pub fn type_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.type_prefix = Some(prefix.into());
        self
    }

    /// Set type name suffix
    pub fn type_suffix(mut self, suffix: impl Into<String>) -> Self {
        self.type_suffix = Some(suffix.into());
        self
    }

    /// Set module name
    pub fn module_name(mut self, name: impl Into<String>) -> Self {
        self.module_name = Some(name.into());
        self
    }

    /// Build the options
    pub fn build(self) -> OpenApiParserOptions {
        let defaults = OpenApiParserOptions::default();

        OpenApiParserOptions {
            generate_client: self.generate_client.unwrap_or(defaults.generate_client),
            generate_validation: self
                .generate_validation
                .unwrap_or(defaults.generate_validation),
            derive_serde: self.derive_serde.unwrap_or(defaults.derive_serde),
            derive_default: self.derive_default.unwrap_or(defaults.derive_default),
            make_fields_optional: self
                .make_fields_optional
                .unwrap_or(defaults.make_fields_optional),
            max_depth: self.max_depth.unwrap_or(defaults.max_depth),
            include_tags: self.include_tags.or(defaults.include_tags),
            exclude_tags: self.exclude_tags.or(defaults.exclude_tags),
            generate_builders: self.generate_builders.unwrap_or(defaults.generate_builders),
            generate_docs: self.generate_docs.unwrap_or(defaults.generate_docs),
            type_prefix: self.type_prefix.or(defaults.type_prefix),
            type_suffix: self.type_suffix.or(defaults.type_suffix),
            module_name: self.module_name.unwrap_or(defaults.module_name),
        }
    }
}
