# 🎉 UniStructGen v0.1 - Complete Feature Demonstration

## Three Powerful Ways to Generate Rust Structs

### 1️⃣ Inline JSON (Proc-Macro)

\`\`\`rust
use unistructgen_macro::generate_struct_from_json;

generate_struct_from_json! {
    name = "User",
    json = r#"{"id": 1, "name": "Alice"}"#
}
\`\`\`

### 2️⃣ External API (Compile-Time HTTP) 🆕

\`\`\`rust
use unistructgen_macro::struct_from_external_api;

struct_from_external_api! {
    struct_name = "User",
    url_api = "https://jsonplaceholder.typicode.com/users/1"
}
\`\`\`

### 3️⃣ CLI (File-Based)

\`\`\`bash
unistructgen generate --input schema.json --name User
\`\`\`

## Live Demo Results

Run the API example to see it in action!
