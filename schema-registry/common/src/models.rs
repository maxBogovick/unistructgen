//! Data models for Schema Registry

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Schema format type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SchemaFormat {
    OpenApi,
    GraphQL,
    Protobuf,
    JsonSchema,
}

impl SchemaFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            SchemaFormat::OpenApi => "openapi",
            SchemaFormat::GraphQL => "graphql",
            SchemaFormat::Protobuf => "protobuf",
            SchemaFormat::JsonSchema => "jsonschema",
        }
    }
}

impl std::fmt::Display for SchemaFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Schema metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    pub id: Uuid,
    pub name: String,
    pub version: String,
    pub format: SchemaFormat,
    pub team: String,
    pub description: Option<String>,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
}

/// Schema version comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaVersion {
    pub version: String,
    pub created_at: DateTime<Utc>,
    pub is_breaking: bool,
    pub changelog: Option<String>,
}

/// Breaking change type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeType {
    Breaking,
    NonBreaking,
}

/// Schema change description
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaChange {
    pub change_type: ChangeType,
    pub description: String,
    pub path: String,
}

/// Schema diff result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaDiff {
    pub from_version: String,
    pub to_version: String,
    pub changes: Vec<SchemaChange>,
    pub is_breaking: bool,
}

/// Code generation target language
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GenerationTarget {
    Rust,
    TypeScript,
    Python,
    Go,
    Java,
    Kotlin,
    Swift,
    CSharp,
}

impl GenerationTarget {
    pub fn as_str(&self) -> &'static str {
        match self {
            GenerationTarget::Rust => "rust",
            GenerationTarget::TypeScript => "typescript",
            GenerationTarget::Python => "python",
            GenerationTarget::Go => "go",
            GenerationTarget::Java => "java",
            GenerationTarget::Kotlin => "kotlin",
            GenerationTarget::Swift => "swift",
            GenerationTarget::CSharp => "csharp",
        }
    }
}

impl std::fmt::Display for GenerationTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Code generation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationRequest {
    pub schema_name: String,
    pub version: String,
    pub targets: Vec<GenerationTarget>,
}

/// Code generation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationResult {
    pub target: GenerationTarget,
    pub files: Vec<GeneratedFile>,
}

/// Generated file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedFile {
    pub path: String,
    pub content: String,
}

/// Team information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Team {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Permission type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Permission {
    SchemasRead,
    SchemasWrite,
    SchemasDelete,
    GenerateRust,
    GenerateTypeScript,
    GeneratePython,
    GenerateGo,
    TeamAdmin,
}

/// Team member
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMember {
    pub team_id: Uuid,
    pub user_id: String,
    pub permissions: Vec<Permission>,
    pub joined_at: DateTime<Utc>,
}

/// Schema statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaStats {
    pub total_schemas: i64,
    pub total_versions: i64,
    pub generations_today: i64,
    pub most_used: Vec<(String, i64)>,
}

/// API response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(error: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error.into()),
        }
    }
}
