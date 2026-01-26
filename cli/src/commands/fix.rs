use anyhow::{Result, Context};
use colored::*; 
use inquire::Confirm;
use std::fs;
use std::path::Path;
use unistructgen_core::diagnostics::CargoDiagnostics;
use unistructgen_core::patch::CodeFix;
use unistructgen_llm::{LlmClient, CompletionRequest, Message};
use unistructgen_llm::ollama::OllamaClient;
use unistructgen_llm::openai::OpenAiClient;
use serde_json::json;

pub async fn run_fix() -> Result<()> {
    println!("{}", "Running cargo check...".yellow());
    
    let errors = CargoDiagnostics::check(Path::new("."))
        .context("Failed to run diagnostics")?;

    if errors.is_empty() {
        println!("{}", "No errors found! Great job.".green());
        return Ok(());
    }

    println!("Found {} errors. Analyzing...", errors.len().to_string().red());

    // Initialize LLM (Prefer OpenAI for coding, fallback to Ollama)
    let client: Box<dyn LlmClient> = if std::env::var("OPENAI_API_KEY").is_ok() {
        println!("{}", "Using OpenAI (gpt-4o)...".cyan());
        Box::new(OpenAiClient::new("gpt-4o")?)
    } else {
        println!("{}", "Using Ollama (llama3)...".cyan());
        Box::new(OllamaClient::new("llama3"))
    };

    for error in errors {
        println!("\n{}", "---------------------------------------------------".dimmed());
        println!("{}", error.message.red().bold());
        
        let span = match error.primary_span {
            Some(s) => s,
            None => {
                println!("No source location for this error. Skipping.");
                continue;
            }
        };

        println!("File: {}:{}:{}", span.file_name, span.line_start, span.column_start);

        // Read file context
        let file_path = Path::new(&span.file_name);
        let file_content = fs::read_to_string(file_path)
            .context("Failed to read file")?;
        
        let lines: Vec<&str> = file_content.lines().collect();
        
        // Extract context (error line +/- 10 lines)
        let start_line = span.line_start.saturating_sub(10).max(1) - 1; // 0-indexed
        let end_line = (span.line_end + 10).min(lines.len());
        
        let context_snippet = lines[start_line..end_line].join("\n");

        // Prepare Prompt
        let system_prompt = "You are a Rust expert. Fix the compilation error. Return a JSON object describing the fix. The `original_snippet` in hunks must MATCH the existing code EXACTLY (including whitespace) for the patch to apply.";
        
        let user_prompt = format!(
            "Error: {}\n\nCode context (lines {}-{}):\n```rust\n{}\n```\n\nFile: {}",
            error.message, start_line + 1, end_line, context_snippet, span.file_name
        );

        // Manual Schema for CodeFix
        let schema = json!({
            "type": "object",
            "properties": {
                "file_path": { "type": "string" },
                "explanation": { "type": "string" },
                "changes": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "original_snippet": { "type": "string" },
                            "new_snippet": { "type": "string" }
                        },
                        "required": ["original_snippet", "new_snippet"]
                    }
                }
            },
            "required": ["file_path", "explanation", "changes"],
            "additionalProperties": false
        });

        println!("{}", "Asking AI for a fix...".yellow().blink());

        let request = CompletionRequest {
            messages: vec![
                Message::system(system_prompt),
                Message::user(user_prompt),
            ],
            response_schema: Some(schema),
            ..Default::default()
        };

        let response: unistructgen_llm::Result<String> = client.complete(request).await;
        
        match response {
            Ok(response_text) => {
                // Parse response
                // response_text is String
                let fix: CodeFix = serde_json::from_str(&response_text)
                    .context("Failed to parse AI response")?;

                println!("\nAI Suggestion: {}", fix.explanation.cyan());
                
                // Show Diff
                println!("{}", "Changes:".bold());
                for hunk in &fix.changes {
                    println!("{}", "--- Original ---".red());
                    println!("{}", hunk.original_snippet.red());
                    println!("{}", "+++ New ---".green());
                    println!("{}", hunk.new_snippet.green());
                }

                // Apply?
                let ans = Confirm::new("Apply this fix?")
                    .with_default(false)
                    .prompt();

                match ans {
                    Ok(true) => {
                        match fix.apply(&file_content) {
                            Ok(new_content) => {
                                fs::write(file_path, new_content)?;
                                println!("{}", "Fix applied!".green());
                            }
                            Err(e) => {
                                println!("{}", format!("Failed to apply patch: {}", e).red());
                            }
                        }
                    }
                    _ => println!("Skipped."),
                }
            }
            Err(e) => {
                println!("{}", format!("AI Request failed: {}", e).red());
            }
        }
    }

    Ok(())
}