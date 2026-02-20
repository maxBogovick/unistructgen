use unistructgen_macro::ai_tool;
use unistructgen::core::{ToolRegistry, Context, tools::ToolCall};
use unistructgen::llm::{LlmClientFactory, LlmClient};
use colored::*;

#[derive(Clone, Debug)]
struct DbPool {
    pub url: String,
}

/// A tool that requires a database pool from context.
#[ai_tool]
async fn get_user_balance(#[context] db: DbPool, user_id: i32) -> Result<f64, String> {
    println!("  -> [DB] Querying balance for user {} on {}", user_id, db.url);
    Ok(1250.50) 
}

/// A simple calculator tool.
#[ai_tool]
fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("{}", "=== UniStructGen Advanced Agency Demo ===".bright_green().bold());

    // 1. Setup Dependencies
    let mut context = Context::new();
    context.insert(DbPool { url: "bolt://localhost:7687".to_string() });

    // 2. Register Tools
    let mut registry = ToolRegistry::new();
    registry.register(GetUserBalanceTool);
    registry.register(AddTool);
    println!("Tools registered: {}", "get_user_balance, add".cyan());

    // 3. Demo: Batch Execution (Parallel)
    println!("\n{}", "Step 1: Parallel Batch Tool Execution...".yellow());
    let calls = vec![
        ToolCall { name: "get_user_balance".into(), arguments: r#"{"user_id": 1}"#.into() },
        ToolCall { name: "add".into(), arguments: r#"{"a": 10, "b": 20}"#.into() },
    ];

    let results = registry.execute_batch(calls, &context).await;
    for (name, res) in results {
        println!("Tool {}: {}", name.cyan(), format!("{:?}", res).green());
    }

    // 4. Demo: Streaming (Optional logic)
    println!("\n{}", "Step 2: Streaming Completion (Conceptual)...".yellow());
    println!("(Requires local Ollama or OpenAI key to run for real)");
    
    // Attempt to create client using Factory
    let client_result: unistructgen::llm::Result<Box<dyn LlmClient>> = LlmClientFactory::new().build();
    if let Ok(client) = client_result {
        println!("Client initialized: {}", client.model().cyan());
    } else {
        println!("Skipping real LLM call (no provider configured).");
    }

    println!("\n{}", "Advanced Agency Demo Finished!".bright_green());
    Ok(())
}