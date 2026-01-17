# 🤝 Contributing to UniStructGen

Thank you for your interest in contributing to UniStructGen! We welcome contributions from everyone.

## 📋 Table of Contents

1. [Code of Conduct](#-code-of-conduct)
2. [Getting Started](#-getting-started)
3. [Development Setup](#-development-setup)
4. [Project Structure](#-project-structure)
5. [Making Changes](#-making-changes)
6. [Testing](#-testing)
7. [Pull Request Process](#-pull-request-process)
8. [Coding Standards](#-coding-standards)
9. [Release Process](#-release-process)

---

## 📜 Code of Conduct

This project follows the [Rust Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct). Please be respectful and constructive in your interactions.

---

## 🚀 Getting Started

### Ways to Contribute

- 🐛 **Report Bugs** - Found an issue? Let us know!
- 💡 **Suggest Features** - Have an idea? Share it!
- 📝 **Improve Documentation** - Help others understand
- 🔧 **Submit Code** - Fix bugs or add features
- 🧪 **Write Tests** - Help improve coverage
- 💬 **Help Others** - Answer questions in discussions

### Before You Start

1. Check existing [issues](https://github.com/yourusername/unistructgen/issues)
2. Check existing [pull requests](https://github.com/yourusername/unistructgen/pulls)
3. Join our [discussions](https://github.com/yourusername/unistructgen/discussions)

---

## 💻 Development Setup

### Prerequisites

- **Rust** 1.70 or later
- **Cargo** (comes with Rust)
- **Git**
- Optional: **rust-analyzer** for IDE support

### Clone and Build

```bash
# Clone the repository
git clone https://github.com/yourusername/unistructgen.git
cd unistructgen

# Build all workspace members
cargo build --workspace

# Run tests
cargo test --workspace

# Check formatting
cargo fmt -- --check

# Run clippy
cargo clippy --workspace -- -D warnings
```

### Verify Installation

```bash
# Run CLI
cargo run --bin unistructgen -- --version

# Run example
cargo run --example api-example

# Build documentation
cargo doc --workspace --no-deps --open
```

---

## 🏗️ Project Structure

```
unistructgen/
├── core/                   # Core IR types and traits
│   ├── src/
│   │   ├── ir.rs          # Intermediate representation
│   │   └── parser.rs      # Parser trait
│   └── Cargo.toml
│
├── parsers/
│   └── json_parser/       # JSON parser implementation
│       ├── src/
│       │   ├── lib.rs
│       │   ├── builder.rs
│       │   └── inference.rs
│       └── Cargo.toml
│
├── codegen/               # Rust code generation
│   ├── src/
│   │   ├── rust_renderer.rs
│   │   └── formatter.rs
│   └── Cargo.toml
│
├── proc-macro/            # Procedural macros
│   ├── src/
│   │   └── lib.rs        # Macro implementations
│   ├── tests/            # Integration tests
│   └── Cargo.toml
│
├── cli/                   # Command-line interface
│   ├── src/
│   │   ├── main.rs
│   │   └── commands/
│   └── Cargo.toml
│
├── examples/              # Example code and schemas
├── docs/                  # Additional documentation
└── Cargo.toml            # Workspace configuration
```

### Key Concepts

1. **IR (Intermediate Representation)**
   - Defined in `core/`
   - Language-agnostic representation of types
   - Used by all parsers and code generators

2. **Parser**
   - Converts input (JSON, etc.) to IR
   - Implements `Parser` trait from `core`
   - Lives in `parsers/`

3. **Code Generator**
   - Converts IR to output (Rust code)
   - Implements `Renderer` trait
   - Lives in `codegen/`

4. **Proc Macros**
   - Compile-time code generation
   - Uses parser + codegen pipeline
   - Lives in `proc-macro/`

---

## 🔧 Making Changes

### 1. Create a Branch

```bash
# Create a feature branch
git checkout -b feature/amazing-feature

# Or a bugfix branch
git checkout -b fix/issue-123
```

### 2. Make Your Changes

Follow these guidelines:

- Write clear, self-documenting code
- Add tests for new functionality
- Update documentation as needed
- Follow Rust idioms and best practices

### 3. Commit Your Changes

Use clear, descriptive commit messages:

```bash
git commit -m "feat: add support for OpenAPI schemas"
git commit -m "fix: handle empty arrays in JSON parser"
git commit -m "docs: update API integration guide"
git commit -m "test: add tests for nested object generation"
```

**Commit message format:**
- `feat:` - New feature
- `fix:` - Bug fix
- `docs:` - Documentation changes
- `test:` - Adding or updating tests
- `refactor:` - Code refactoring
- `perf:` - Performance improvements
- `chore:` - Maintenance tasks

---

## 🧪 Testing

### Run All Tests

```bash
# Run all tests
cargo test --workspace

# Run specific package tests
cargo test --package unistructgen-core
cargo test --package unistructgen-json-parser
cargo test --package unistructgen-macro

# Run with output
cargo test --workspace -- --nocapture
```

### Write Tests

#### Unit Tests

```rust
// In the same file as your code
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature() {
        let result = my_function();
        assert_eq!(result, expected);
    }
}
```

#### Integration Tests

```rust
// In tests/ directory
use unistructgen_macro::generate_struct_from_json;

#[test]
fn test_integration() {
    generate_struct_from_json! {
        name = "TestStruct",
        json = r#"{"field": "value"}"#
    }

    let instance = TestStruct {
        field: "test".to_string(),
    };

    assert_eq!(instance.field, "test");
}
```

### Test Coverage

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Run coverage
cargo tarpaulin --workspace --out Html
```

---

## 📤 Pull Request Process

### 1. Update Your Branch

```bash
# Fetch latest changes
git fetch origin
git rebase origin/master
```

### 2. Run Pre-Submission Checks

```bash
# Format code
cargo fmt --all

# Run clippy
cargo clippy --workspace --all-targets -- -D warnings

# Run tests
cargo test --workspace

# Build documentation
cargo doc --workspace --no-deps
```

### 3. Push Your Changes

```bash
git push origin feature/amazing-feature
```

### 4. Create Pull Request

1. Go to the [repository](https://github.com/yourusername/unistructgen)
2. Click "New Pull Request"
3. Select your branch
4. Fill in the PR template:

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
- [ ] All tests pass
- [ ] Added new tests
- [ ] Manual testing performed

## Checklist
- [ ] Code follows project style
- [ ] Self-review completed
- [ ] Documentation updated
- [ ] No new warnings
```

### 5. Code Review

- Respond to feedback promptly
- Make requested changes
- Push updates to your branch
- Request re-review when ready

### 6. Merge

Once approved:
- Squash commits if needed
- Merge via GitHub UI
- Delete your branch

---

## 📏 Coding Standards

### Rust Style

Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/):

```rust
// ✅ Good
pub fn parse_json(input: &str) -> Result<IRModule> {
    // Implementation
}

// ❌ Bad
pub fn ParseJSON(input: &str) -> IRModule {
    // Implementation
}
```

### Documentation

Every public item must have documentation:

```rust
/// Parses JSON input into an IR module
///
/// # Arguments
///
/// * `input` - JSON string to parse
///
/// # Returns
///
/// * `Ok(IRModule)` - Successfully parsed module
/// * `Err(JsonParserError)` - Parse error
///
/// # Examples
///
/// ```
/// use unistructgen_json_parser::JsonParser;
///
/// let json = r#"{"name": "Alice"}"#;
/// let result = parser.parse(json)?;
/// ```
pub fn parse(&mut self, input: &str) -> Result<IRModule> {
    // Implementation
}
```

### Error Handling

Use `thiserror` for custom errors:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("Invalid JSON at line {line}: {message}")]
    SyntaxError {
        line: usize,
        message: String,
    },

    #[error("Type inference failed: {0}")]
    TypeInferenceError(String),
}
```

### Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_json() {
        let json = r#"{"name": "Alice"}"#;
        let result = parse_json(json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_json() {
        let json = r#"{"invalid"#;
        let result = parse_json(json);
        assert!(result.is_err());
    }
}
```

---

## 🎯 Areas for Contribution

### High Priority

- [ ] OpenAPI/Swagger support
- [ ] GraphQL schema parsing
- [ ] Watch mode for file changes
- [ ] Validation derive generation
- [ ] Better error messages

### Medium Priority

- [ ] Markdown table parsing
- [ ] SQL DDL parsing
- [ ] Builder pattern generation
- [ ] TypeScript definitions export
- [ ] Performance optimizations

### Low Priority

- [ ] VSCode extension
- [ ] Web playground
- [ ] Plugin system
- [ ] Multiple language support

### Documentation

- [ ] More examples
- [ ] Video tutorials
- [ ] Blog posts
- [ ] API design guides

---

## 🚢 Release Process

### Versioning

We follow [Semantic Versioning](https://semver.org/):

- **MAJOR** - Breaking changes
- **MINOR** - New features (backwards compatible)
- **PATCH** - Bug fixes

### Release Checklist

1. **Update Version**
   ```toml
   # Cargo.toml
   [workspace.package]
   version = "0.2.0"
   ```

2. **Update Changelog**
   ```markdown
   ## [0.2.0] - 2024-01-15
   ### Added
   - Feature X
   ### Fixed
   - Bug Y
   ```

3. **Run Full Test Suite**
   ```bash
   cargo test --workspace --all-features
   cargo clippy --workspace -- -D warnings
   cargo fmt -- --check
   ```

4. **Build Documentation**
   ```bash
   cargo doc --workspace --no-deps
   ```

5. **Create Git Tag**
   ```bash
   git tag -a v0.2.0 -m "Release v0.2.0"
   git push origin v0.2.0
   ```

6. **Publish to crates.io**
   ```bash
   cargo publish -p unistructgen-core
   cargo publish -p unistructgen-json-parser
   cargo publish -p unistructgen-codegen
   cargo publish -p unistructgen-macro
   cargo publish -p unistructgen
   ```

---

## 🎓 Learning Resources

### Understanding the Codebase

1. Start with `core/src/ir.rs` - Understanding IR is key
2. Read `parsers/json_parser/src/lib.rs` - See how parsing works
3. Check `codegen/src/rust_renderer.rs` - See code generation
4. Explore `proc-macro/src/lib.rs` - See macro magic

### Rust Resources

- [The Rust Book](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [Proc Macro Workshop](https://github.com/dtolnay/proc-macro-workshop)
- [API Guidelines](https://rust-lang.github.io/api-guidelines/)

---

## 💬 Getting Help

- **Questions?** Use [GitHub Discussions](https://github.com/yourusername/unistructgen/discussions)
- **Bugs?** Open an [Issue](https://github.com/yourusername/unistructgen/issues)
- **Chat?** Join our Discord (coming soon)

---

## 🙏 Recognition

All contributors will be:
- Listed in `CONTRIBUTORS.md`
- Mentioned in release notes
- Credited in documentation

Thank you for making UniStructGen better! 🎉

---

<div align="center">

**[⬅️ Back to README](README.md)** • **[📖 Documentation](QUICKSTART.md)** • **[💬 Discussions](https://github.com/yourusername/unistructgen/discussions)**

Made with ❤️ by the community

</div>
