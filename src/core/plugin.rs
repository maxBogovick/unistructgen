//! Plugin system for extending functionality
//!
//! Plugins allow you to hook into the parsing and generation process at specific points,
//! enabling custom processing without modifying the core library.

use crate::core::IRModule;
use std::error::Error as StdError;
use thiserror::Error;

/// Errors that can occur during plugin execution
#[derive(Error, Debug)]
pub enum PluginError {
    /// Plugin initialization failed
    #[error("Plugin '{plugin}' initialization failed: {message}")]
    Initialization {
        /// Name of the plugin
        plugin: String,
        /// Error message
        message: String,
    },

    /// Plugin execution failed
    #[error("Plugin '{plugin}' execution failed: {message}")]
    Execution {
        /// Name of the plugin
        plugin: String,
        /// Error message
        message: String,
        /// Optional underlying error
        #[source]
        source: Option<Box<dyn StdError + Send + Sync>>,
    },

    /// Plugin not found
    #[error("Plugin '{0}' not found in registry")]
    NotFound(String),

    /// Plugin already registered
    #[error("Plugin '{0}' is already registered")]
    AlreadyRegistered(String),
}

impl PluginError {
    /// Create an initialization error
    pub fn initialization(plugin: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Initialization {
            plugin: plugin.into(),
            message: message.into(),
        }
    }

    /// Create an execution error
    pub fn execution(plugin: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Execution {
            plugin: plugin.into(),
            message: message.into(),
            source: None,
        }
    }

    /// Add a source error
    pub fn with_source(mut self, error: Box<dyn StdError + Send + Sync>) -> Self {
        if let Self::Execution { source, .. } = &mut self {
            *source = Some(error);
        }
        self
    }
}

/// Trait for implementing plugins
///
/// Plugins can hook into various stages of the processing pipeline:
/// - **Initialization**: Setup when the plugin is loaded
/// - **Before parsing**: Modify input before it's parsed
/// - **After parsing**: Modify the IR after parsing
/// - **After generation**: Modify generated code
///
/// # Examples
///
/// ```
/// use unistructgen::core::{Plugin, PluginError, IRModule};
///
/// struct MyPlugin {
///     name: String,
/// }
///
/// impl Plugin for MyPlugin {
///     fn name(&self) -> &str {
///         &self.name
///     }
///
///     fn version(&self) -> &str {
///         "1.0.0"
///     }
///
///     fn initialize(&mut self) -> Result<(), PluginError> {
///         println!("Plugin {} initialized", self.name());
///         Ok(())
///     }
///
///     fn after_parse(&mut self, module: IRModule) -> Result<IRModule, PluginError> {
///         // Modify the IR module here
///         Ok(module)
///     }
/// }
/// ```
pub trait Plugin: Send + Sync {
    /// Name of the plugin
    fn name(&self) -> &str;

    /// Version of the plugin
    fn version(&self) -> &str;

    /// Optional description
    fn description(&self) -> Option<&str> {
        None
    }

    /// Called during plugin initialization
    ///
    /// Use this to set up any resources the plugin needs.
    fn initialize(&mut self) -> Result<(), PluginError>;

    /// Called before parsing
    ///
    /// This allows the plugin to modify the input before it's parsed.
    /// By default, returns the input unchanged.
    fn before_parse(&mut self, input: &str) -> Result<String, PluginError> {
        Ok(input.to_string())
    }

    /// Called after parsing
    ///
    /// This allows the plugin to modify the IR after parsing.
    /// By default, returns the module unchanged.
    fn after_parse(&mut self, module: IRModule) -> Result<IRModule, PluginError> {
        Ok(module)
    }

    /// Called after code generation
    ///
    /// This allows the plugin to modify the generated code.
    /// By default, returns the code unchanged.
    fn after_generate(&mut self, code: String) -> Result<String, PluginError> {
        Ok(code)
    }

    /// Called during plugin shutdown
    ///
    /// Use this to clean up any resources.
    fn shutdown(&mut self) -> Result<(), PluginError> {
        Ok(())
    }
}

/// Registry for managing plugins
///
/// The registry allows you to register, retrieve, and execute plugins.
///
/// # Examples
///
/// ```
/// use unistructgen::core::PluginRegistry;
///
/// let mut registry = PluginRegistry::new();
///
/// // Register plugins
/// // registry.register(Box::new(my_plugin)).unwrap();
///
/// // Get a plugin
/// // let plugin = registry.get("my_plugin");
/// ```
pub struct PluginRegistry {
    plugins: Vec<Box<dyn Plugin>>,
}

impl PluginRegistry {
    /// Create a new plugin registry
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    /// Register a plugin
    ///
    /// # Errors
    ///
    /// Returns an error if a plugin with the same name is already registered.
    ///
    /// # Examples
    ///
    /// ```
    /// use unistructgen::core::PluginRegistry;
    ///
    /// let mut registry = PluginRegistry::new();
    /// // registry.register(Box::new(my_plugin)).unwrap();
    /// ```
    pub fn register(&mut self, mut plugin: Box<dyn Plugin>) -> Result<(), PluginError> {
        let name = plugin.name().to_string();

        // Check if already registered
        if self.plugins.iter().any(|p| p.name() == name) {
            return Err(PluginError::AlreadyRegistered(name));
        }

        // Initialize the plugin
        plugin
            .initialize()
            .map_err(|e| PluginError::initialization(&name, format!("{}", e)))?;

        self.plugins.push(plugin);
        Ok(())
    }

    /// Get a reference to a plugin by name
    ///
    /// # Examples
    ///
    /// ```
    /// use unistructgen::core::PluginRegistry;
    ///
    /// let registry = PluginRegistry::new();
    /// let plugin = registry.get("my_plugin");
    /// assert!(plugin.is_none());
    /// ```
    pub fn get(&self, name: &str) -> Option<&dyn Plugin> {
        self.plugins
            .iter()
            .map(|p| p.as_ref())
            .find(|p| p.name() == name)
    }

    /// Get a mutable reference to a plugin by name
    pub fn get_mut(&mut self, name: &str) -> Option<&mut dyn Plugin> {
        for plugin in &mut self.plugins {
            if plugin.name() == name {
                return Some(plugin.as_mut());
            }
        }
        None
    }

    /// Get all registered plugin names
    ///
    /// # Examples
    ///
    /// ```
    /// use unistructgen::core::PluginRegistry;
    ///
    /// let registry = PluginRegistry::new();
    /// let names = registry.plugin_names();
    /// assert_eq!(names.len(), 0);
    /// ```
    pub fn plugin_names(&self) -> Vec<&str> {
        self.plugins.iter().map(|p| p.name()).collect()
    }

    /// Get the number of registered plugins
    pub fn count(&self) -> usize {
        self.plugins.len()
    }

    /// Check if a plugin is registered
    pub fn has_plugin(&self, name: &str) -> bool {
        self.plugins.iter().any(|p| p.name() == name)
    }

    /// Remove a plugin by name
    ///
    /// # Examples
    ///
    /// ```
    /// use unistructgen::core::PluginRegistry;
    ///
    /// let mut registry = PluginRegistry::new();
    /// let removed = registry.remove("my_plugin");
    /// assert!(removed.is_none());
    /// ```
    pub fn remove(&mut self, name: &str) -> Option<Box<dyn Plugin>> {
        let pos = self.plugins.iter().position(|p| p.name() == name)?;
        Some(self.plugins.remove(pos))
    }

    /// Execute `before_parse` on all plugins
    ///
    /// Plugins are executed in the order they were registered.
    pub fn before_parse(&mut self, mut input: String) -> Result<String, PluginError> {
        for plugin in &mut self.plugins {
            input = plugin.before_parse(&input)?;
        }
        Ok(input)
    }

    /// Execute `after_parse` on all plugins
    ///
    /// Plugins are executed in the order they were registered.
    pub fn after_parse(&mut self, mut module: IRModule) -> Result<IRModule, PluginError> {
        for plugin in &mut self.plugins {
            module = plugin.after_parse(module)?;
        }
        Ok(module)
    }

    /// Execute `after_generate` on all plugins
    ///
    /// Plugins are executed in the order they were registered.
    pub fn after_generate(&mut self, mut code: String) -> Result<String, PluginError> {
        for plugin in &mut self.plugins {
            code = plugin.after_generate(code)?;
        }
        Ok(code)
    }

    /// Shutdown all plugins
    pub fn shutdown(&mut self) -> Result<(), PluginError> {
        for plugin in &mut self.plugins {
            plugin.shutdown()?;
        }
        Ok(())
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for PluginRegistry {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}

/// Example plugin that adds a header comment to generated code
///
/// # Examples
///
/// ```
/// use unistructgen::core::plugin::HeaderPlugin;
/// use unistructgen::core::Plugin;
///
/// let mut plugin = HeaderPlugin::new("Generated by MyTool");
/// plugin.initialize().unwrap();
/// ```
pub struct HeaderPlugin {
    header: String,
}

impl HeaderPlugin {
    /// Create a new header plugin
    pub fn new(header: impl Into<String>) -> Self {
        Self {
            header: header.into(),
        }
    }
}

impl Plugin for HeaderPlugin {
    fn name(&self) -> &str {
        "HeaderPlugin"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn description(&self) -> Option<&str> {
        Some("Adds a header comment to generated code")
    }

    fn initialize(&mut self) -> Result<(), PluginError> {
        Ok(())
    }

    fn after_generate(&mut self, code: String) -> Result<String, PluginError> {
        Ok(format!("// {}\n\n{}", self.header, code))
    }
}

/// Example plugin that logs processing stages
///
/// Useful for debugging and monitoring the pipeline.
pub struct LoggingPlugin {
    verbose: bool,
}

impl LoggingPlugin {
    /// Create a new logging plugin
    pub fn new(verbose: bool) -> Self {
        Self { verbose }
    }

    fn log(&self, stage: &str, message: &str) {
        if self.verbose {
            println!("[LoggingPlugin] {}: {}", stage, message);
        }
    }
}

impl Plugin for LoggingPlugin {
    fn name(&self) -> &str {
        "LoggingPlugin"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn description(&self) -> Option<&str> {
        Some("Logs processing stages for debugging")
    }

    fn initialize(&mut self) -> Result<(), PluginError> {
        self.log("init", "Plugin initialized");
        Ok(())
    }

    fn before_parse(&mut self, input: &str) -> Result<String, PluginError> {
        self.log("before_parse", &format!("Input length: {}", input.len()));
        Ok(input.to_string())
    }

    fn after_parse(&mut self, module: IRModule) -> Result<IRModule, PluginError> {
        self.log(
            "after_parse",
            &format!("Generated {} types", module.types.len()),
        );
        Ok(module)
    }

    fn after_generate(&mut self, code: String) -> Result<String, PluginError> {
        self.log(
            "after_generate",
            &format!("Generated code: {} bytes", code.len()),
        );
        Ok(code)
    }

    fn shutdown(&mut self) -> Result<(), PluginError> {
        self.log("shutdown", "Plugin shutting down");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestPlugin {
        name: String,
        initialized: bool,
    }

    impl TestPlugin {
        fn new(name: impl Into<String>) -> Self {
            Self {
                name: name.into(),
                initialized: false,
            }
        }
    }

    impl Plugin for TestPlugin {
        fn name(&self) -> &str {
            &self.name
        }

        fn version(&self) -> &str {
            "1.0.0"
        }

        fn initialize(&mut self) -> Result<(), PluginError> {
            self.initialized = true;
            Ok(())
        }

        fn after_generate(&mut self, code: String) -> Result<String, PluginError> {
            Ok(format!("// Modified by {}\n{}", self.name, code))
        }
    }

    #[test]
    fn test_plugin_registration() {
        let mut registry = PluginRegistry::new();
        let plugin = Box::new(TestPlugin::new("test1"));

        registry.register(plugin).unwrap();
        assert_eq!(registry.count(), 1);
        assert!(registry.has_plugin("test1"));
    }

    #[test]
    fn test_duplicate_registration() {
        let mut registry = PluginRegistry::new();
        registry
            .register(Box::new(TestPlugin::new("test1")))
            .unwrap();

        let result = registry.register(Box::new(TestPlugin::new("test1")));
        assert!(result.is_err());
    }

    #[test]
    fn test_plugin_retrieval() {
        let mut registry = PluginRegistry::new();
        registry
            .register(Box::new(TestPlugin::new("test1")))
            .unwrap();

        let plugin = registry.get("test1");
        assert!(plugin.is_some());
        assert_eq!(plugin.unwrap().name(), "test1");

        let missing = registry.get("missing");
        assert!(missing.is_none());
    }

    #[test]
    fn test_plugin_removal() {
        let mut registry = PluginRegistry::new();
        registry
            .register(Box::new(TestPlugin::new("test1")))
            .unwrap();

        assert_eq!(registry.count(), 1);

        let removed = registry.remove("test1");
        assert!(removed.is_some());
        assert_eq!(registry.count(), 0);
    }

    #[test]
    fn test_after_generate() {
        let mut registry = PluginRegistry::new();
        registry
            .register(Box::new(TestPlugin::new("test1")))
            .unwrap();

        let code = "original code".to_string();
        let result = registry.after_generate(code).unwrap();

        assert!(result.contains("Modified by test1"));
        assert!(result.contains("original code"));
    }

    #[test]
    fn test_header_plugin() {
        let mut plugin = HeaderPlugin::new("Generated by Test");
        plugin.initialize().unwrap();

        let code = "fn main() {}".to_string();
        let result = plugin.after_generate(code).unwrap();

        assert!(result.starts_with("// Generated by Test"));
    }

    #[test]
    fn test_logging_plugin() {
        let mut plugin = LoggingPlugin::new(false);
        plugin.initialize().unwrap();

        let input = "test input";
        let result = plugin.before_parse(input).unwrap();
        assert_eq!(result, input);
    }
}
