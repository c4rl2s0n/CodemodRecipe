# Dogfood MCP

```bash
cargo run -q --manifest-path rust/Cargo.toml -p codemod_recipe_host --bin codemod_mcp -- \
  --workspace-root test/fixtures/flutter_poc/workspace \
  --codemod-root test/fixtures/flutter_poc/.codemod
```
