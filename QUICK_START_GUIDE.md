# Quick Start Guide - 5 Minutes to Your First Struct

> Short version. For the full guide, see `QUICKSTART.md`.

## 1. Installation (30 seconds)

```toml
# Add to Cargo.toml
[dependencies]
unistructgen-macro = "0.1"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

## 2. Your First Struct (2 minutes)

```rust
use unistructgen_macro::generate_struct_from_json;

// Generate from inline JSON
generate_struct_from_json! {
    name = "User",
    json = r#"{
        "id": 42,
        "name": "Alice",
        "email": "alice@example.com"
    }"#
}

fn main() {
    let user = User {
        id: 1,
        name: "Bob".to_string(),
        email: "bob@example.com".to_string(),
    };
    
    println!("User: {}", user.name);
}
```

Run it:
```bash
cargo run
```

## 3. From Real API (2 minutes)

```rust
use unistructgen_macro::struct_from_external_api;

// Fetch from real API at compile time!
struct_from_external_api! {
    struct_name = "GitHubUser",
    url_api = "https://api.github.com/users/octocat"
}

fn main() {
    // GitHubUser struct is now available with all API fields!
    println!("GitHub API struct generated!");
}
```

That's it! You now have type-safe structs from JSON. 🎉

## Next Steps

- Read [GETTING_STARTED.md](GETTING_STARTED.md) for detailed tutorial
- Check [EXAMPLES.md](EXAMPLES.md) for real-world examples
- Explore advanced features in [README.md](README.md)
