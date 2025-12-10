use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;
use unistructgen_codegen::{RenderOptions, RustRenderer};
use unistructgen_json_parser::{JsonParser, ParserOptions};

#[derive(Parser)]
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
}

fn main() -> Result<()> {
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
            anyhow::bail!("Markdown parsing is not yet implemented");
        }
        "sql" => {
            anyhow::bail!("SQL parsing is not yet implemented");
        }
        other => {
            anyhow::bail!("Unsupported format: {}", other);
        }
    };

    // Generate Rust code
    let renderer = RustRenderer::new(RenderOptions::default());
    let generated_code = renderer
        .render(&ir_module)
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
