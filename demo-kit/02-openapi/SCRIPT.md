# 🎬 Script: Video 2 - Production-Ready API Clients

**Duration:** ~2 minutes
**Goal:** Show type safety and ease of use.

---

### 0:00 - Introduction
**(Screen: Show `petstore.yaml`)**
**You:** "Consuming APIs in Rust often means fighting with `reqwest` and defining types manually. Let's convert this OpenAPI spec into a full Rust library."

### 0:20 - Generation
**(Screen: Terminal)**
**You:** "One command."
**(Action: Run command)**
```bash
unistructgen client --spec petstore.yaml --name PetStore --output ./my-client
```

### 0:40 - Exploring the Code
**(Screen: Open `my-client/src/client.rs` and `types.rs`)**
**You:** "It generated a complete crate. We have:
- Async client.
- Validated types.
- Request builders."

### 1:00 - The "Magic" (IDE Autocomplete)
**(Screen: Open `my-client/examples/main.rs`)**
**You:** "Watch the developer experience."
**(Action: Start typing code to use the client. Highlight how IDE suggests `.get_pets()` and shows the types)**

```rust
let client = Client::new(config);
// Show autocomplete here!
let pets = client.get_pets().await?;
```

**You:** "If the API changes, just regenerate. Your compiler will catch any breaking changes instantly."
