# 🚀 Getting Started with UniStructGen - For Developers

**Stop writing boilerplate. Start building.**

UniStructGen generates type-safe Rust code from APIs, JSON, and schemas, with a focus on compile-time safety and a clean dev workflow.

## 🎯 Why UniStructGen?

### The Problem

You want to integrate an API. You have an OpenAPI spec. Now you face:

- ❌ **Hours of tedious typing** - Writing hundreds of structs manually
- ❌ **Error-prone** - Typos, missing fields, wrong types
- ❌ **Hard to maintain** - API changes = rewrite everything
- ❌ **No validation** - Runtime errors instead of compile-time safety
- ❌ **Boring work** - You'd rather be building features

### The UniStructGen Solution

```bash
# One command. Generate a typed client scaffold from OpenAPI.
unistructgen client --spec api.yaml --output ./client
```

- ✅ **Instant generation** - 5 seconds vs 8+ hours of manual work
- ✅ **Spec-driven** - Generated directly from spec
- ✅ **Type-safe** - Compiler catches errors before runtime
- ✅ **Auto-validated** - Built-in constraints where available
- ✅ **Scaffold-ready** - Use immediately or extend as needed
- ✅ **Always up-to-date** - Regenerate when API changes

## ⚡ Quick Start (60 Seconds)

> Note: OpenAPI client generation is still evolving. It produces a strong typed scaffold, but you should review and extend it for your API’s edge cases.

### 1. Install

```bash
# From crates.io
cargo install unistructgen

# Or from source
git clone https://github.com/maxBogovick/unistructgen
cd unistructgen
cargo install --path ./cli
```

### 2. Generate Your First Client

```bash
# Try with a real API (JSONPlaceholder)
unistructgen client \
  --spec examples/realworld-clients/jsonplaceholder.yaml \
  --output ./my-api-client \
  --name "MyApi"
```

**What just happened?**
- Generated type-safe Rust types
- Created a client scaffold with async/await helpers
- Added validation from OpenAPI constraints where possible
- Included usage examples
- Created `Cargo.toml` with dependencies

### 3. Use It

```bash
cd my-api-client
cargo run --example basic
```

```rust
// Your generated client - ready to use!
use my_api_client::{Client, types::*};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::with_base_url("https://jsonplaceholder.typicode.com");

    // All types are generated and validated!
    // let posts = client.get_posts().await?;
    // let user = client.get_user(1).await?;

    Ok(())
}
```

## 🔥 Real-World Examples

### Example 1: GitHub API Client

Generate a complete GitHub API client:

```bash
unistructgen client \
  --url https://raw.githubusercontent.com/github/rest-api-description/main/descriptions/api.github.com/api.github.com.json \
  --output ./github-client \
  --name "GitHub"
```

**Result:** Type-safe client for the entire GitHub REST API in 5 seconds!

### Example 2: Internal Microservice

Your team has an OpenAPI spec for your internal API:

```bash
# Generate client from your API spec
unistructgen client \
  --url http://internal-api.company.com/openapi.yaml \
  --output ./internal-client

# Add to your service
cd my-service
# Add this to Cargo.toml:
# internal-client = { path = "../internal-client" }
```

Now you have:
- ✅ Type-safe communication between services
- ✅ Compile-time API contract verification
- ✅ Auto-generated when API changes
- ✅ No manual sync needed

### Example 3: Quick JSON to Rust Structs

Got a JSON response? Need Rust structs?

```bash
# From a JSON file
unistructgen generate \
  --input response.json \
  --name User \
  --output user.rs

# Or download first, then generate
curl -o response.json https://api.github.com/users/octocat
unistructgen generate --input response.json --name GitHubUser
```

## 💡 Use Cases

### 1. **Microservices Communication**

```bash
# Service A exposes OpenAPI spec
# Service B needs to call Service A

# Generate type-safe client for Service B
unistructgen client --spec service-a/openapi.yaml --output clients/service-a

# In Service B's code:
# use service_a_client::Client;
# let client = Client::new(config);
# let result = client.do_something().await?;
```

**Benefits:**
- Services stay in sync automatically
- Breaking changes caught at compile time
- No manual client maintenance

### 2. **Third-Party API Integration**

```bash
# Want to use Stripe API?
unistructgen client \
  --url https://raw.githubusercontent.com/stripe/openapi/master/openapi/spec3.yaml \
  --output ./stripe-client \
  --name "Stripe"
```

**Result:** Production-ready Stripe client without writing a single line!

### 3. **Rapid Prototyping**

```bash
# Got an API idea? Start with the spec
unistructgen client --spec my-api-idea.yaml --output ./client

# Client is ready! Start building features, not boilerplate
```

### 4. **Documentation as Code**

```bash
# Your OpenAPI spec IS your client
# Change spec → Regenerate → Done

# In CI/CD:
- name: Update API Client
  run: |
    unistructgen client --spec api.yaml --output ./client
    git diff  # See what changed
```

## 📚 All Features

### Generate from Multiple Sources

```bash
# From OpenAPI file
unistructgen client --spec openapi.yaml

# From OpenAPI URL
unistructgen client --url https://api.example.com/openapi.json

# From JSON (quick structs)
unistructgen generate --input data.json --name MyStruct

# Future: From SQL DDL, Protobuf, GraphQL...
```

### Customization Options

```bash
unistructgen client \
  --spec api.yaml \
  --output ./client \
  --name "MyAPI"
```

### What You Get

Every generated client includes:

```
generated-client/
├── Cargo.toml      # Ready-to-use dependencies
├── lib.rs          # Public API
├── types.rs        # All generated types
├── client.rs       # HTTP client with methods
├── README.md       # Documentation
└── examples/
    └── basic.rs    # Usage examples
```

## 🎓 How It Works

```
OpenAPI Spec → Parser → IR → Code Generator → Rust Code
     ↓            ↓       ↓         ↓              ↓
  api.yaml    Validated  Types   Templates    types.rs
                                              client.rs
```

1. **Parse** - Read OpenAPI/JSON/SQL
2. **Validate** - Ensure correctness
3. **Transform** - Convert to Intermediate Representation
4. **Generate** - Produce idiomatic Rust code
5. **Done** - Production-ready code

## 📊 Comparison

| Task | Manual | UniStructGen |
|------|--------|--------------|
| Generate types | 4-8 hours | **5 seconds** |
| Add validation | 2-4 hours | **Included** |
| Write HTTP client | 4-8 hours | **Included** |
| Documentation | 2-4 hours | **Auto-generated** |
| Maintenance | **Ongoing** | **Regenerate** |
| **Total** | **12-24 hours** | **5 seconds** |

## 🚀 Advanced Usage

### Proc Macro (Compile-Time Generation)

```rust
// In your Rust code - types generated at compile time!
use unistructgen_macro::openapi_to_rust;

openapi_to_rust! {
    file = "api.yaml"
}

// Use generated types immediately
fn main() {
    let user = User {
        id: 1,
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    };

    user.validate().unwrap();  // Built-in validation!
}
```

### CI/CD Integration

```yaml
# .github/workflows/update-client.yml
name: Update API Client

on:
  schedule:
    - cron: '0 0 * * *'  # Daily

jobs:
  update:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install UniStructGen
        run: cargo install unistructgen

      - name: Generate Client
        run: |
          unistructgen client \
            --spec api.yaml \
            --output ./client

      - name: Test
        run: cd client && cargo test

      - name: Create PR
        if: ${{ github.event_name == 'schedule' }}
        uses: peter-evans/create-pull-request@v5
        with:
          title: "chore: Update API client"
          body: "Auto-generated client update"
```

### Multiple Clients

```bash
# Generate clients for all your APIs
for spec in specs/*.yaml; do
  name=$(basename $spec .yaml)
  unistructgen client --spec $spec --output "clients/$name"
done
```

## 🤝 Contributing

We welcome contributions! Ideas:

- Add new parsers (GraphQL, Protobuf, SQL DDL)
- Improve code generation
- Add more real-world examples
- Better documentation

See [CONTRIBUTING.md](CONTRIBUTING.md)

## 📖 Learn More

- [Full Documentation](README.md)
- [Real-World Examples](./examples/realworld-clients/SHOWCASE.md)
- [API Reference](./docs/)
- [OpenAPI Guide](./OPENAPI_GUIDE.md)

## 💬 Community & Support

- GitHub Issues: Report bugs, request features
- Discussions: Ask questions, share examples
- Twitter: [@unistructgen](https://twitter.com/unistructgen) (example)

## 📜 License

MIT OR Apache-2.0

---

## 🎉 Ready to Save Hours of Work?

```bash
# Install
git clone https://github.com/maxBogovick/unistructgen
cd unistructgen
cargo install --path ./cli

# Generate
unistructgen client --spec your-api.yaml --output ./client

# Build
cd client && cargo build

# Ship! 🚀
```

**Stop writing boilerplate. Start building features.**

---

Made with ❤️ by developers who hate boilerplate

⭐ Star us on GitHub if UniStructGen saved you time!
