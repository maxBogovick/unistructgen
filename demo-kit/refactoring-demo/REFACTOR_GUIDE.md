# 🧪 Refactoring Session: The Power of Macros
**Concept:** "Delete code to write better code."
**Goal:** Show how `unistructgen_macro` replaces manual work with compile-time intelligence.

---

## Phase 1: The Setup (Show the Pain)

1.  Open `demo-kit/refactoring-demo/src/main.rs`.
2.  Scroll through the struct definitions (`User`, `Address`, `Geo`, `Company`).
3.  **Say:** "Look at this file. I want to use a simple user API. But to do that safely in Rust, I had to write **40 lines of struct definitions**."
4.  **Say:** "I had to guess types. Is `catch_phrase` optional? Is `lat` a string or float? I guessed. If I guessed wrong, the app crashes."

**(Action: Run the code to prove it works)**
`cargo run`

---

## Phase 2: The Refactor (The "Wow" Moment)

1.  **Say:** "Let's use UniStructGen macros. Watch closely."
2.  **Action:** DELETE all struct definitions (lines 10-42).
    *   Delete `struct User ...`
    *   Delete `struct Address ...`
    *   Delete `struct Geo ...`
    *   Delete `struct Company ...`
3.  **Say:** "I deleted everything. Now, let the compiler do the work."
4.  **Action:** Add the macro import at the top:
    ```rust
    use unistructgen_macro::struct_from_external_api;
    ```
5.  **Action:** Add the macro call where the structs used to be:
    ```rust
    struct_from_external_api! {
        struct_name = "User",
        url_api = "https://jsonplaceholder.typicode.com/users/1",
        serde = true
    }
    ```
    *(Tip: Have this snippet ready to copy-paste or use `main_refactored.rs` as a cheat sheet)*

---

## Phase 3: Verification (The Proof)

1.  **Say:** "That's it. This macro will connect to the API *during compilation*, analyze the JSON, and write the code for me."
2.  **Action:** Scroll down to `main()`.
3.  **Say:** "Look at the `main` function. I haven't changed a line. `user.address.city` still works."
4.  **Action:** Hover over `user.address` with your mouse.
5.  **Say:** "Look! The IDE knows the types. It knows `Address` exists. It knows `Geo` exists. But *I didn't write them*."

**(Action: Run the code)**
`cargo run`

---

## Phase 4: Conclusion

**Say:**
"We just replaced 40 lines of brittle, manual code with 5 lines of intelligent, auto-updating code.
If the API changes, I just recompile.

This is the power of UniStructGen macros."
