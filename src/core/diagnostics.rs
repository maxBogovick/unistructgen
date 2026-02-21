use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DiagnosticError {
    #[error("Failed to run cargo check: {0}")]
    CargoRunError(std::io::Error),
    #[error("Cargo execution failed: {0}")]
    CargoExecutionError(String),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CargoMessage {
    pub reason: String,
    pub message: Option<Diagnostic>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Diagnostic {
    pub message: String,
    pub level: String, // "error", "warning", etc.
    pub spans: Vec<DiagnosticSpan>,
    pub rendered: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DiagnosticSpan {
    pub file_name: String,
    pub line_start: usize,
    pub line_end: usize,
    pub column_start: usize,
    pub column_end: usize,
    pub is_primary: bool,
    pub text: Vec<DiagnosticSpanLine>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DiagnosticSpanLine {
    pub text: String,
    pub highlight_start: usize,
    pub highlight_end: usize,
}

#[derive(Debug, Clone)]
pub struct CompilerError {
    pub message: String,
    pub primary_span: Option<DiagnosticSpan>,
    pub rendered: String,
}

pub struct CargoDiagnostics;

impl CargoDiagnostics {
    /// Runs `cargo check` in the given project path and returns a list of errors.
    pub fn check(project_path: &Path) -> Result<Vec<CompilerError>, DiagnosticError> {
        let output = Command::new("cargo")
            .arg("check")
            .arg("--message-format=json")
            .arg("--quiet") // Less noise on stderr
            .current_dir(project_path)
            .output()
            .map_err(DiagnosticError::CargoRunError)?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut errors = Vec::new();

        for line in stdout.lines() {
            if line.trim().is_empty() { continue; }
            
            // Parse JSON line
            if let Ok(msg) = serde_json::from_str::<CargoMessage>(line) {
                if msg.reason == "compiler-message" {
                    if let Some(diag) = msg.message {
                        if diag.level == "error" {
                            // Find the primary span (where the error actually is)
                            let primary_span = diag.spans.iter().find(|s| s.is_primary).cloned()
                                .or_else(|| diag.spans.first().cloned());
                            
                            errors.push(CompilerError {
                                message: diag.message,
                                primary_span,
                                rendered: diag.rendered.unwrap_or_default(),
                            });
                        }
                    }
                }
            }
        }
        
        // If cargo failed but gave no structured errors (e.g. failed to parse Cargo.toml)
        if !output.status.success() && errors.is_empty() {
             let stderr = String::from_utf8_lossy(&output.stderr);
             if !stderr.is_empty() {
                 errors.push(CompilerError {
                     message: "Cargo execution failed (non-compiler error)".to_string(),
                     primary_span: None,
                     rendered: stderr.to_string(),
                 });
             }
        }

        Ok(errors)
    }
}
