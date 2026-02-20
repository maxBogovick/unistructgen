use unistructgen::core::api::{StructGen, FieldType};
use unistructgen::codegen::JsonSchemaRenderer;
use unistructgen::core::{CodeGenerator, ValidationReport, map_serde_error};
use unistructgen::parsers::markdown::chunker::{SemanticChunker, SplitterOptions};
use colored::*;
use serde::{Deserialize, Serialize};
use std::fs;

/// This is the struct we want our AI to return.
/// We define it once in Rust, and UniStructGen handles the rest.
#[derive(Debug, Serialize, Deserialize, Clone)]
struct AgentResponse {
    pub answer: String,
    pub confidence: f64,
    pub sources: Vec<String>,
    pub requires_action: bool,
}

fn main() -> anyhow::Result<()> {
    println!("{}", "=== UniStructGen AI Docu-Agent Demo ===".bright_green().bold());

    // --- STEP 1: RAG INGESTION ---
    println!("\n{}", "Step 1: Ingesting Documentation (RAG)...".yellow());
    let readme_content = fs::read_to_string("../../../README_OLD.md")
        .unwrap_or_else(|_| "# Welcome\nThis is a fallback README content.".to_string());
    
    let chunker = SemanticChunker::new(SplitterOptions::default());
    let chunks = chunker.split(&readme_content);
    println!("Successfully parsed README into {} semantic chunks.", chunks.len().to_string().green());
    
    // Show a sample chunk
    if let Some(sample) = chunks.get(1) {
        println!("Sample chunk path: {:?}", sample.header_path);
    }

    // --- STEP 2: DATA CONTRACT (JSON SCHEMA) ---
    println!("\n{}", "Step 2: Generating Data Contract for AI...".yellow());
    
    // We can generate the IR programmatically or from existing types
    // Here we'll use the StructGen API to show how to build it
    let agent_ir = StructGen::new()
        .name("AgentResponse")
        .doc("Structured response from the documentation agent")
        .field("answer", FieldType::String)
        .field("confidence", FieldType::F64)
        .field("sources", FieldType::vec(FieldType::String))
        .field("requires_action", FieldType::Bool)
        .build_ir_module();

    let schema_renderer = JsonSchemaRenderer::new();
    let json_schema = schema_renderer.generate(&agent_ir)?;
    
    println!("Generated JSON Schema (what we send to OpenAI/Claude):");
    println!("{}", "-------------------------------------------".dimmed());
    println!("{}", json_schema.dimmed());
    println!("{}", "-------------------------------------------".dimmed());

    // --- STEP 3: THE AI LOOP (with simulated errors) ---
    println!("\n{}", "Step 3: Running Agent Query Loop...".yellow());
    
    let query = "How do I use UniStructGen with external APIs?";
    println!("Query: '{}'", query.bright_blue());

    // Find relevant chunks (mock search)
    let context = chunks.iter()
        .find(|c| c.content.contains("API"))
        .map(|c| c.content.as_str())
        .unwrap_or("No specific API section found.");

    let mut attempts = 0;
    let mut current_json_input = simulate_ai_response(context, true); // First attempt is BROKEN

    loop {
        attempts += 1;
        println!("\nAttempt {}: Received response from AI...", attempts);
        println!("Raw JSON: {}", current_json_input.cyan());

        // Try to parse the response
        match serde_json::from_str::<AgentResponse>(&current_json_input) {
            Ok(response) => {
                println!("\n{}", "✅ SUCCESS: Validated AI Response!".bright_green().bold());
                println!("{:#?}", response);
                break;
            }
            Err(e) => {
                println!("\n{}", "❌ VALIDATION ERROR!".bright_red().bold());
                
                // Use UniStructGen Validation Engine
                let err_details = map_serde_error(&e);
                let mut report = ValidationReport::new();
                report.add_error(err_details);
                
                let correction_prompt = report.to_correction_prompt();
                println!("{}", "Sending correction instructions to AI:".bright_yellow());
                println!("{}", correction_prompt.italic().dimmed());

                // Simulate AI fixing itself in the next attempt
                current_json_input = simulate_ai_response(context, false); 
                
                if attempts > 3 {
                    println!("Too many attempts. Stopping.");
                    break;
                }
            }
        }
    }

    println!("\n{}", "Demo finished successfully!".bright_green());
    Ok(())
}

/// Simulates an AI response.
/// If `broken` is true, it returns invalid data (string where number expected).
fn simulate_ai_response(context: &str, broken: bool) -> String {
    if broken {
        // ERROR: confidence is a string "0.95" instead of a number 0.95
        format!(r#"{{
            "answer": "You can use the `struct_from_external_api!` macro as described in the context: {}",
            "confidence": "high",
            "sources": ["README.md > Real-World Examples"],
            "requires_action": false
        }}"#, &context[..50])
    } else {
        // VALID response
        format!(r#"{{
            "answer": "To use UniStructGen with external APIs, use the `struct_from_external_api!` macro. Provide the `struct_name` and `url_api`.",
            "confidence": 0.98,
            "sources": ["README.md > Example 1: API Client Development"],
            "requires_action": false
        }}"#)
    }
}
