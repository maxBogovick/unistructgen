# 🎓 Master Class: From JSON to Production Rust
**Project Context:** "Payment Processor Service"
**Goal:** Refactor "Spaghetti Code" using UniStructGen.

---

## 0. Setup

1.  Open `demo-kit/01-intro/payment-processor`.
2.  Make sure `src/main.rs` contains the "Manual Mode" code (using `serde_json::Value`).
3.  Terminal is open.

---

## 1. The "Before" State (The Problem)

**Narrator:**
"We have a legacy payment processor here. It reads a JSON transaction, but look at the code."

**(Action: Run `cargo run`. Show output.)**

```text
--- Payment Processor v1.0 (Manual Mode) ---
Transaction ID: a0eebc99-9c0b...
Payer: customer@example.com
Created At: 2024-01-15T14:30:00Z (Raw String)
Fee calculated: $72.50
```

**Narrator:**
"It works, but the code is terrible (`src/main.rs`). 
1. We are using string keys (`"transaction_id"`) - prone to typos.
2. We manually validate UUID length.
3. Dates are just strings.
4. Nested fields require chains of `.get().and_then()`. 
   
Let's fix this mess in 30 seconds using UniStructGen."

---

## 2. Generating the Solution

**Narrator:**
"First, I generate the types directly from the data."

**(Action: Run command)**

```bash
unistructgen generate \
    --input data/transaction.json \
    --output src/models.rs \
    --name Transaction \
    --serde \
    --optional
```

**(Action: Quickly open `src/models.rs`)**

**Narrator:**
"Look at this. Strict types. UUIDs. DateTimes. No work required."

---

## 3. The Refactor (Live Coding)

**Narrator:**
"Now, let's delete that spaghetti code in `main.rs` and use the power of Rust."

**(Action: Edit `src/main.rs`. Delete almost everything inside `main` and type this:)**

```rust
mod models;
use models::Transaction;
use std::fs;

fn main() {
    println!("---" Payment Processor v2.0 (UniStructGen) ---");

    // 1. Load Data
    let data = fs::read_to_string("data/transaction.json").expect("Unable to read file");

    // 2. Strong Typing (One line!)
    let tx: Transaction = serde_json::from_str(&data).expect("Invalid data");

    // 3. Clean Access
    println!("Transaction ID: {}", tx.transaction_id); // It's a real UUID type! 
    
    // Look mom, no .get("payer") chains!
    if let Some(payer) = &tx.payer {
        println!("Payer: {}", payer.email);
    }

    // Working with real Dates
    println!("Created At: {}", tx.created_at.format("%Y-%m-%d %H:%M"));

    // Safe Math
    if let Some(amount) = tx.amount {
         let fee = amount * 0.029;
         println!("Fee calculated: ${:.2}", fee);
    }
}
```

---

## 4. The Result

**(Action: Run `cargo run`)**

**Narrator:**
"Same output, but the code is safe, clean, and maintainable. And we did it by typing almost nothing. 

Download UniStructGen today."

```
