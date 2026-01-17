use anyhow::Result;
use colored::*;
use schema_registry_common::Schema;

use crate::client::RegistryClient;

pub async fn get_schema(
    client: &RegistryClient,
    name: String,
    version: Option<String>,
) -> Result<()> {
    let path = if let Some(ver) = version {
        format!("/api/schemas/{}/{}", name, ver)
    } else {
        format!("/api/schemas/{}", name)
    };

    let schema: Schema = client.get(&path).await?;

    println!("{}", "Schema Details".bold());
    println!("  {}: {}", "Name".cyan(), schema.name);
    println!("  {}: {}", "Version".cyan(), schema.version);
    println!("  {}: {}", "Format".cyan(), schema.format);
    println!("  {}: {}", "Team".cyan(), schema.team);
    println!("  {}: {}", "Created".cyan(), schema.created_at);
    if let Some(desc) = schema.description {
        println!("  {}: {}", "Description".cyan(), desc);
    }
    println!("\n{}", "Content:".bold());
    println!("{}", schema.content);

    Ok(())
}
