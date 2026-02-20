use unistructgen::core::CodeGenerator;
use unistructgen_macro::IntoIR;
use unistructgen::codegen::JsonSchemaRenderer;
use unistructgen::core::IRModule;

#[derive(IntoIR)]
struct User {
    #[field(doc = "The unique identifier for the user", min_value = 1)]
    id: i64,

    #[field(doc = "The user's full name", max_length = 100)]
    name: String,

    #[field(doc = "The user's email address", format = "email", optional)]
    email: Option<String>,

    #[field(doc = "List of tags associated with the user")]
    tags: Vec<String>,
    
    is_active: bool,
}

fn main() -> anyhow::Result<()> {
    // 1. Get the IR definition from the Rust struct
    let definition = <User as unistructgen::core::IntoIR>::ir_definition()
        .ok_or_else(|| anyhow::anyhow!("Failed to get IR definition"))?;

    // 2. Wrap it in a Module
    let mut module = IRModule::new("UserModule".to_string());
    module.add_type(definition);

    // 3. Render to JSON Schema
    let renderer = JsonSchemaRenderer::new();
    let schema = renderer.generate(&module)?;

    println!("Generated JSON Schema:\n");
    println!("{}", schema);

    // Verify it parses as JSON
    let _parsed: serde_json::Value = serde_json::from_str(&schema)?;
    println!("\nVerification: Valid JSON.");

    Ok(())
}
