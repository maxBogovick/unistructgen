//! Type inference strategies for JSON values
//!
//! This module provides extensible type inference through the Strategy pattern.
//! Instead of hardcoding type detection logic, you can compose different detectors
//! and inference strategies.

use crate::core::{IRTypeRef, PrimitiveKind};
use serde_json::Value;
use thiserror::Error;

/// Errors that can occur during type inference
#[derive(Error, Debug)]
pub enum InferenceError {
    /// Failed to infer type
    #[error("Cannot infer type for value: {value}")]
    CannotInfer {
        /// The value that failed inference
        value: String,
        /// Optional hint about what was expected
        hint: Option<String>,
    },

    /// Detector-specific error
    #[error("Detector '{detector}' error: {message}")]
    DetectorError {
        /// Name of the detector
        detector: String,
        /// Error message
        message: String,
    },

    /// Ambiguous type
    #[error("Ambiguous type: multiple detectors matched")]
    Ambiguous {
        /// Detectors that matched
        matches: Vec<String>,
    },
}

impl InferenceError {
    /// Create a cannot infer error
    pub fn cannot_infer(value: impl Into<String>) -> Self {
        Self::CannotInfer {
            value: value.into(),
            hint: None,
        }
    }

    /// Add a hint to a cannot infer error
    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        if let Self::CannotInfer { hint: h, .. } = &mut self {
            *h = Some(hint.into());
        }
        self
    }

    /// Create a detector error
    pub fn detector_error(detector: impl Into<String>, message: impl Into<String>) -> Self {
        Self::DetectorError {
            detector: detector.into(),
            message: message.into(),
        }
    }
}

/// Trait for type inference strategies
///
/// A type inference strategy determines how to convert JSON values to IR types.
///
/// # Examples
///
/// ```
/// use serde_json::Value;
/// use unistructgen::parsers::json::inference::{TypeInferenceStrategy, InferenceError};
/// use unistructgen::core::{IRTypeRef, PrimitiveKind};
///
/// struct SimpleInference;
///
/// impl TypeInferenceStrategy for SimpleInference {
///     fn infer(&self, value: &Value, _hint: &str) -> Result<IRTypeRef, InferenceError> {
///         match value {
///             Value::String(_) => Ok(IRTypeRef::Primitive(PrimitiveKind::String)),
///             Value::Number(_) => Ok(IRTypeRef::Primitive(PrimitiveKind::I64)),
///             Value::Bool(_) => Ok(IRTypeRef::Primitive(PrimitiveKind::Bool)),
///             _ => Err(InferenceError::cannot_infer("unsupported type")),
///         }
///     }
/// }
/// ```
pub trait TypeInferenceStrategy: Send + Sync {
    /// Infer the IR type from a JSON value
    ///
    /// # Arguments
    ///
    /// * `value` - The JSON value to infer from
    /// * `hint` - Optional hint about the expected type (e.g., field name)
    fn infer(&self, value: &Value, hint: &str) -> Result<IRTypeRef, InferenceError>;
}

/// Trait for custom type detectors
///
/// Detectors check if a string value matches a specific pattern or format,
/// and return the appropriate IR type if it does.
///
/// # Examples
///
/// ```
/// use unistructgen::parsers::json::inference::{CustomTypeDetector, InferenceError};
/// use unistructgen::core::{IRTypeRef, PrimitiveKind};
///
/// struct EmailDetector;
///
/// impl CustomTypeDetector for EmailDetector {
///     fn name(&self) -> &str {
///         "EmailDetector"
///     }
///
///     fn detect(&self, value: &str) -> Result<Option<IRTypeRef>, InferenceError> {
///         if value.contains('@') && value.contains('.') {
///             // In a real implementation, use proper email validation
///             Ok(Some(IRTypeRef::Named("Email".to_string())))
///         } else {
///             Ok(None)
///         }
///     }
/// }
/// ```
pub trait CustomTypeDetector: Send + Sync {
    /// Name of this detector (for error messages)
    fn name(&self) -> &str;

    /// Try to detect if this value matches the detector's pattern
    ///
    /// Returns:
    /// - `Ok(Some(type))` if the value matches
    /// - `Ok(None)` if the value doesn't match
    /// - `Err(...)` if detection failed
    fn detect(&self, value: &str) -> Result<Option<IRTypeRef>, InferenceError>;

    /// Optional description of what this detector does
    fn description(&self) -> Option<&str> {
        None
    }
}

/// DateTime detector
///
/// Detects ISO 8601 datetime strings.
pub struct DateTimeDetector;

impl DateTimeDetector {
    /// Create a new DateTime detector
    pub fn new() -> Self {
        Self
    }

    /// Check if a string looks like an ISO 8601 datetime
    fn is_datetime(&self, s: &str) -> bool {
        // Simple heuristic: contains 'T' and ends with 'Z' or has timezone
        (s.contains('T') && (s.ends_with('Z') || s.contains('+') || s.contains('-')))
            || s.contains("T00:00:00")
    }
}

impl Default for DateTimeDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl CustomTypeDetector for DateTimeDetector {
    fn name(&self) -> &str {
        "DateTimeDetector"
    }

    fn description(&self) -> Option<&str> {
        Some("Detects ISO 8601 datetime strings")
    }

    fn detect(&self, value: &str) -> Result<Option<IRTypeRef>, InferenceError> {
        if self.is_datetime(value) {
            Ok(Some(IRTypeRef::Primitive(PrimitiveKind::DateTime)))
        } else {
            Ok(None)
        }
    }
}

/// UUID detector
///
/// Detects UUID strings in standard format (8-4-4-4-12).
pub struct UuidDetector;

impl UuidDetector {
    /// Create a new UUID detector
    pub fn new() -> Self {
        Self
    }

    /// Check if a string looks like a UUID
    fn is_uuid(&self, s: &str) -> bool {
        // Simple heuristic: 36 chars with dashes at positions 8, 13, 18, 23
        s.len() == 36
            && s.chars().nth(8) == Some('-')
            && s.chars().nth(13) == Some('-')
            && s.chars().nth(18) == Some('-')
            && s.chars().nth(23) == Some('-')
    }
}

impl Default for UuidDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl CustomTypeDetector for UuidDetector {
    fn name(&self) -> &str {
        "UuidDetector"
    }

    fn description(&self) -> Option<&str> {
        Some("Detects UUID strings (8-4-4-4-12 format)")
    }

    fn detect(&self, value: &str) -> Result<Option<IRTypeRef>, InferenceError> {
        if self.is_uuid(value) {
            Ok(Some(IRTypeRef::Primitive(PrimitiveKind::Uuid)))
        } else {
            Ok(None)
        }
    }
}

/// Email detector
///
/// Detects email addresses.
pub struct EmailDetector;

impl EmailDetector {
    /// Create a new email detector
    pub fn new() -> Self {
        Self
    }

    /// Simple email validation heuristic
    fn is_email(&self, s: &str) -> bool {
        s.contains('@') && s.contains('.') && s.len() > 3
    }
}

impl Default for EmailDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl CustomTypeDetector for EmailDetector {
    fn name(&self) -> &str {
        "EmailDetector"
    }

    fn description(&self) -> Option<&str> {
        Some("Detects email addresses")
    }

    fn detect(&self, value: &str) -> Result<Option<IRTypeRef>, InferenceError> {
        if self.is_email(value) {
            Ok(Some(IRTypeRef::Named("Email".to_string())))
        } else {
            Ok(None)
        }
    }
}

/// URL detector
///
/// Detects HTTP/HTTPS URLs.
pub struct UrlDetector;

impl UrlDetector {
    /// Create a new URL detector
    pub fn new() -> Self {
        Self
    }

    /// Check if a string looks like a URL
    fn is_url(&self, s: &str) -> bool {
        s.starts_with("http://") || s.starts_with("https://")
    }
}

impl Default for UrlDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl CustomTypeDetector for UrlDetector {
    fn name(&self) -> &str {
        "UrlDetector"
    }

    fn description(&self) -> Option<&str> {
        Some("Detects HTTP/HTTPS URLs")
    }

    fn detect(&self, value: &str) -> Result<Option<IRTypeRef>, InferenceError> {
        if self.is_url(value) {
            Ok(Some(IRTypeRef::Named("Url".to_string())))
        } else {
            Ok(None)
        }
    }
}

/// Smart type inference strategy
///
/// Uses multiple detectors to intelligently infer types from JSON values.
///
/// # Examples
///
/// ```
/// use unistructgen::parsers::json::inference::{SmartTypeInference, TypeInferenceStrategy};
/// use serde_json::json;
///
/// let inference = SmartTypeInference::new();
///
/// let datetime = json!("2024-01-15T10:30:00Z");
/// let ty = inference.infer(&datetime, "created_at").unwrap();
/// // Returns IRTypeRef::Primitive(PrimitiveKind::DateTime)
///
/// let uuid = json!("550e8400-e29b-41d4-a716-446655440000");
/// let ty = inference.infer(&uuid, "id").unwrap();
/// // Returns IRTypeRef::Primitive(PrimitiveKind::Uuid)
/// ```
pub struct SmartTypeInference {
    custom_detectors: Vec<Box<dyn CustomTypeDetector>>,
    datetime_detector: DateTimeDetector,
    uuid_detector: UuidDetector,
}

impl SmartTypeInference {
    /// Create a new smart type inference strategy
    pub fn new() -> Self {
        Self {
            custom_detectors: Vec::new(),
            datetime_detector: DateTimeDetector::new(),
            uuid_detector: UuidDetector::new(),
        }
    }

    /// Add a custom detector
    pub fn add_detector(mut self, detector: Box<dyn CustomTypeDetector>) -> Self {
        self.custom_detectors.push(detector);
        self
    }

    /// Add multiple custom detectors
    pub fn add_detectors(mut self, detectors: Vec<Box<dyn CustomTypeDetector>>) -> Self {
        self.custom_detectors.extend(detectors);
        self
    }

    /// Create with common detectors (Email, URL)
    pub fn with_common_detectors() -> Self {
        Self::new()
            .add_detector(Box::new(EmailDetector::new()))
            .add_detector(Box::new(UrlDetector::new()))
    }

    /// Infer type for a string value
    fn infer_string(&self, s: &str) -> Result<IRTypeRef, InferenceError> {
        // Try custom detectors first
        for detector in &self.custom_detectors {
            if let Some(ty) = detector.detect(s)? {
                return Ok(ty);
            }
        }

        // Try built-in detectors
        if let Some(ty) = self.datetime_detector.detect(s)? {
            return Ok(ty);
        }

        if let Some(ty) = self.uuid_detector.detect(s)? {
            return Ok(ty);
        }

        // Default to string
        Ok(IRTypeRef::Primitive(PrimitiveKind::String))
    }
}

impl Default for SmartTypeInference {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeInferenceStrategy for SmartTypeInference {
    fn infer(&self, value: &Value, _hint: &str) -> Result<IRTypeRef, InferenceError> {
        match value {
            Value::Null => Ok(IRTypeRef::Primitive(PrimitiveKind::String)), // Or Option<String>
            Value::Bool(_) => Ok(IRTypeRef::Primitive(PrimitiveKind::Bool)),
            Value::Number(n) => {
                if n.is_f64() {
                    Ok(IRTypeRef::Primitive(PrimitiveKind::F64))
                } else if n.is_u64() {
                    Ok(IRTypeRef::Primitive(PrimitiveKind::U64))
                } else {
                    Ok(IRTypeRef::Primitive(PrimitiveKind::I64))
                }
            }
            Value::String(s) => self.infer_string(s),
            Value::Array(arr) => {
                if let Some(first) = arr.first() {
                    let inner = self.infer(first, _hint)?;
                    Ok(IRTypeRef::Vec(Box::new(inner)))
                } else {
                    // Empty array, default to Vec<String>
                    Ok(IRTypeRef::Vec(Box::new(IRTypeRef::Primitive(
                        PrimitiveKind::String,
                    ))))
                }
            }
            Value::Object(_) => {
                // Will be handled by the parser to create a nested struct
                Ok(IRTypeRef::Named("Object".to_string()))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_datetime_detector() {
        let detector = DateTimeDetector::new();

        assert!(detector.detect("2024-01-15T10:30:00Z").unwrap().is_some());
        assert!(detector
            .detect("2024-01-15T10:30:00+00:00")
            .unwrap()
            .is_some());
        assert!(detector.detect("not a datetime").unwrap().is_none());
    }

    #[test]
    fn test_uuid_detector() {
        let detector = UuidDetector::new();

        assert!(detector
            .detect("550e8400-e29b-41d4-a716-446655440000")
            .unwrap()
            .is_some());
        assert!(detector.detect("not-a-uuid").unwrap().is_none());
    }

    #[test]
    fn test_email_detector() {
        let detector = EmailDetector::new();

        assert!(detector
            .detect("user@example.com")
            .unwrap()
            .is_some());
        assert!(detector.detect("not-an-email").unwrap().is_none());
    }

    #[test]
    fn test_url_detector() {
        let detector = UrlDetector::new();

        assert!(detector
            .detect("https://example.com")
            .unwrap()
            .is_some());
        assert!(detector.detect("http://example.com").unwrap().is_some());
        assert!(detector.detect("not-a-url").unwrap().is_none());
    }

    #[test]
    fn test_smart_inference_datetime() {
        let inference = SmartTypeInference::new();
        let value = json!("2024-01-15T10:30:00Z");

        let result = inference.infer(&value, "created_at").unwrap();

        assert!(matches!(
            result,
            IRTypeRef::Primitive(PrimitiveKind::DateTime)
        ));
    }

    #[test]
    fn test_smart_inference_uuid() {
        let inference = SmartTypeInference::new();
        let value = json!("550e8400-e29b-41d4-a716-446655440000");

        let result = inference.infer(&value, "id").unwrap();

        assert!(matches!(result, IRTypeRef::Primitive(PrimitiveKind::Uuid)));
    }

    #[test]
    fn test_smart_inference_with_custom_detector() {
        let inference = SmartTypeInference::new().add_detector(Box::new(EmailDetector::new()));

        let value = json!("user@example.com");
        let result = inference.infer(&value, "email").unwrap();

        assert!(matches!(result, IRTypeRef::Named(_)));
        if let IRTypeRef::Named(name) = result {
            assert_eq!(name, "Email");
        }
    }

    #[test]
    fn test_smart_inference_numbers() {
        let inference = SmartTypeInference::new();

        // Integer
        let int_value = json!(42);
        if let IRTypeRef::Primitive(kind) = inference.infer(&int_value, "").unwrap() {
            assert!(matches!(kind, PrimitiveKind::I64 | PrimitiveKind::U64));
        } else {
            assert!(false, "Expected number type");
        }

        // Float
        let float_value = json!(3.14);
        assert!(matches!(
            inference.infer(&float_value, "").unwrap(),
            IRTypeRef::Primitive(PrimitiveKind::F64)
        ));
    }

    #[test]
    fn test_smart_inference_array() {
        let inference = SmartTypeInference::new();
        let value = json!(["a", "b", "c"]);

        let result = inference.infer(&value, "").unwrap();

        if let IRTypeRef::Vec(inner) = result {
            assert!(matches!(
                *inner,
                IRTypeRef::Primitive(PrimitiveKind::String)
            ));
        } else {
            assert!(false, "Expected Vec");
        }
    }
}
