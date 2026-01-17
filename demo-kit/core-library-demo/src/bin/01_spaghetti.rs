// ❌ THE BAD WAY: String Concatenation
// "Quick script" that becomes a maintenance nightmare.

use std::fs;
use serde_json::Value;

fn main() {
    println!("--- Generating Code (The Spaghetti Way) ---");
    
    let data = fs::read_to_string("data/legacy.json").unwrap();
    let v: Value = serde_json::from_str(&data).unwrap();

    // Look at this mess. We are manually building strings.
    // No AST. No safety. No formatting.
    let mut code = String::from("pub struct Config {\n");
    
    if let Value::Object(map) = v {
        for (key, value) in map {
            let type_str = match value {
                Value::String(_) => "String",
                Value::Number(n) if n.is_i64() => "i64",
                Value::Bool(_) => "bool",
                Value::Object(_) => "serde_json::Value", // GAVE UP ON NESTING!
                _ => "String", // Catch-all guess
            };
            
            // Snake case conversion manually? Forgot it.
            // Documentation? None.
            code.push_str(&format!("    pub {}: {},\n", key, type_str));
        }
    }
    code.push_str("}\n");

    println!("{}", code);
}
