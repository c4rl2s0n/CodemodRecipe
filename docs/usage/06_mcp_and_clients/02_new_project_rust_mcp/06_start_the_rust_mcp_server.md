# Start the Rust MCP server

```bash
cargo run -q --manifest-path rust/Cargo.toml -p codemod_recipe_host --bin codemod_mcp -- \
  --workspace-root /absolute/path/to/your/target-workspace \
  --codemod-root .codemod
```
