use anyhow::{Context, Result};
use clap::{Parser as ClapParser, Subcommand};
use std::fs;
use std::path::PathBuf;
use unistructgen::core::{Parser, CodeGenerator};
use unistructgen::codegen::{RenderOptions, RustRenderer};
use unistructgen::parsers::json::{JsonParser, ParserOptions};
use unistructgen::parsers::markdown::{MarkdownParser, MarkdownParserOptions};
use unistructgen::parsers::sql::{SqlParser, SqlParserOptions};

mod client_gen;
mod commands;

use commands::fix::run_fix;

#[derive(ClapParser)]
#[command(name = "unistructgen")]
#[command(about = "Generate Rust structs from JSON, Markdown, or SQL", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate Rust code from input
    Generate {
        /// Input file path (JSON, MD, or SQL)
        #[arg(short, long)]
        input: PathBuf,

        /// Output file path (defaults to stdout if not provided)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Format of the input file (json, markdown, sql)
        /// Auto-detected from extension if not provided
        #[arg(short, long)]
        format: Option<String>,

        /// Name of the root struct to generate
        #[arg(short, long, default_value = "Root")]
        name: String,

        /// Add serde derives (Serialize, Deserialize)
        #[arg(long, default_value = "true")]
        serde: bool,

        /// Add Default derive
        #[arg(long, default_value = "false")]
        default: bool,

        /// Make all fields optional (wrapped in Option<T>)
        #[arg(long, default_value = "false")]
        optional: bool,
    },

    /// Generate a complete HTTP client from OpenAPI specification
    Client {
        /// Path to OpenAPI specification file (YAML or JSON)
        #[arg(short, long)]
        spec: Option<PathBuf>,

        /// URL to fetch OpenAPI specification from
        #[arg(short, long)]
        url: Option<String>,

        /// Output directory for generated client
        #[arg(short, long, default_value = "./generated-client")]
        output: PathBuf,

        /// Name of the client (e.g., "GitHub", "Stripe")
        #[arg(short, long, default_value = "Api")]
        name: String,

        /// Generate usage examples
        #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
        examples: bool,
    },

    /// (Experimental) Attempt to fix compilation errors using AI
    Fix,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Generate {
            input,
            output,
            format,
            name,
            serde,
            default: default_derive,
            optional,
        } => {
            generate_code(input, output, format, name, serde, default_derive, optional)?;
        }
        Commands::Client {
            spec,
            url,
            output,
            name,
            examples,
        } => {
            let generator = client_gen::ClientGenerator {
                spec_path: spec,
                spec_url: url,
                output_dir: output,
                client_name: name,
                include_examples: examples,
            };
            generator.generate()?;
        }
        Commands::Fix => {
            run_fix().await?;
        }
    }

    Ok(())
}

fn generate_code(
    input: PathBuf,
    output: Option<PathBuf>,
    format: Option<String>,
    name: String,
    derive_serde: bool,
    derive_default: bool,
    make_fields_optional: bool,
) -> Result<()> {
    // Read input file
    let input_content = fs::read_to_string(&input)
        .with_context(|| format!("Failed to read input file: {}", input.display()))?;

    // Determine format
    let detected_format = format.unwrap_or_else(|| {
        input
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase())
            .unwrap_or_else(|| "json".to_string())
    });

    // Parse input based on format
    let ir_module = match detected_format.as_str() {
        "json" => {
            let parser_options = ParserOptions {
                struct_name: name.clone(),
                derive_serde,
                derive_default,
                make_fields_optional,
            };

            let mut parser = JsonParser::new(parser_options);
            parser
                .parse(&input_content)
                .context("Failed to parse JSON input")?
        }
        "md" | "markdown" => {
            let parser_options = MarkdownParserOptions {
                struct_name: name.clone(),
                derive_serde,
                derive_default,
                make_fields_optional,
            };
            
            let mut parser = MarkdownParser::new(parser_options);
            parser
                .parse(&input_content)
                .context("Failed to parse Markdown input")?
        }
        "sql" => {
            let parser_options = SqlParserOptions {
                derive_serde,
                derive_default,
                make_fields_optional,
            };

            let mut parser = SqlParser::new(parser_options);
            parser
                .parse(&input_content)
                .context("Failed to parse SQL input")?
        }
        other => {
            anyhow::bail!("Unsupported format: {}", other);
        }
    };

    // Generate Rust code using CodeGenerator trait
    let generator = RustRenderer::new(RenderOptions::default());
    let generated_code = generator
        .generate(&ir_module)
        .context("Failed to generate Rust code")?;

    // Output
    match output {
        Some(output_path) => {
            // Write to file
            if let Some(parent) = output_path.parent() {
                fs::create_dir_all(parent).with_context(|| {
                    format!("Failed to create output directory: {}", parent.display())
                })?;
            }

            fs::write(&output_path, generated_code).with_context(|| {
                format!("Failed to write output file: {}", output_path.display())
            })?;

            println!("✓ Generated code written to: {}", output_path.display());
        }
        None => {
            // Write to stdout
            println!("{}", generated_code);
        }
    }

    Ok(())
}
