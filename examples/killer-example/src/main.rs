use anyhow::Result;
use serde_json::json;
use unistructgen::core::{Context, ToolRegistry};
use unistructgen_macro::{ai_tool, generate_struct_from_json};

// 1) Generate Rust types at compile time from JSON
// This is the "schema-as-example" path that teams use for real APIs.
generate_struct_from_json! {
    name = "PurchaseEvent",
    json = r#"{
        "event_id": "evt_123",
        "user_id": 42,
        "amount": 19.99,
        "currency": "USD",
        "items": ["book", "pen"],
        "source": "web"
    }"#,
    serde = true
}

/// Calculate tax for a purchase.
/// The macro turns this into an LLM tool with JSON Schema.
#[ai_tool]
fn calculate_tax(amount: f64, country: String) -> f64 {
    let rate = match country.as_str() {
        "US" => 0.07,
        "DE" => 0.19,
        "UK" => 0.20,
        _ => 0.0,
    };
    amount * rate
}

#[tokio::main]
async fn main() -> Result<()> {
    // 2) Use the generated type safely
    let event = PurchaseEvent {
        event_id: "evt_123".to_string(),
        user_id: 42,
        amount: 19.99,
        currency: "USD".to_string(),
        items: vec!["book".to_string(), "pen".to_string()],
        source: "web".to_string(),
    };

    let payload = serde_json::to_string_pretty(&event)?;
    println!("Event:\n{}\n", payload);

    // 3) Export tool definitions (JSON Schema) for LLMs
    let mut registry = ToolRegistry::new();
    registry.register(CalculateTaxTool);

    let definitions = registry.get_definitions();
    println!("Tool schema:\n{}\n", serde_json::to_string_pretty(&definitions)?);

    // 4) Execute a tool call (as if it came from an LLM)
    let context = Context::new();
    let args = json!({"amount": 100.0, "country": "US"}).to_string();
    let result = registry
        .execute("calculate_tax", &args, &context)
        .await?;
    println!("Tool result: {:?}", result);

    Ok(())
}
