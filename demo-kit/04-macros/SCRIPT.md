# 🎬 Script: Video 4 - Compile-Time Magic

**Duration:** ~1 minute
**Goal:** Show the power of Rust macros fetching data.

---

### 0:00 - The Concept
**(Screen: `demo_macro.rs` in VS Code)**
**You:** "What if you could generate types from a live API response *during compilation*?"

### 0:15 - The Code
**(Action: Hover over `struct_from_external_api!`)**
**You:** "I'm using the `struct_from_external_api!` macro. I point it to a JSON placeholder URL."

### 0:30 - The Compilation
**(Action: Run `cargo run`)**
**You:** "I haven't defined `struct Todo` anywhere. The macro fetched the JSON, inferred the schema, and generated the code before `main` ran."

### 0:45 - Breaking it
**(Action: Change the URL to a different endpoint, e.g., `/users/1`, and hit save/check)**
**You:** "If I change the endpoint to a User object... boom! The compiler errors immediately because the fields `title` and `completed` don't exist on a User. Type safety from an external API."
