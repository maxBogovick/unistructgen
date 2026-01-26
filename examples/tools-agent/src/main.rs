use unistructgen_macro::ai_tool;
use unistructgen_core::{ToolRegistry, AiTool};
use colored::*;
use std::time::Duration;

/// Calculates the shipping cost based on weight and destination.
#[ai_tool]
fn calculate_shipping(weight_kg: f64, destination: String) -> f64 {
    println!("  -> Executing calculate_shipping(weight={}, dest={})", weight_kg, destination);
    
    let base_rate = match destination.to_lowercase().as_str() {
        "us" => 5.0,
        "eu" => 10.0,
        _ => 20.0,
    };
    
    weight_kg * base_rate
}

/// Gets the current weather for a city asynchronously.
#[ai_tool]
async fn get_weather(city: String) -> Result<String, String> {
    println!("  -> Executing async get_weather(city={})", city);
    // Simulate network delay
    tokio::time::sleep(Duration::from_millis(500)).await;
    
    if city.to_lowercase() == "unknown" {
        Err("City not found".to_string())
    } else if city.to_lowercase() == "london" {
        Ok("Rainy, 15°C".to_string())
    } else {
        Ok("Sunny, 25°C".to_string())
    }
}

#[tokio::main]
async fn main() {
    println!("{}", "=== UniStructGen AI Tools Agent Demo (Async) ===".bright_green().bold());

    // 1. Register Tools
    println!("\n{}", "Step 1: Registering AI Tools...".yellow());
    let mut registry = ToolRegistry::new();
    
    registry.register(CalculateShippingTool);
    registry.register(GetWeatherTool);
    
    println!("Tools registered: {}", "calculate_shipping, get_weather".cyan());

    // 2. Export Definitions (for LLM)
    println!("\n{}", "Step 2: Exporting Tool Definitions (JSON Schema)...".yellow());
    let definitions = registry.get_definitions();
    // Compact output for brevity
    println!("Definitions count: {}", definitions.len());

    // 3. Simulate LLM Tool Execution
    println!("\n{}", "Step 3: Simulating Async Execution...".yellow());
    
    // Scenario 1: Sync Tool (via Async Wrapper)
    println!("\nAgent: Calculate shipping to US.");
    let tool_name = "calculate_shipping";
    let args_json = r#"{ "weight_kg": 5.5, "destination": "US" }"#;
    
    println!("Calling tool: {} with args: {}", tool_name.cyan(), args_json.blue());
    match registry.execute(tool_name, args_json).await {
        Ok(result) => println!("Result: {}", result.green().bold()),
        Err(e) => println!("Error: {}", e.to_string().red()),
    }

    // Scenario 2: Async Tool with Result::Ok
    println!("\nAgent: Get weather in London.");
    let tool_name = "get_weather";
    let args_json = r#"{ "city": "London" }"#;
    
    println!("Calling tool: {} with args: {}", tool_name.cyan(), args_json.blue());
    match registry.execute(tool_name, args_json).await {
        Ok(result) => println!("Result: {}", result.green().bold()),
        Err(e) => println!("Error: {}", e.to_string().red()),
    }

    // Scenario 3: Async Tool with Result::Err
    println!("\nAgent: Get weather in Unknown city.");
    let tool_name = "get_weather";
    let args_json = r#"{ "city": "Unknown" }"#;
    
    println!("Calling tool: {} with args: {}", tool_name.cyan(), args_json.blue());
    match registry.execute(tool_name, args_json).await {
        // If the tool returns Err("City not found"), our macro converts it to Err(ToolError::ExecutionError)
        Ok(result) => println!("Result (Unexpected): {}", result.green().bold()),
        Err(e) => println!("Error (Expected): {}", e.to_string().red()),
    }
}