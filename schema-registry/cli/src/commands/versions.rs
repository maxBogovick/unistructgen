use anyhow::Result;
use colored::*;

use crate::client::RegistryClient;

pub async fn list_versions(client: &RegistryClient, name: String) -> Result<()> {
    let versions: Vec<String> = client.get(&format!("/api/schemas/{}/versions", name)).await?;

    println!("{} for {}", "Versions".bold(), name.cyan());
    for version in versions {
        println!("  • {}", version);
    }

    Ok(())
}
