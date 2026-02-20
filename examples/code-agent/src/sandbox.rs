use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use anyhow::{Context, Result};
use colored::*;

/// Represents an ephemeral Rust project environment.
pub struct RustSandbox {
    _tmp_dir: TempDir, // Kept to prevent deletion until drop
    project_path: PathBuf,
}

impl RustSandbox {
    /// Create a new sandbox environment.
    pub fn new() -> Result<Self> {
        let tmp_dir = tempfile::Builder::new()
            .prefix("unistructgen::agent_")
            .tempdir()?;
        
        let project_path = tmp_dir.path().to_path_buf();
        
        println!("{} {}", "📦 Created sandbox at:".dimmed(), project_path.display());

        let sandbox = Self {
            _tmp_dir: tmp_dir,
            project_path,
        };

        sandbox.init_cargo_project()?;

        Ok(sandbox)
    }

    /// Initialize a minimal Cargo library project.
    fn init_cargo_project(&self) -> Result<()> {
        // We write Cargo.toml manually to control dependencies
        let cargo_toml = r#"
[package]
name = "generated_code"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "1.0"
anyhow = "1.0"
chrono = "0.4"
regex = "1.10"
"#;
        fs::write(self.project_path.join("Cargo.toml"), cargo_toml)
            .context("Failed to write Cargo.toml")?;

        // Ensure src directory exists
        fs::create_dir_all(self.project_path.join("src"))
            .context("Failed to create src directory")?;

        // Write a dummy lib.rs initially
        fs::write(self.project_path.join("src/lib.rs"), "")
            .context("Failed to write initial lib.rs")?;

        Ok(())
    }

    /// Write the AI-generated code to src/lib.rs
    pub fn write_code(&self, code: &str) -> Result<()> {
        let file_path = self.project_path.join("src/lib.rs");
        fs::write(&file_path, code)
            .context("Failed to write generated code to file")?;
        Ok(())
    }

    pub fn path(&self) -> &Path {
        &self.project_path
    }
}

/// Helper to extract Rust code blocks from Markdown.
pub fn extract_rust_code(markdown: &str) -> Option<String> {
    use regex::Regex;
    
    // Look for ```rust ... ``` block
    let re = Regex::new(r"```rust\s*([\s\S]*?)```").unwrap();
    if let Some(cap) = re.captures(markdown) {
        return Some(cap[1].to_string());
    }

    // Fallback: look for generic ``` ... ``` block
    let re_generic = Regex::new(r"```\s*([\s\S]*?)```").unwrap();
    if let Some(cap) = re_generic.captures(markdown) {
        return Some(cap[1].to_string());
    }

    // Fallback: assume the whole text is code if no blocks found (dangerous but useful for raw output)
    if !markdown.contains("```") {
         return Some(markdown.to_string());
    }

    None
}
