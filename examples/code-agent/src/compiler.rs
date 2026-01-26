use std::process::Command;
use std::path::Path;
use anyhow::{Result, Context};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct CargoMessage {
    reason: String,
    message: Option<Diagnostic>,
}

#[derive(Debug, Deserialize)]
struct Diagnostic {
    message: String,
    level: String, // "error", "warning", etc.
    spans: Vec<DiagnosticSpan>,
    rendered: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DiagnosticSpan {
    line_start: usize,
    column_start: usize,
    file_name: String,
}

#[derive(Debug, Clone)]
pub struct CompilerError {
    pub message: String,
    pub location: Option<String>,
    pub rendered: String,
}

pub struct Compiler;

impl Compiler {
    /// Runs `cargo check` in the given project path and returns a list of errors.
    pub fn check(project_path: &Path) -> Result<Vec<CompilerError>> {
        let output = Command::new("cargo")
            .arg("check")
            .arg("--message-format=json")
            .arg("--quiet")
            .current_dir(project_path)
            .output()
            .context("Failed to execute cargo check")?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut errors = Vec::new();

        for line in stdout.lines() {
            if line.trim().is_empty() { continue; }
            
            // Parse JSON line
            if let Ok(msg) = serde_json::from_str::<CargoMessage>(line) {
                if msg.reason == "compiler-message" {
                    if let Some(diag) = msg.message {
                        if diag.level == "error" {
                            let location = diag.spans.first().map(|s| {
                                format!("{}:{}:{}", s.file_name, s.line_start, s.column_start)
                            });
                            
                            errors.push(CompilerError {
                                message: diag.message,
                                location,
                                rendered: diag.rendered.unwrap_or_default(),
                            });
                        }
                    }
                }
            }
        }
        
        // Sometimes cargo outputs hard errors to stderr if it can't parse manifest, etc.
        if !output.status.success() && errors.is_empty() {
             let stderr = String::from_utf8_lossy(&output.stderr);
             if !stderr.is_empty() {
                 errors.push(CompilerError {
                     message: "Cargo execution failed".to_string(),
                     location: None,
                     rendered: stderr.to_string(),
                 });
             }
        }

        Ok(errors)
    }
}
