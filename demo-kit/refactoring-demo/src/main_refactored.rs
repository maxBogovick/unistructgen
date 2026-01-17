// ✅ THE "AFTER" STATE: Macro Magic
// Clean, type-safe, and auto-updating.

use unistructgen_macro::struct_from_external_api;
use serde::{Deserialize, Serialize};

// ✨ MAGIC HAPPENS HERE
// This macro fetches the JSON at compile time,
// detects schema, nested objects (Address, Geo, Company),
// and generates strict Rust types for you.
struct_from_external_api! {
    struct_name = "User",
    url_api = "https://jsonplaceholder.typicode.com/users/1",
    serde = true
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Fetching User (UniStructGen Mode) ---");

    // We still fetch data at runtime normally...
    let url = "https://jsonplaceholder.typicode.com/users/1";
    let resp = reqwest::blocking::get(url)?.text()?;
    
    // But now we use the AUTO-GENERATED struct 'User'!
    // No manual definitions above.
    let user: User = serde_json::from_str(&resp)?;

    println!("User: {} ({})", user.name, user.email);
    
    // The macro correctly inferred nested structs!
    // Notice how IDE autocompletes '.address' and '.city'
    println!("City: {}", user.address.city);
    
    println!("Company: {}", user.company.name);

    Ok(())
}
