# Release Checklist for UniStructGen v0.1.0

This checklist is a verification list. Do not mark items as completed unless you have personally validated them in the current commit.

## ✅ Code Quality
- [ ] `cargo test --workspace --all-features`
- [ ] `cargo test --workspace --doc`
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [ ] `cargo fmt --all -- --check`
- [ ] `cargo doc --workspace --no-deps --all-features`

## 📦 Package Metadata (crates.io)
- [ ] `Cargo.toml` workspace metadata points to the correct repository/homepage/documentation
- [ ] License files present: `LICENSE-MIT` and `LICENSE-APACHE`
- [ ] Each crate uses workspace metadata where appropriate
- [ ] README reference in workspace is correct (`readme = "README.md"`)

## 📚 Documentation Consistency
- [ ] `README.md` is the single source of truth (others clearly marked archived)
- [ ] Examples compile and match current APIs
- [ ] CLI docs match real flags and behavior

## 🧪 Tests & Golden Files
- [ ] Golden tests cover Rust codegen and JSON Schema codegen
- [ ] Parser tests cover JSON/Markdown/OpenAPI basic and edge cases

## 🧭 Publishing Order (workspace)
Publish in dependency order:

1. `unistructgen-core`
2. `unistructgen-json-parser`
3. `unistructgen-openapi-parser`
4. `unistructgen-markdown-parser`
5. `unistructgen-sql-parser`
6. `unistructgen-graphql-parser`
7. `unistructgen-env-parser`
8. `unistructgen-codegen`
9. `unistructgen-macro`
10. `unistructgen-llm`
11. `unistructgen-mcp`
12. `unistructgen-agent`
13. `unistructgen` (CLI)

## 🚀 Tag + Release
- [ ] `git tag -a v0.1.0 -m "Release v0.1.0"`
- [ ] `git push origin v0.1.0`
- [ ] Create GitHub release notes from `CHANGELOG`/highlights

## 📣 Post-Release
- [ ] Validate install from crates.io (CLI + macro examples)
- [ ] Announce to relevant communities
- [ ] Monitor issues and triage feedback
