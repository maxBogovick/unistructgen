use anyhow::Result;
use colored::*;
use schema_registry_common::{ChangeType, SchemaDiff};
use serde::Serialize;

use crate::client::RegistryClient;

#[derive(Serialize)]
struct DiffRequest {
    from_version: String,
    to_version: String,
}

pub async fn diff_schemas(
    client: &RegistryClient,
    name: String,
    from: String,
    to: String,
) -> Result<()> {
    println!("{} Comparing versions {} → {}", "→".cyan(), from.bold(), to.bold());

    let request = DiffRequest {
        from_version: from.clone(),
        to_version: to.clone(),
    };

    let diff: SchemaDiff = client.post(&format!("/api/diff/{}", name), &request).await?;

    if diff.is_breaking {
        println!("\n{} Breaking changes detected!", "⚠".red().bold());
    } else {
        println!("\n{} No breaking changes", "✓".green());
    }

    if diff.changes.is_empty() {
        println!("  No changes found");
        return Ok(());
    }

    println!("\n{}", "Changes:".bold());
    for change in &diff.changes {
        let (icon, color_fn): (&str, fn(&str) -> colored::ColoredString) = match change.change_type {
            ChangeType::Breaking => ("✗", |s| s.red()),
            ChangeType::NonBreaking => ("✓", |s| s.green()),
        };
        println!("  {} {}", icon, color_fn(&change.description));
        println!("    Path: {}", change.path.dimmed());
    }

    Ok(())
}
