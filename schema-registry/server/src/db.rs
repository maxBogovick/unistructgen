//! Database models and queries

use chrono::{DateTime, Utc};
use schema_registry_common::{Schema, SchemaFormat, SchemaStats, Team};
use sqlx::PgPool;
use uuid::Uuid;

/// Database schema record
#[derive(Debug, sqlx::FromRow)]
pub struct SchemaRecord {
    pub id: Uuid,
    pub name: String,
    pub version: String,
    pub format: String,
    pub team_id: Uuid,
    pub description: Option<String>,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
}

impl SchemaRecord {
    pub fn into_schema(self) -> Schema {
        Schema {
            id: self.id,
            name: self.name,
            version: self.version,
            format: parse_format(&self.format),
            team: self.team_id.to_string(),
            description: self.description,
            content: self.content,
            created_at: self.created_at,
            created_by: self.created_by,
        }
    }
}

fn parse_format(s: &str) -> SchemaFormat {
    match s.to_lowercase().as_str() {
        "openapi" => SchemaFormat::OpenApi,
        "graphql" => SchemaFormat::GraphQL,
        "protobuf" => SchemaFormat::Protobuf,
        "jsonschema" => SchemaFormat::JsonSchema,
        _ => SchemaFormat::OpenApi,
    }
}

/// Get schema by name and version
pub async fn get_schema(
    pool: &PgPool,
    name: &str,
    version: &str,
) -> sqlx::Result<Option<Schema>> {
    let record = sqlx::query_as::<_, SchemaRecord>(
        "SELECT * FROM schemas WHERE name = $1 AND version = $2"
    )
    .bind(name)
    .bind(version)
    .fetch_optional(pool)
    .await?;

    Ok(record.map(|r| r.into_schema()))
}

/// Get latest version of schema
pub async fn get_latest_schema(
    pool: &PgPool,
    name: &str,
) -> sqlx::Result<Option<Schema>> {
    let record = sqlx::query_as::<_, SchemaRecord>(
        "SELECT * FROM schemas WHERE name = $1 ORDER BY created_at DESC LIMIT 1"
    )
    .bind(name)
    .fetch_optional(pool)
    .await?;

    Ok(record.map(|r| r.into_schema()))
}

/// List all schemas for a team
pub async fn list_schemas(
    pool: &PgPool,
    team_name: Option<&str>,
) -> sqlx::Result<Vec<Schema>> {
    let records = if let Some(team) = team_name {
        sqlx::query_as::<_, SchemaRecord>(
            "SELECT s.* FROM schemas s
             JOIN teams t ON s.team_id = t.id
             WHERE t.name = $1
             ORDER BY s.name, s.created_at DESC"
        )
        .bind(team)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as::<_, SchemaRecord>(
            "SELECT * FROM schemas ORDER BY name, created_at DESC"
        )
        .fetch_all(pool)
        .await?
    };

    Ok(records.into_iter().map(|r| r.into_schema()).collect())
}

/// Create new schema
pub async fn create_schema(
    pool: &PgPool,
    name: String,
    version: String,
    format: SchemaFormat,
    team_name: &str,
    description: Option<String>,
    content: String,
    created_by: String,
) -> sqlx::Result<Schema> {
    // Get team ID
    let team_id: Uuid = sqlx::query_scalar(
        "SELECT id FROM teams WHERE name = $1"
    )
    .bind(team_name)
    .fetch_one(pool)
    .await?;

    let record = sqlx::query_as::<_, SchemaRecord>(
        "INSERT INTO schemas (name, version, format, team_id, description, content, created_by)
         VALUES ($1, $2, $3, $4, $5, $6, $7)
         RETURNING *"
    )
    .bind(&name)
    .bind(&version)
    .bind(format.as_str())
    .bind(team_id)
    .bind(description)
    .bind(content)
    .bind(created_by)
    .fetch_one(pool)
    .await?;

    Ok(record.into_schema())
}

/// List versions for a schema
pub async fn list_versions(
    pool: &PgPool,
    name: &str,
) -> sqlx::Result<Vec<String>> {
    let versions: Vec<String> = sqlx::query_scalar(
        "SELECT version FROM schemas WHERE name = $1 ORDER BY created_at DESC"
    )
    .bind(name)
    .fetch_all(pool)
    .await?;

    Ok(versions)
}

/// Record generation event
pub async fn record_generation(
    pool: &PgPool,
    schema_id: Uuid,
    target: &str,
    user_id: &str,
) -> sqlx::Result<()> {
    sqlx::query(
        "INSERT INTO generation_events (schema_id, target, user_id)
         VALUES ($1, $2, $3)"
    )
    .bind(schema_id)
    .bind(target)
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Get statistics
pub async fn get_stats(pool: &PgPool) -> sqlx::Result<SchemaStats> {
    let total_schemas: i64 = sqlx::query_scalar(
        "SELECT COUNT(DISTINCT name) FROM schemas"
    )
    .fetch_one(pool)
    .await?;

    let total_versions: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM schemas"
    )
    .fetch_one(pool)
    .await?;

    let generations_today: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM generation_events
         WHERE created_at > NOW() - INTERVAL '1 day'"
    )
    .fetch_one(pool)
    .await?;

    // Most used schemas
    let most_used: Vec<(String, i64)> = sqlx::query_as(
        "SELECT s.name, COUNT(g.id) as count
         FROM generation_events g
         JOIN schemas s ON g.schema_id = s.id
         GROUP BY s.name
         ORDER BY count DESC
         LIMIT 5"
    )
    .fetch_all(pool)
    .await?;

    Ok(SchemaStats {
        total_schemas,
        total_versions,
        generations_today,
        most_used,
    })
}

/// Team operations
#[derive(Debug, sqlx::FromRow)]
pub struct TeamRecord {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl TeamRecord {
    pub fn into_team(self) -> Team {
        Team {
            id: self.id,
            name: self.name,
            description: self.description,
            created_at: self.created_at,
        }
    }
}

pub async fn list_teams(pool: &PgPool) -> sqlx::Result<Vec<Team>> {
    let records = sqlx::query_as::<_, TeamRecord>(
        "SELECT * FROM teams ORDER BY name"
    )
    .fetch_all(pool)
    .await?;

    Ok(records.into_iter().map(|r| r.into_team()).collect())
}

pub async fn create_team(
    pool: &PgPool,
    name: String,
    description: Option<String>,
) -> sqlx::Result<Team> {
    let record = sqlx::query_as::<_, TeamRecord>(
        "INSERT INTO teams (name, description) VALUES ($1, $2) RETURNING *"
    )
    .bind(name)
    .bind(description)
    .fetch_one(pool)
    .await?;

    Ok(record.into_team())
}
