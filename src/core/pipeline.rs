//! Pipeline architecture for processing data through multiple stages
//!
//! The pipeline allows you to chain together parsing, transformation, and code generation
//! into a single, reusable workflow.

use crate::core::{CodeGenerator, IRTransformer, Parser, TransformError};
use std::error::Error as StdError;
use thiserror::Error;

/// Errors that can occur during pipeline execution
#[derive(Error, Debug)]
pub enum PipelineError {
    /// Parser error
    #[error("Parse error: {0}")]
    Parse(#[source] Box<dyn StdError + Send + Sync>),

    /// Transformer error
    #[error("Transform error in '{transformer}': {source}")]
    Transform {
        /// The transformer that failed
        transformer: String,
        /// The underlying error
        #[source]
        source: TransformError,
    },

    /// Code generation error
    #[error("Generation error: {0}")]
    Generate(#[source] Box<dyn StdError + Send + Sync>),

    /// Plugin error
    #[error("Plugin error in '{plugin}': {message}")]
    Plugin {
        /// The plugin that failed
        plugin: String,
        /// Error message
        message: String,
    },
}

/// A complete processing pipeline from input to generated code
///
/// The pipeline chains together:
/// 1. **Parsing**: Convert input (JSON, etc.) to IR
/// 2. **Transformation**: Apply zero or more IR transformations
/// 3. **Generation**: Convert IR to target code (Rust, etc.)
///
/// # Examples
///
/// ```ignore
/// use unistructgen::core::{Pipeline, IRTransformer};
/// use unistructgen::parsers::json::JsonParser;
/// use unistructgen::codegen::RustRenderer;
///
/// let parser = JsonParser::new(Default::default());
/// let generator = RustRenderer::new(Default::default());
///
/// let mut pipeline = Pipeline::new(parser, generator);
///
/// // Add transformations
/// // pipeline = pipeline.add_transformer(Box::new(MyTransformer));
///
/// // Execute the pipeline
/// let json = r#"{"name": "John", "age": 30}"#;
/// let rust_code = pipeline.execute(json).unwrap();
/// ```
pub struct Pipeline<P, G>
where
    P: Parser,
    G: CodeGenerator,
{
    /// The parser to use
    parser: P,
    /// The code generator to use
    generator: G,
    /// IR transformers to apply (in order)
    transformers: Vec<Box<dyn IRTransformer>>,
}

impl<P, G> Pipeline<P, G>
where
    P: Parser,
    G: CodeGenerator,
{
    /// Create a new pipeline with the given parser and generator
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use unistructgen::core::Pipeline;
    /// use unistructgen::parsers::json::JsonParser;
    /// use unistructgen::codegen::RustRenderer;
    ///
    /// let parser = JsonParser::new(Default::default());
    /// let generator = RustRenderer::new(Default::default());
    /// let pipeline = Pipeline::new(parser, generator);
    /// ```
    pub fn new(parser: P, generator: G) -> Self {
        Self {
            parser,
            generator,
            transformers: Vec::new(),
        }
    }

    /// Add a transformer to the pipeline
    ///
    /// Transformers are applied in the order they are added.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use unistructgen::core::{Pipeline, transformer::FieldOptionalizer};
    /// use unistructgen::parsers::json::JsonParser;
    /// use unistructgen::codegen::RustRenderer;
    ///
    /// let parser = JsonParser::new(Default::default());
    /// let generator = RustRenderer::new(Default::default());
    ///
    /// let pipeline = Pipeline::new(parser, generator)
    ///     .add_transformer(Box::new(FieldOptionalizer::new()));
    /// ```
    pub fn add_transformer(mut self, transformer: Box<dyn IRTransformer>) -> Self {
        self.transformers.push(transformer);
        self
    }

    /// Add multiple transformers at once
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use unistructgen::core::{Pipeline, transformer::{FieldOptionalizer, DocCommentAdder}};
    /// use unistructgen::parsers::json::JsonParser;
    /// use unistructgen::codegen::RustRenderer;
    ///
    /// let parser = JsonParser::new(Default::default());
    /// let generator = RustRenderer::new(Default::default());
    ///
    /// let transformers: Vec<Box<dyn crate::core::IRTransformer>> = vec![
    ///     Box::new(FieldOptionalizer::new()),
    ///     Box::new(DocCommentAdder::new()),
    /// ];
    ///
    /// let pipeline = Pipeline::new(parser, generator)
    ///     .add_transformers(transformers);
    /// ```
    pub fn add_transformers(mut self, transformers: Vec<Box<dyn IRTransformer>>) -> Self {
        self.transformers.extend(transformers);
        self
    }

    /// Execute the pipeline on the given input
    ///
    /// This method:
    /// 1. Parses the input to IR
    /// 2. Applies all transformers in order
    /// 3. Generates code from the final IR
    ///
    /// # Errors
    ///
    /// Returns an error if parsing, transformation, or generation fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use unistructgen::core::Pipeline;
    /// use unistructgen::parsers::json::JsonParser;
    /// use unistructgen::codegen::RustRenderer;
    ///
    /// let parser = JsonParser::new(Default::default());
    /// let generator = RustRenderer::new(Default::default());
    /// let mut pipeline = Pipeline::new(parser, generator);
    ///
    /// let json = r#"{"name": "John"}"#;
    /// let code = pipeline.execute(json).unwrap();
    /// ```
    pub fn execute(&mut self, input: &str) -> Result<String, PipelineError> {
        // 1. Parse
        let mut ir = self
            .parser
            .parse(input)
            .map_err(|e| PipelineError::Parse(Box::new(e)))?;

        // 2. Transform
        for transformer in &self.transformers {
            ir = transformer.transform(ir).map_err(|e| PipelineError::Transform {
                transformer: transformer.name().to_string(),
                source: e,
            })?;
        }

        // 3. Generate
        let code = self
            .generator
            .generate(&ir)
            .map_err(|e| PipelineError::Generate(Box::new(e)))?;

        Ok(code)
    }

    /// Get a reference to the parser
    pub fn parser(&self) -> &P {
        &self.parser
    }

    /// Get a mutable reference to the parser
    pub fn parser_mut(&mut self) -> &mut P {
        &mut self.parser
    }

    /// Get a reference to the generator
    pub fn generator(&self) -> &G {
        &self.generator
    }

    /// Get a mutable reference to the generator
    pub fn generator_mut(&mut self) -> &mut G {
        &mut self.generator
    }

    /// Get the number of transformers in the pipeline
    pub fn transformer_count(&self) -> usize {
        self.transformers.len()
    }

    /// Get the names of all transformers in the pipeline
    pub fn transformer_names(&self) -> Vec<&str> {
        self.transformers.iter().map(|t| t.name()).collect()
    }
}

/// Builder for constructing pipelines with a fluent API
///
/// # Examples
///
/// ```ignore
/// use unistructgen::core::PipelineBuilder;
/// use unistructgen::parsers::json::JsonParser;
/// use unistructgen::codegen::RustRenderer;
/// use unistructgen::core::transformer::FieldOptionalizer;
///
/// let pipeline = PipelineBuilder::new()
///     .parser(JsonParser::new(Default::default()))
///     .generator(RustRenderer::new(Default::default()))
///     .transformer(Box::new(FieldOptionalizer::new()))
///     .build();
/// ```
pub struct PipelineBuilder<P, G>
where
    P: Parser,
    G: CodeGenerator,
{
    parser: Option<P>,
    generator: Option<G>,
    transformers: Vec<Box<dyn IRTransformer>>,
}

impl<P, G> PipelineBuilder<P, G>
where
    P: Parser,
    G: CodeGenerator,
{
    /// Create a new pipeline builder
    pub fn new() -> Self {
        Self {
            parser: None,
            generator: None,
            transformers: Vec::new(),
        }
    }

    /// Set the parser
    pub fn parser(mut self, parser: P) -> Self {
        self.parser = Some(parser);
        self
    }

    /// Set the generator
    pub fn generator(mut self, generator: G) -> Self {
        self.generator = Some(generator);
        self
    }

    /// Add a transformer
    pub fn transformer(mut self, transformer: Box<dyn IRTransformer>) -> Self {
        self.transformers.push(transformer);
        self
    }

    /// Add multiple transformers
    pub fn transformers(mut self, transformers: Vec<Box<dyn IRTransformer>>) -> Self {
        self.transformers.extend(transformers);
        self
    }

    /// Build the pipeline
    ///
    /// # Panics
    ///
    /// Panics if parser or generator is not set.
    pub fn build(self) -> Pipeline<P, G> {
        Pipeline {
            parser: self.parser.expect("Parser must be set"),
            generator: self.generator.expect("Generator must be set"),
            transformers: self.transformers,
        }
    }
}

impl<P, G> Default for PipelineBuilder<P, G>
where
    P: Parser,
    G: CodeGenerator,
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{
        transformer::FieldOptionalizer, IRModule, IRField,
        IRStruct, IRType, IRTypeRef, PrimitiveKind,
    };

    // Mock parser for testing
    struct MockParser;

    impl Parser for MockParser {
        type Error = std::io::Error;

        fn parse(&mut self, _input: &str) -> Result<IRModule, Self::Error> {
            let mut module = IRModule::new("Test".to_string());
            let mut s = IRStruct::new("User".to_string());
            let id_field = IRField::new("id".to_string(), IRTypeRef::Primitive(PrimitiveKind::I64));
            let name_field = IRField::new("name".to_string(), IRTypeRef::Primitive(PrimitiveKind::String));
            s.add_field(id_field);
            s.add_field(name_field);
            module.add_type(IRType::Struct(s));
            Ok(module)
        }

        fn name(&self) -> &'static str {
            "MockParser"
        }

        fn extensions(&self) -> &[&'static str] {
            &["mock"]
        }
    }

    // Mock generator for testing
    struct MockGenerator;

    impl CodeGenerator for MockGenerator {
        type Error = std::io::Error;

        fn generate(&self, module: &IRModule) -> Result<String, Self::Error> {
            Ok(format!("Generated code for {} types", module.types.len()))
        }

        fn language(&self) -> &'static str {
            "Mock"
        }

        fn file_extension(&self) -> &'static str {
            "mock"
        }
    }

    #[test]
    fn test_pipeline_basic() {
        let parser = MockParser;
        let generator = MockGenerator;
        let mut pipeline = Pipeline::new(parser, generator);

        let result = pipeline.execute("input").unwrap();
        assert!(result.contains("Generated code"));
    }

    #[test]
    fn test_pipeline_with_transformer() {
        let parser = MockParser;
        let generator = MockGenerator;
        let mut pipeline = Pipeline::new(parser, generator)
            .add_transformer(Box::new(FieldOptionalizer::new()));

        let result = pipeline.execute("input").unwrap();
        assert!(result.contains("Generated code"));
        assert_eq!(pipeline.transformer_count(), 1);
    }

    #[test]
    fn test_pipeline_builder() {
        let pipeline = PipelineBuilder::new()
            .parser(MockParser)
            .generator(MockGenerator)
            .transformer(Box::new(FieldOptionalizer::new()))
            .build();

        assert_eq!(pipeline.transformer_count(), 1);
        assert_eq!(pipeline.transformer_names(), vec!["FieldOptionalizer"]);
    }

    #[test]
    fn test_pipeline_accessors() {
        let parser = MockParser;
        let generator = MockGenerator;
        let pipeline = Pipeline::new(parser, generator);

        assert_eq!(pipeline.parser().name(), "MockParser");
        assert_eq!(pipeline.generator().language(), "Mock");
    }
}
