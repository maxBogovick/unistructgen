use serde_json::Value;
use std::fs;

fn main() {
    println!("--- Payment Processor v1.0 (Manual Mode) ---");

    // 1. Read raw data
    let data = fs::read_to_string("data/transaction.json").expect("Unable to read file");

    // 2. Parse as untyped "Value" map (The "Lazy" way)
    // We don't have structs yet, so we use a dynamic map.
    let v: Value = serde_json::from_str(&data).expect("JSON was not valid");

    // 3. THE PAIN: Manual field extraction
    // Look how unsafe and verbose this is. String keys everywhere.
    if let Some(tx_id) = v.get("transaction_id").and_then(|v| v.as_str()) {
        println!("Transaction ID: {}", tx_id);
        // Manual validation because we don't have types
        if tx_id.len() != 36 {
            println!("⚠️ Warning: ID doesn't look like a valid UUID");
        }
    }

    // Nested fields are a nightmare
    let payer_email = v.get("payer")
        .and_then(|p| p.get("email"))
        .and_then(|e| e.as_str())
        .unwrap_or("unknown");
    
    println!("Payer: {}", payer_email);

    // Date handling? It's just a string here.
    if let Some(date_str) = v.get("created_at").and_then(|d| d.as_str()) {
        println!("Created At: {} (Raw String)", date_str);
    }

    // Money math is risky without types
    if let Some(amount) = v.get("amount").and_then(|a| a.as_f64()) {
        let fee = amount * 0.029; // 2.9% fee
        println!("Fee calculated: ${:.2}", fee);
    }
}