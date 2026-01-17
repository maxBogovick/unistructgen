//! Schema diff and breaking change detection

use crate::{ChangeType, SchemaChange, SchemaDiff, SchemaFormat};
use serde_json::Value;

/// Compare two schemas and detect changes
pub fn compare_schemas(
    old_content: &str,
    new_content: &str,
    format: SchemaFormat,
    old_version: String,
    new_version: String,
) -> crate::Result<SchemaDiff> {
    match format {
        SchemaFormat::OpenApi => compare_openapi(old_content, new_content, old_version, new_version),
        SchemaFormat::JsonSchema => compare_json_schema(old_content, new_content, old_version, new_version),
        _ => {
            // For now, basic comparison for other formats
            Ok(SchemaDiff {
                from_version: old_version,
                to_version: new_version,
                changes: vec![],
                is_breaking: false,
            })
        }
    }
}

fn compare_openapi(
    old: &str,
    new: &str,
    old_version: String,
    new_version: String,
) -> crate::Result<SchemaDiff> {
    let old_spec: Value = serde_yaml::from_str(old)
        .map_err(|e| crate::Error::InvalidSchema(e.to_string()))?;
    let new_spec: Value = serde_yaml::from_str(new)
        .map_err(|e| crate::Error::InvalidSchema(e.to_string()))?;

    let mut changes = Vec::new();

    // Check for removed paths
    if let (Some(old_paths), Some(new_paths)) = (
        old_spec.get("paths").and_then(|v| v.as_object()),
        new_spec.get("paths").and_then(|v| v.as_object()),
    ) {
        for (path, _) in old_paths {
            if !new_paths.contains_key(path) {
                changes.push(SchemaChange {
                    change_type: ChangeType::Breaking,
                    description: format!("Removed endpoint: {}", path),
                    path: format!("paths.{}", path),
                });
            }
        }

        // Check for added paths
        for (path, _) in new_paths {
            if !old_paths.contains_key(path) {
                changes.push(SchemaChange {
                    change_type: ChangeType::NonBreaking,
                    description: format!("Added endpoint: {}", path),
                    path: format!("paths.{}", path),
                });
            }
        }
    }

    // Check schema changes
    if let (Some(old_schemas), Some(new_schemas)) = (
        old_spec.get("components")
            .and_then(|v| v.get("schemas"))
            .and_then(|v| v.as_object()),
        new_spec.get("components")
            .and_then(|v| v.get("schemas"))
            .and_then(|v| v.as_object()),
    ) {
        for (schema_name, old_schema) in old_schemas {
            if let Some(new_schema) = new_schemas.get(schema_name) {
                // Compare properties
                if let (Some(old_props), Some(new_props)) = (
                    old_schema.get("properties").and_then(|v| v.as_object()),
                    new_schema.get("properties").and_then(|v| v.as_object()),
                ) {
                    // Removed fields
                    for (field, _) in old_props {
                        if !new_props.contains_key(field) {
                            changes.push(SchemaChange {
                                change_type: ChangeType::Breaking,
                                description: format!("Removed field: {}.{}", schema_name, field),
                                path: format!("components.schemas.{}.properties.{}", schema_name, field),
                            });
                        }
                    }

                    // Added fields
                    for (field, value) in new_props {
                        if !old_props.contains_key(field) {
                            let is_required = new_schema
                                .get("required")
                                .and_then(|v| v.as_array())
                                .map(|arr| arr.iter().any(|v| v.as_str() == Some(field)))
                                .unwrap_or(false);

                            let change_type = if is_required {
                                ChangeType::Breaking
                            } else {
                                ChangeType::NonBreaking
                            };

                            changes.push(SchemaChange {
                                change_type,
                                description: format!(
                                    "Added field: {}.{} ({})",
                                    schema_name,
                                    field,
                                    if is_required { "required" } else { "optional" }
                                ),
                                path: format!("components.schemas.{}.properties.{}", schema_name, field),
                            });
                        } else {
                            // Check for type changes
                            if let (Some(old_type), Some(new_type)) = (
                                old_props.get(field).and_then(|v| v.get("type")),
                                value.get("type"),
                            ) {
                                if old_type != new_type {
                                    changes.push(SchemaChange {
                                        change_type: ChangeType::Breaking,
                                        description: format!(
                                            "Changed type: {}.{} ({} → {})",
                                            schema_name,
                                            field,
                                            old_type.as_str().unwrap_or("unknown"),
                                            new_type.as_str().unwrap_or("unknown")
                                        ),
                                        path: format!("components.schemas.{}.properties.{}.type", schema_name, field),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let is_breaking = changes.iter().any(|c| matches!(c.change_type, ChangeType::Breaking));

    Ok(SchemaDiff {
        from_version: old_version,
        to_version: new_version,
        changes,
        is_breaking,
    })
}

fn compare_json_schema(
    _old: &str,
    _new: &str,
    old_version: String,
    new_version: String,
) -> crate::Result<SchemaDiff> {
    // Simplified JSON Schema comparison
    Ok(SchemaDiff {
        from_version: old_version,
        to_version: new_version,
        changes: vec![],
        is_breaking: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_removed_endpoint() {
        let old = r#"
openapi: 3.0.0
paths:
  /users:
    get: {}
  /users/{id}:
    delete: {}
"#;

        let new = r#"
openapi: 3.0.0
paths:
  /users:
    get: {}
"#;

        let diff = compare_openapi(old, new, "1.0.0".to_string(), "2.0.0".to_string()).unwrap();
        assert!(diff.is_breaking);
        assert_eq!(diff.changes.len(), 1);
        assert!(diff.changes[0].description.contains("Removed endpoint"));
    }

    #[test]
    fn test_detect_field_removal() {
        let old = r#"
openapi: 3.0.0
components:
  schemas:
    User:
      properties:
        id: { type: string }
        name: { type: string }
        phone: { type: string }
"#;

        let new = r#"
openapi: 3.0.0
components:
  schemas:
    User:
      properties:
        id: { type: string }
        name: { type: string }
"#;

        let diff = compare_openapi(old, new, "1.0.0".to_string(), "2.0.0".to_string()).unwrap();
        assert!(diff.is_breaking);
        assert!(diff.changes.iter().any(|c| c.description.contains("phone")));
    }
}
