use anyhow::Result;
use colored::*;
use schema_registry_common::SchemaStats;

use crate::client::RegistryClient;

pub async fn show_stats(client: &RegistryClient) -> Result<()> {
    let stats: SchemaStats = client.get("/api/stats").await?;

    println!("{}", "Registry Statistics".bold());
    println!();
    println!("  {}: {}", "Total Schemas".cyan(), stats.total_schemas);
    println!("  {}: {}", "Total Versions".cyan(), stats.total_versions);
    println!("  {}: {}", "Generations Today".cyan(), stats.generations_today);
    println!();

    if !stats.most_used.is_empty() {
        println!("{}", "Most Used Schemas:".bold());
        for (name, count) in stats.most_used {
            println!("  • {} ({} generations)", name, count);
        }
    }

    Ok(())
}
