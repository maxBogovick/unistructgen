//! Upload schema command

use anyhow::{Context, Result};
use colored::*;
use schema_registry_common::{Schema, SchemaFormat};
use std::path::PathBuf;

use crate::client::RegistryClient;

pub async fn upload(
    client: &RegistryClient,
    name: String,
    version: String,
    file: PathBuf,
    team: String,
    format_str: String,
    description: Option<String>,
) -> Result<()> {
    println!("{} Uploading schema...", "→".cyan());
    println!("  Name:    {}", name.bold());
    println!("  Version: {}", version.bold());
    println!("  Team:    {}", team);
    println!("  Format:  {}", format_str);

    // Read file content
    let content = std::fs::read_to_string(&file)
        .with_context(|| format!("Failed to read file: {}", file.display()))?;

    let format = match format_str.to_lowercase().as_str() {
        "openapi" => SchemaFormat::OpenApi,
        "graphql" => SchemaFormat::GraphQL,
        "protobuf" => SchemaFormat::Protobuf,
        "jsonschema" => SchemaFormat::JsonSchema,
        _ => anyhow::bail!("Unknown format: {}", format_str),
    };

    #[derive(serde::Serialize)]
    struct Request {
        name: String,
        version: String,
        format: SchemaFormat,
        team: String,
        description: Option<String>,
        content: String,
        created_by: String,
    }

    let request = Request {
        name,
        version,
        format,
        team,
        description,
        content,
        created_by: whoami::username(),
    };

    let schema: Schema = client.post("/api/schemas", &request).await?;

    println!("{} Schema uploaded successfully!", "✓".green());
    println!("  ID: {}", schema.id);

    Ok(())
}
