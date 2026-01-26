use unistructgen_core::validation::{ValidationReport, AiValidationError, map_serde_error};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct User {
    id: i64,
    name: String,
}

#[test]
fn test_validation_report_empty() {
    let report = ValidationReport::valid();
    assert!(report.is_valid);
    assert_eq!(report.to_correction_prompt(), "No errors found.");
}

#[test]
fn test_validation_report_with_errors() {
    let mut report = ValidationReport::new();
    report.add_error(AiValidationError {
        path: "users[0].age".to_string(),
        message: "Value must be positive".to_string(),
        invalid_value: Some("-5".to_string()),
        correction_hint: Some("Set age to a positive integer".to_string()),
    });

    assert!(!report.is_valid);
    let prompt = report.to_correction_prompt();
    assert!(prompt.contains("The generated JSON response was invalid"));
    assert!(prompt.contains("Field `users[0].age`: Value must be positive"));
    assert!(prompt.contains("Hint: Set age to a positive integer"));
}

#[test]
fn test_map_serde_error() {
    let json = r#"{"id": "not_an_int", "name": "Alice"}"#;
    let result: Result<User, _> = serde_json::from_str(json);
    
    assert!(result.is_err());
    let err = result.unwrap_err();
    
    let validation_error = map_serde_error(&err);
    // Note: Serde error message formats vary, but "invalid type" usually contains info
    // Standard serde_json err: "invalid type: string \"not_an_int\", expected i64 at line 1 column 19"
    
    // Our simplistic mapper might not catch field names perfectly for all error types yet,
    // but let's check it produces a usable error object.
    assert!(!validation_error.message.is_empty());
    
    // Create a report from it
    let mut report = ValidationReport::new();
    report.add_error(validation_error);
    let prompt = report.to_correction_prompt();
    assert!(prompt.contains("Field"));
}
