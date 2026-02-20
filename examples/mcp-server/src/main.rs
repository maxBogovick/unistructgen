use unistructgen_macro::ai_tool;
use unistructgen::core::{ToolRegistry, Context};
use unistructgen::mcp::serve_stdio;
use std::sync::Arc;

/// Echo the input back
#[ai_tool]
fn echo(message: String) -> String {
    format!("Echo: {}", message)
}

/// Calculate the sum of two numbers
#[ai_tool]
fn add(a: i64, b: i64) -> i64 {
    a + b
}

/// Get current weather (mock)
#[ai_tool]
fn get_weather(city: String) -> String {
    format!("The weather in {} is sunny and 25C", city)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Configure tracing to write to stderr, as stdout is used for MCP protocol
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .init();

    let mut registry = ToolRegistry::new();
    registry.register(EchoTool);
    registry.register(AddTool);
    registry.register(GetWeatherTool);

    let context = Context::new();

    eprintln!("Starting MCP server on stdio...");
    serve_stdio(Arc::new(registry), context).await?;

    Ok(())
}
