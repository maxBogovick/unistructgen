# 🚀 UniStructGen Real-World Showcase

This directory demonstrates the **true power** of UniStructGen by generating production-ready HTTP clients for real APIs.

## 🎯 What This Demonstrates

UniStructGen can generate **complete, type-safe, production-ready HTTP clients** from OpenAPI specifications in seconds:

- ✨ **Fully type-safe** - All types auto-generated from specs
- 🔒 **Built-in validation** - Request/response validation included
- 📝 **Auto-documented** - Complete documentation from OpenAPI
- 🚀 **Ready to use** - Just add to your project
- ⚡ **Fast** - Generate in seconds, not hours

## 📦 Included Examples

### 1. GitHub API Client
**The complete GitHub REST API client in one command!**

```bash
# Generate the entire GitHub API client
unistructgen client \
  --url https://raw.githubusercontent.com/github/rest-api-description/main/descriptions/api.github.com/api.github.com.json \
  --output ./github-client \
  --name "GitHub" \
  --examples true

# Use it immediately
cd github-client
cargo build
cargo run --example basic
```

**What you get:**
- 📊 Hundreds of type-safe structs
- 🔧 Ready-to-use HTTP client
- 📝 Full API documentation
- ✅ Request/response validation
- 💡 Usage examples

### 2. JSONPlaceholder Client
**Perfect for testing and demos**

```bash
# Generate a client for the test API
unistructgen client \
  --spec examples/realworld-clients/jsonplaceholder.yaml \
  --output ./jsonplaceholder-client \
  --name "JsonPlaceholder"
```

### 3. Stripe API Client (Coming Soon)
**Payment processing made type-safe**

### 4. OpenAI API Client (Coming Soon)
**AI APIs with full type safety**

## 🎨 How It Works

### Before UniStructGen (Manual Approach)
❌ Hours of manual coding
❌ Prone to typos and errors
❌ Hard to maintain
❌ No automatic updates
❌ Tedious testing

```rust
// You'd have to write ALL of this by hand:
pub struct Repository {
    pub id: u64,
    pub name: String,
    pub description: Option<String>,
    // ... 50 more fields
}

pub struct User {
    // ... another 30 fields
}

// ... hundreds more types
// ... all the HTTP client code
// ... all the error handling
// ... all the validation
```

### After UniStructGen (One Command)
✅ **5 seconds** to generate
✅ **100% accurate** from spec
✅ **Auto-maintained** - regenerate anytime
✅ **Auto-validated** - types enforce correctness
✅ **Production-ready** immediately

```bash
unistructgen client --spec api.yaml --output ./client
```

## 🌟 Real-World Use Cases

### Use Case 1: Internal API Client
**Problem:** Your backend team maintains an OpenAPI spec. Your frontend/services need a client.

**Solution:**
```bash
# Generate client whenever the spec changes
unistructgen client \
  --spec http://api.yourcompany.com/openapi.yaml \
  --output ./internal-api-client \
  --name "InternalAPI"

# Integrate into your project
# Add to Cargo.toml: internal-api-client = { path = "./internal-api-client" }
```

### Use Case 2: Third-Party API Integration
**Problem:** Need to integrate Stripe/GitHub/OpenAI but don't want to write boilerplate.

**Solution:**
```bash
# Generate from their public OpenAPI spec
unistructgen client \
  --url https://api.example.com/openapi.json \
  --output ./example-client

# Start using immediately - no boilerplate!
```

### Use Case 3: Microservices Communication
**Problem:** Multiple microservices need to talk to each other.

**Solution:**
```bash
# Each service exposes OpenAPI spec
# Generate type-safe clients for all services

unistructgen client --spec user-service/openapi.yaml --output clients/user-service
unistructgen client --spec payment-service/openapi.yaml --output clients/payment-service
unistructgen client --spec notification-service/openapi.yaml --output clients/notification-service
```

## 📈 Performance Comparison

| Method | Time | Lines of Code | Errors | Maintenance |
|--------|------|---------------|--------|-------------|
| **Manual** | 8-40 hours | 2000+ | High risk | Constant |
| **UniStructGen** | **5 seconds** | **1 command** | **Zero** | **Automated** |

## 🎓 Try It Yourself

### Quick Start

1. **Install UniStructGen**
```bash
cargo install --path ./cli
```

2. **Generate a client**
```bash
# Try with JSONPlaceholder (simple, fast)
unistructgen client \
  --spec examples/realworld-clients/jsonplaceholder.yaml \
  --output ./my-first-client \
  --name "MyApi"
```

3. **Use it**
```bash
cd my-first-client
cargo build
cargo run --example basic
```

4. **Integrate into your project**
```toml
# Add to your Cargo.toml
[dependencies]
my-first-client = { path = "./my-first-client" }
```

## 🔥 Advanced Features

### Custom Configuration
```bash
unistructgen client \
  --spec api.yaml \
  --output ./client \
  --name "MyAPI" \
  --examples true
```

### CI/CD Integration
```yaml
# .github/workflows/generate-client.yml
name: Update API Client
on:
  schedule:
    - cron: '0 0 * * *'  # Daily
  workflow_dispatch:

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
            --url ${{ secrets.API_SPEC_URL }} \
            --output ./api-client
      - name: Create PR
        # ... create PR with updated client
```

## 💡 Pro Tips

1. **Version your specs** - Keep OpenAPI specs in version control
2. **Automate generation** - Regenerate clients in CI/CD
3. **Share clients** - Publish generated clients as crates
4. **Test thoroughly** - Generated code includes validation

## 🤝 Contributing

Want to add more examples? PRs welcome!

Ideas for examples:
- Stripe API
- OpenAI/Anthropic API
- AWS APIs
- Google Cloud APIs
- Kubernetes API
- Your favorite API!

## 📚 Learn More

- [UniStructGen Documentation](../../README.md)
- [OpenAPI Specification](https://swagger.io/specification/)
- [Generated Code Examples](./examples/)

---

**Made with ❤️ using UniStructGen - Stop writing boilerplate, start building!**
