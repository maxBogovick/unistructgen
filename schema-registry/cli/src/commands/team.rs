use anyhow::Result;
use colored::*;
use comfy_table::{presets::UTF8_FULL, Table};
use schema_registry_common::Team;
use serde::Serialize;

use crate::client::RegistryClient;

pub async fn list_teams(client: &RegistryClient) -> Result<()> {
    let teams: Vec<Team> = client.get("/api/teams").await?;

    if teams.is_empty() {
        println!("{} No teams found", "ℹ".blue());
        return Ok(());
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_header(vec!["Name", "Description", "Created"]);

    for team in teams {
        table.add_row(vec![
            team.name,
            team.description.unwrap_or_else(|| "-".to_string()),
            team.created_at.format("%Y-%m-%d").to_string(),
        ]);
    }

    println!("{}", table);
    Ok(())
}

#[derive(Serialize)]
struct CreateTeamRequest {
    name: String,
    description: Option<String>,
}

pub async fn create_team(
    client: &RegistryClient,
    name: String,
    description: Option<String>,
) -> Result<()> {
    println!("{} Creating team '{}'...", "→".cyan(), name.bold());

    let request = CreateTeamRequest { name, description };
    let team: Team = client.post("/api/teams", &request).await?;

    println!("{} Team created successfully!", "✓".green());
    println!("  ID: {}", team.id);

    Ok(())
}
