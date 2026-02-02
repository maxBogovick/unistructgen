# Killer Example: Types + Tools in 60 Seconds

This example shows the core value of UniStructGen in one small program:

- Generate Rust types from JSON at compile time
- Expose Rust functions as LLM tools with JSON Schema
- Execute tool calls safely

## Run

```bash
cargo run -p killer-example
```

## What you will see

1) A generated `PurchaseEvent` struct serialized as JSON
2) The tool schema for `calculate_tax`
3) A tool execution result

## Why this matters

This is the shortest path from messy data to:

- **Type-safe Rust**
- **LLM‑friendly JSON Schema**
- **Reliable tool calls**

Everything is generated from a single source of truth.
