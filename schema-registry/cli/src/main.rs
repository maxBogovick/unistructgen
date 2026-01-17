//! Schema Registry CLI

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::*;
use schema_registry_common::*;
use std::path::PathBuf;

mod client;
mod commands;

use client::RegistryClient;

#[derive(Parser)]
#[command(name = "schema-registry")]
#[command(about = "Schema Registry CLI", long_about = None)]
#[command(version)]
struct Cli {
    /// Registry server URL
    #[arg(long, env = "SCHEMA_REGISTRY_URL", default_value = "http://localhost:3000")]
    server: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Upload a schema
    Upload {
        /// Schema name
        #[arg(short, long)]
        name: String,

        /// Schema version
        #[arg(short, long)]
        version: String,

        /// Schema file path
        #[arg(short, long)]
        file: PathBuf,

        /// Team name
        #[arg(short, long, default_value = "default")]
        team: String,

        /// Schema format
        #[arg(long, default_value = "openapi")]
        format: String,

        /// Description
        #[arg(short, long)]
        description: Option<String>,
    },

    /// List schemas
    List {
        /// Filter by team
        #[arg(short, long)]
        team: Option<String>,
    },

    /// Get schema details
    Get {
        /// Schema name
        name: String,

        /// Schema version (optional, uses latest if not specified)
        #[arg(short, long)]
        version: Option<String>,
    },

    /// List versions of a schema
    Versions {
        /// Schema name
        name: String,
    },

    /// Compare two versions
    Diff {
        /// Schema name
        name: String,

        /// From version
        #[arg(short, long)]
        from: String,

        /// To version
        #[arg(short, long)]
        to: String,
    },

    /// Generate code from schema
    Generate {
        /// Schema name
        name: String,

        /// Schema version
        #[arg(short, long)]
        version: String,

        /// Target languages (comma-separated: rust,typescript,python,go)
        #[arg(short, long)]
        targets: String,

        /// Output directory
        #[arg(short, long, default_value = "./generated")]
        output: PathBuf,
    },

    /// Show registry statistics
    Stats,

    /// Manage teams
    Team {
        #[command(subcommand)]
        command: TeamCommands,
    },
}

#[derive(Subcommand)]
enum TeamCommands {
    /// List all teams
    List,

    /// Create a new team
    Create {
        /// Team name
        name: String,

        /// Team description
        #[arg(short, long)]
        description: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let client = RegistryClient::new(&cli.server);

    match cli.command {
        Commands::Upload {
            name,
            version,
            file,
            team,
            format,
            description,
        } => commands::upload::upload(&client, name, version, file, team, format, description).await?,

        Commands::List { team } => commands::list::list_schemas(&client, team).await?,

        Commands::Get { name, version } => commands::get::get_schema(&client, name, version).await?,

        Commands::Versions { name } => commands::versions::list_versions(&client, name).await?,

        Commands::Diff { name, from, to } => commands::diff::diff_schemas(&client, name, from, to).await?,

        Commands::Generate {
            name,
            version,
            targets,
            output,
        } => commands::generate::generate(&client, name, version, targets, output).await?,

        Commands::Stats => commands::stats::show_stats(&client).await?,

        Commands::Team { command } => match command {
            TeamCommands::List => commands::team::list_teams(&client).await?,
            TeamCommands::Create { name, description } => {
                commands::team::create_team(&client, name, description).await?
            }
        },
    }

    Ok(())
}
