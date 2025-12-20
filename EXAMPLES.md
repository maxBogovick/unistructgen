# UniStructGen Examples

This document contains practical examples for using UniStructGen in various scenarios.

## Table of Contents

1. [Basic JSON to Struct](#basic-json-to-struct)
2. [External API Examples](#external-api-examples)
3. [Complex Nested Structures](#complex-nested-structures)
4. [Real-World Use Cases](#real-world-use-cases)

## Basic JSON to Struct

### Simple User Model

\`\`\`rust
use unistructgen_macro::generate_struct_from_json;

generate_struct_from_json! {
    name = "User",
    json = r#"{
        "id": 1,
        "username": "alice",
        "email": "alice@example.com",
        "is_verified": true
    }"#
}

fn main() {
    let user = User {
        id: 42,
        username: "bob".to_string(),
        email: "bob@example.com".to_string(),
        is_verified: false,
    };
    println!("{:?}", user);
}
\`\`\`

## External API Examples

### GitHub API

\`\`\`rust
use unistructgen_macro::struct_from_external_api;

struct_from_external_api! {
    struct_name = "GitHubUser",
    url_api = "https://api.github.com/users/octocat"
}
\`\`\`

### JSONPlaceholder API

\`\`\`rust
struct_from_external_api! {
    struct_name = "Post",
    url_api = "https://jsonplaceholder.typicode.com/posts/1"
}
\`\`\`

See [GETTING_STARTED.md](GETTING_STARTED.md) for more detailed examples.
