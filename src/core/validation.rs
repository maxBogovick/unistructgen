use serde::{Serialize, Deserialize};
use std::fmt;

/// Represents a validation error intended for an AI model to understand and correct.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AiValidationError {
    /// The path to the field that failed validation (e.g., "users[0].age").
    pub path: String,
    /// A human-readable (and AI-readable) description of the error.
    pub message: String,
    /// The value that caused the error (optional).
    pub invalid_value: Option<String>,
    /// A hint for correction (optional).
    pub correction_hint: Option<String>,
}

impl fmt::Display for AiValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error at '{}': {}", self.path, self.message)
    }
}

/// A report containing all validation errors found in a response.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct ValidationReport {
    pub is_valid: bool,
    pub errors: Vec<AiValidationError>,
}

impl ValidationReport {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn valid() -> Self {
        Self { is_valid: true, errors: Vec::new() }
    }

    pub fn add_error(&mut self, error: AiValidationError) {
        self.is_valid = false;
        self.errors.push(error);
    }
    
    /// Returns a formatted prompt instructing the AI how to fix the errors.
    pub fn to_correction_prompt(&self) -> String {
        if self.is_valid {
            return "No errors found.".to_string();
        }

        let mut prompt = String::from("The generated JSON response was invalid. Please fix the following errors:\n");
        for (i, err) in self.errors.iter().enumerate() {
            prompt.push_str(&format!("{}. Field `{}`: {}
", i + 1, err.path, err.message));
            if let Some(hint) = &err.correction_hint {
                prompt.push_str(&format!("   Hint: {}
", hint));
            }
        }
        prompt.push_str("\nReturn the corrected JSON only.");
        prompt
    }
}

/// Helper to map serde_json errors to AiValidationErrors.
/// Note: Standard serde_json errors provide line/column, but not always the path.
/// This helper attempts to provide a best-effort structured error.
pub fn map_serde_error(err: &serde_json::Error) -> AiValidationError {
    // serde_json errors are text-based, like "missing field `id` at line 1 column 123"
    let msg = err.to_string();
    
    // Heuristics to extract field name
    let path = if let Some(start) = msg.find("field `") {
        if let Some(_end) = msg[start..].find('`') { // Finding closing quote logic is tricky with offsets
             // Simple extraction
             let after_field = &msg[start + 7..];
             if let Some(end_quote) = after_field.find('`') {
                 after_field[..end_quote].to_string()
             } else {
                 "unknown".to_string()
             }
        } else {
            "unknown".to_string()
        }
    } else {
        "unknown".to_string()
    };

    AiValidationError {
        path,
        message: msg,
        invalid_value: None,
        correction_hint: Some("Ensure the field name and type matches the schema exactly.".to_string()),
    }
}

/// Trait for types that can validate themselves for AI contexts.
pub trait AiValidatable {
    fn validate_ai(&self) -> ValidationReport;
}
