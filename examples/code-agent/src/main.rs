mod sandbox;
mod compiler;

use colored::*;
use std::thread;

use sandbox::{RustSandbox, extract_rust_code};
use compiler::Compiler;
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    println!("{}", "=== UniStructGen Compiler-Driven Development (CDDL) Agent ===".bright_green().bold());

    // 1. Initialize Sandbox
    println!("\n{}", "Step 1: Initializing Ephemeral Sandbox...".yellow());
    let sandbox = RustSandbox::new()?;
    
    // 2. Define the task
    let task = "Write a Rust function `validate_email(email: &str) -> bool` using regex. Do NOT add main function.";
    println!("Task: {}", task.bright_blue());

    // 3. The Loop
    let mut current_code = mock_ai_generate_code(task, 0); // Start with buggy code
    let mut attempts = 0;
    let max_attempts = 5;

    loop {
        attempts += 1;
        println!("\n{}", format!("--- Attempt {} ---", attempts).dimmed());
        
        // Extract code
        let code_block = extract_rust_code(&current_code).unwrap_or(current_code.clone());
        println!("AI wrote code ({} chars)", code_block.len());

        // Write to sandbox
        sandbox.write_code(&code_block)?;
        println!("Code content:\n{}", code_block.dimmed());

        // Compile
        println!("{}", "Compiling...".yellow());
        let errors = Compiler::check(sandbox.path())?;

        if errors.is_empty() {
            println!("\n{}", "✅ SUCCESS: Code compiled successfully!".bright_green().bold());
            println!("{}", "Final Code:".dimmed());
            println!("{}", code_block);
            break;
        } else {
            println!("\n{}", "❌ COMPILATION FAILED".bright_red().bold());
            
            // Generate Feedback
            println!("Compiler reported {} errors:", errors.len());
            let mut feedback = String::new();
            feedback.push_str("The code failed to compile. Please fix the following errors:\n");
            
            for (i, err) in errors.iter().enumerate() {
                // We only take the first 2 errors to avoid overwhelming the AI
                if i >= 2 { break; }
                
                println!("{}. {}", i+1, err.message);
                // Also print rendered for user to see
                println!("{}", err.rendered.dimmed());
                
                feedback.push_str(&format!("Error {}: {}\n", i+1, err.message));
                if let Some(loc) = &err.location {
                    feedback.push_str(&format!("Location: {}\n", loc));
                }
            }

            if attempts >= max_attempts {
                println!("Max attempts reached. Giving up.");
                break;
            }

            println!("\n{}", "Sending feedback to AI...".bright_yellow());
            // Simulate AI fixing code based on feedback
            current_code = mock_ai_generate_code(&feedback, attempts);
            
            // Small pause for dramatic effect
            thread::sleep(Duration::from_millis(500));
        }
    }

    Ok(())
}

/// Mock AI that improves code based on iteration count.
/// Real implementation would call OpenAI with the feedback prompt.
fn mock_ai_generate_code(_prompt: &str, attempt: usize) -> String {
    if attempt == 0 {
        // Attempt 1: Buggy code (missing imports, wrong return type maybe)
        r#" 
        // I forgot to import Regex!
        pub fn validate_email(email: &str) -> bool {
            let re = Regex::new(r"^[\w-\.]+@([\w-]+\.)+[\w-]{2,4}$").unwrap();
            re.is_match(email)
        }
        "#.to_string()
    } else if attempt == 1 {
        // Attempt 2: Still buggy (Regex imported, but unwrap() is bad practice? No, let's say typo)
        r#" 
        use regex::Regex;

        pub fn validate_email(email: &str) -> bool {
            // Typo in variable name
            let re = Regex::new(r"^[\w-\.]+@([\w-]+\.)+[\w-]{2,4}$").unwrap();
            ree.is_match(email) 
        }
        "#.to_string()
    } else {
        // Attempt 3: Fixed
        r#" 
        use regex::Regex;

        pub fn validate_email(email: &str) -> bool {
            let re = Regex::new(r"^[\w-\.]+@([\w-]+\.)+[\w-]{2,4}$").unwrap();
            re.is_match(email)
        }
        "#.to_string()
    }
}
