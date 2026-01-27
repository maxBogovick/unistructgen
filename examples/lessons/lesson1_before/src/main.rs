use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::json;

// 1) Ручное описание структуры
#[derive(Debug, Clone, Serialize, Deserialize)]
struct User {
    #[serde(rename = "id")]
    id: i64,

    #[serde(rename = "name")]
    name: String,

    #[serde(rename = "tags")]
    tags: Vec<String>,
}

// 2) Ручная JSON Schema для инструмента (чтобы LLM понимал аргументы)
fn calculate_shipping_schema() -> serde_json::Value {
    json!({
        "type": "object",
        "properties": {
            "weight_kg": { "type": "number" },
            "destination": { "type": "string" }
        },
        "required": ["weight_kg", "destination"]
    })
}

// 3) Ручной парсинг аргументов инструмента
#[derive(Deserialize)]
struct CalculateShippingArgs {
    weight_kg: f64,
    destination: String,
}

fn calculate_shipping(weight_kg: f64, destination: &str) -> f64 {
    weight_kg * 2.5 + if destination == "international" { 15.0 } else { 5.0 }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Симулируем JSON, который пришел снаружи
    let raw = r#"{\"id\": 1, \"name\": \"Alice\", \"tags\": [\"admin\", \"rust\"]}"#;

    // Ручной парсинг
    let user: User = serde_json::from_str(raw)?;
    println!("Manual struct: {:?}", user);

    // Ручная JSON Schema
    let schema = calculate_shipping_schema();
    println!("Manual tool schema: {}", schema);

    // Ручная работа с аргументами инструмента
    let args_raw = r#"{\"weight_kg\": 3.0, \"destination\": \"domestic\"}"#;
    let args: CalculateShippingArgs = serde_json::from_str(args_raw)?;
    let result = calculate_shipping(args.weight_kg, &args.destination);
    println!("Manual tool result: {}", result);

    Ok(())
}
