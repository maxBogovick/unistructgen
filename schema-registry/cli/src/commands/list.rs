use anyhow::Result;
use colored::*;
use comfy_table::{presets::UTF8_FULL, Table};
use schema_registry_common::Schema;

use crate::client::RegistryClient;

pub async fn list_schemas(client: &RegistryClient, team: Option<String>) -> Result<()> {
    let path = if let Some(t) = &team {
        format!("/api/schemas?team={}", t)
    } else {
        "/api/schemas".to_string()
    };

    let schemas: Vec<Schema> = client.get(&path).await?;

    if schemas.is_empty() {
        println!("{} No schemas found", "ℹ".blue());
        return Ok(());
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_header(vec!["Name", "Version", "Format", "Team", "Created"]);

    for schema in schemas {
        table.add_row(vec![
            schema.name,
            schema.version,
            schema.format.to_string(),
            schema.team,
            schema.created_at.format("%Y-%m-%d %H:%M").to_string(),
        ]);
    }

    println!("{}", table);
    Ok(())
}
