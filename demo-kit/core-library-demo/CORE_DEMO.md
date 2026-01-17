# 🧠 Deep Dive: Building Custom Tools with Core
**Concept:** "Don't just use the tool. Build the tool."
**Target:** Senior Engineers, Platform Teams.

---

## Phase 1: The Problem (String Building)

1.  Open `demo-kit/core-library-demo/src/bin/01_spaghetti.rs`.
2.  **Narrator:** "Imagine you need to migrate 1000 legacy JSON configs to Rust. You write a script."
3.  **Action:** Run it. `cargo run --bin 01_spaghetti`.
4.  **Critique:**
    *   "It failed to create a struct for `database_config` (it just gave up and used `Value`)."
    *   "Field names are mixed case (`TIMEOUT_MS`)."
    *   "It's just string concatenation. You can't easily add features."

---

## Phase 2: The Solution (The Pipeline)

1.  Open `demo-kit/core-library-demo/src/bin/02_pipeline.rs`.
2.  **Narrator:** "Instead, we use `unistructgen-core` as a library. We build a **Pipeline**."
3.  **Visual:** Show the imports. `Pipeline`, `JsonParser`, `RustRenderer`.
4.  **Narrator:** "It's composed of three stages: Parse -> Transform -> Generate."

---

## Phase 3: The Power Move (Custom Transformer)

1.  **Narrator:** "Here is the killer feature. You can inject your own logic into the generation process."
2.  **Action:** Scroll to `struct EnterpriseStandard`.
3.  **Explain:**
    *   "We implemented `IRTransformer`."
    *   "We are iterating over the Abstract Syntax Tree (IR)."
    *   "We inject `PartialEq`, `Eq`."
    *   "We detect fields named 'timeout' and add critical documentation automatically."
4.  **Narrator:** "You can't do this with a regex script."

---

## Phase 4: Execution

1.  **Action:** Run it. `cargo run --bin 02_pipeline`.
2.  **Analyze Output:**
    *   "Look! `database_config` is now a proper struct `DatabaseConfig`."
    *   "Field names are fixed (`timeout_ms`)."
    *   "The documentation comment `Critical timeout value in ms` appeared!"
    *   "The struct has `#[derive(..., PartialEq, Eq)]`."

## Conclusion

**Narrator:**
"UniStructGen isn't just a CLI. It's a framework for code generation. Build your own platform on top of it."
