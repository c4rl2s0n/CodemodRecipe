# Quick start

Build the Rust MCP server:

```bash
cd /path/to/codemod_recipe
cargo build -q --manifest-path rust/Cargo.toml -p codemod_recipe_host --bin codemod_mcp --release
```

Run the Rust MCP server:

```bash
cd /path/to/codemod_recipe
cargo run -q --manifest-path rust/Cargo.toml -p codemod_recipe_host --bin codemod_mcp -- \
  --workspace-root /absolute/path/to/target-workspace \
  --codemod-root .codemod
```

The server speaks MCP over stdio. It appears idle when healthy.
