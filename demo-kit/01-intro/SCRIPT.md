# 🎬 Script: Video 1 - Stop Writing Structs Manually

**Duration:** ~45-60 seconds
**Goal:** Hook the viewer immediately.

---

### 0:00 - The Problem
**(Screen: Show `user_data.json` open in IDE)**

**You:** "Stop doing this."
**(Action: Create a new file `user.rs` and start typing `pub struct User { ... }` slowly, making a typo)**
**You:** "Manually rewriting JSON fields into Rust structs is slow, boring, and error-prone. What if you have nested objects? Dates? UUIDs?"

### 0:15 - The Solution
**(Screen: Switch to Terminal, clear screen)**

**You:** "Use UniStructGen."
**(Action: Paste and run the command)**
```bash
unistructgen generate -i user_data.json -n User --serde
```

### 0:25 - The Result
**(Screen: Open the output file or show stdout)**

**You:** "Done. In less than a second."
**(Action: Highlight specific lines with mouse)**
**You:** "Look at this:
1. It detected `uuid::Uuid` automatically.
2. It detected `DateTime` automatically.
3. It created nested structs for `Settings` and `Notifications`.
4. It added `serde` derives."

### 0:45 - Call to Action
**(Screen: GitHub Repository or Cargo Install command)**

**You:** "Stop wasting time on boilerplate. Install UniStructGen today."
```bash
cargo install unistructgen
```
