use anyhow::Result;
use unistructgen_core::{Context, ToolRegistry};
use unistructgen_macro::{ai_tool, generate_struct_from_json};

// 1) Компилятор сам генерирует структуру из JSON
generate_struct_from_json! {
    name = "User",
    json = r#"{"id": 1, "name": "Alice", "tags": ["admin", "rust"]}"#,
    serde = true
}

// 2) Превращаем функцию в AI‑tool одной аннотацией
#[ai_tool]
fn calculate_shipping(weight_kg: f64, destination: String) -> f64 {
    weight_kg * 2.5 + if destination == "international" { 15.0 } else { 5.0 }
}

#[tokio::main]
async fn main() -> Result<()> {
    // ✅ Структура уже существует
    let user = User {
        id: 42,
        name: "Maxim".to_string(),
        tags: vec!["founder".into(), "rust".into()],
    };
    println!("Generated struct: {:?}", user);

    // ✅ ToolRegistry сам экспортирует JSON Schema
    let mut registry = ToolRegistry::new();
    registry.register(CalculateShippingTool);

    let defs = registry.get_definitions();
    println!("Tool definitions: {}", serde_json::to_string_pretty(&defs)?);

    // ✅ Вызов инструмента — без ручного парсинга аргументов
    let context = Context::new();
    let result = registry
        .execute(
            "calculate_shipping",
            r#"{"weight_kg": 3.0, "destination": "domestic"}"#,
            &context,
        )
        .await?;

    println!("Tool result: {}", result);

    Ok(())
}
