# Setup in Cursor

Create `.cursor/mcp.json` in the target workspace:

```json
{
  "mcpServers": {
    "codemod-mcp": {
      "command": "cargo",
      "args": [
        "run",
        "-q",
        "--manifest-path",
        "/absolute/path/to/codemod_recipe/rust/Cargo.toml",
        "-p",
        "codemod_recipe_host",
        "--bin",
        "codemod_mcp",
        "--",
        "--workspace-root",
        ".",
        "--codemod-root",
        ".codemod",
        "--sql-default",
        "sqlite"
      ]
    }
  }
}
```

Reload MCP servers and call `list_recipes` to verify connectivity.
