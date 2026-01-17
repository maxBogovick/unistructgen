use anyhow::{Context, Result};
use colored::*;
use schema_registry_common::{GenerationRequest, GenerationResult, GenerationTarget};
use std::path::PathBuf;

use crate::client::RegistryClient;

pub async fn generate(
    client: &RegistryClient,
    name: String,
    version: String,
    targets_str: String,
    output: PathBuf,
) -> Result<()> {
    // Parse targets
    let targets: Vec<GenerationTarget> = targets_str
        .split(',')
        .map(|s| parse_target(s.trim()))
        .collect::<Result<Vec<_>>>()?;

    println!("{} Generating code...", "→".cyan());
    println!("  Schema:  {} v{}", name.bold(), version);
    println!("  Targets: {}", targets_str);
    println!("  Output:  {}", output.display());

    let request = GenerationRequest {
        schema_name: name,
        version,
        targets: targets.clone(),
    };

    let results: Vec<GenerationResult> = client.post("/api/generate", &request).await?;

    // Write files
    for result in results {
        let target_dir = output.join(result.target.to_string());
        std::fs::create_dir_all(&target_dir)?;

        for file in result.files {
            let file_path = target_dir.join(&file.path);
            if let Some(parent) = file_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&file_path, file.content)?;
            println!("  {} {}", "✓".green(), file_path.display());
        }
    }

    println!("{} Code generated successfully!", "✓".green());
    Ok(())
}

fn parse_target(s: &str) -> Result<GenerationTarget> {
    match s.to_lowercase().as_str() {
        "rust" => Ok(GenerationTarget::Rust),
        "typescript" | "ts" => Ok(GenerationTarget::TypeScript),
        "python" | "py" => Ok(GenerationTarget::Python),
        "go" | "golang" => Ok(GenerationTarget::Go),
        "java" => Ok(GenerationTarget::Java),
        "kotlin" => Ok(GenerationTarget::Kotlin),
        "swift" => Ok(GenerationTarget::Swift),
        "csharp" | "cs" => Ok(GenerationTarget::CSharp),
        _ => anyhow::bail!("Unknown target: {}", s),
    }
}
