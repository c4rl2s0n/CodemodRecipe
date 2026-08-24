# codemod_recipe

Deterministic codemods for Dart, Rust, Java, Kotlin, SQL/SQLite, and 300+ languages using
**sandboxed Luau recipes**, tree-sitter queries, and a VS Code extension for
preview/apply with selective patches.

## Quick start

### VS Code (recommended)

1. Build the extension (bundles the Rust `codemod_host` binary):

   ```bash
   cd vscode_extension && ./build.sh
   ```

2. Open a workspace with a `.codemod/` directory and install the VSIX.

3. Use the **Codemod Recipe** activity bar: pick a recipe, fill args, preview
   diffs (Files + Transcript), select patches, apply.

The extension talks to the **Rust host** over JSON stdio.

### Rust CLI / MCP

```bash
# Host (stdio JSON protocol)
cargo run -q --manifest-path rust/Cargo.toml -p codemod_recipe_host --bin codemod_host -- \
  --stdio-server --workspace-root . --codemod-root .codemod

# MCP server
cargo run -q --manifest-path rust/Cargo.toml -p codemod_recipe_host --bin codemod_mcp -- \
  --workspace-root . --codemod-root .codemod
```

See [docs/codemod-mcp.md](docs/codemod-mcp.md) for Cursor MCP setup.

## Project layout

| Path | Purpose |
|------|---------|
| `rust/` | Rust workspace: tree-sitter engine, Luau runtime, stdio host, MCP |
| `.codemod/recipes/` | Luau recipe modules (`return { id, apply, ... }`) |
| `.codemod/maps/` | Luau map modules (`id` + `map` table) |
| `.codemod/variables/` | Luau variable modules (`id` + `values` table) |
| `.codemod/queries/` | Luau query libraries (`id` + `queries` table) |
| `vscode_extension/` | VS Code / Codium extension |
| `test/fixtures/` | Integration and golden fixtures for the Rust engine |

## Luau recipe format

Recipes are Luau modules that `return` a table. **`apply(c)`** is the only mutation entrypoint:

```lua
return {
  id = "dart.logging.add_log_line",
  args = {
    { name = "file", required = true, inputKind = "file" },
    { name = "className", required = true, inputKind = "symbol" },
    { name = "methodName", required = true, inputKind = "symbol" },
  },
  postExecution = { "dart format ." },
  apply = function(c)
    c:insert({
      path = c.args.file,
      query = [[
        (class_definition
          name: (identifier) @className
          body: (class_body
            (method_signature
              (function_signature
                name: (identifier) @methodName))
            (function_body
              (block) @body))
          (#eq? @className "]] .. c.args.className .. [[")
          (#eq? @methodName "]] .. c.args.methodName .. [["))
      ]],
      capture = "body",
      anchor = "end",
      text = "    print('codemod');\n",
    })
  end,
}
```

`capture` selects which `@name` from the query to edit; `anchor` (`start` \| `end`)
places an insert before or after that span. Full walkthrough:
[docs/writing-recipes.md](docs/writing-recipes.md). Day-1 guide:
[docs/getting-started.md](docs/getting-started.md).

### Host API

| Method | Role |
|--------|------|
| `c.args` | Recipe argument map |
| `c:insert` / `c:replace` / `c:remove` | AST edits |
| `c:query` / `c:read` | Read-only inspection |
| `c:create` / `c:delete` | File create/delete |
| `c:run(id, args?)` | Invoke another recipe |
| `c:postExec` | Shell command (apply only) |
| `c:applyTemplate` / `c:renderFileTemplate` | MiniJinja |
| `c:map` / `c:var` / `c:queryLib` | Asset lookup |

Set `language = "dart"` (etc.) on edit specs for non-Dart or ambiguous paths.

### Template syntax (MiniJinja)

Templates are rendered via `c:applyTemplate` and `c:renderFileTemplate` — not YAML steps.
See [docs/recipe-templates.md](docs/recipe-templates.md).

### Preview vs apply

- Both execute `apply()`. Preview uses a **WorkingTree** and skips postExec (transcript only).
- Apply **re-runs** `apply()` and flushes before postExec / `postExecution`.
- `previewToken` = hash(recipe + args + starting file contents).

### Composition

```lua
apply = function(c)
  c:run("dart.settings.add_counter_field")
  c:run("dart.logging.add_log_line", { file = c.args.file })
end
```

**Inline recipes** — pass Luau source as `inlineRecipe` to the host/MCP (no file on disk).

### Centralized path resolution

File-backed resources use `rust/crates/core/src/resource_path.rs`:

- Workspace targets resolve under the workspace root.
- Recipe resources (`.scm`, templates, postExecution scripts) resolve recipe-local first, then `.codemod/`.
- Query libraries are Luau modules under `.codemod/queries/*.luau`.

## Rust engine status

| Feature | Status |
|---------|--------|
| Luau sandboxed recipes | Done |
| insert / replace / remove (tree-sitter, multi-language) | Done |
| Language registry (language-pack + sqlite native) | Done |
| Query file paths (`.scm`) and query libs | Done |
| Maps/variables registry + MiniJinja | Done |
| Host: preview / apply / diff / validate / transcript | Done |
| previewToken + patch selection + applyMayDiverge | Done |
| Atomic multi-file apply | Done |
| MCP subprocess | Done |
| `c:run` recipe composition | Done |
| `inlineRecipe` (Luau string) | Done |

Run tests:

```bash
cd rust && cargo test --all && cargo clippy --all-targets -- -D warnings
```

## Documentation

- [docs/getting-started.md](docs/getting-started.md) — **start here** (human walkthrough)
- [docs/README.md](docs/README.md) — documentation map / index
- [ARCHITECTURE.md](ARCHITECTURE.md) — design overview
- [docs/writing-recipes.md](docs/writing-recipes.md) — Luau recipe authoring
- [docs/new-project-rust-mcp.md](docs/new-project-rust-mcp.md) — Rust MCP quickstart
- [docs/tree-sitter-queries.md](docs/tree-sitter-queries.md) — tree-sitter queries
- [docs/language-support.md](docs/language-support.md) — multi-language support
- [docs/recipe-templates.md](docs/recipe-templates.md) — MiniJinja templates
- [docs/codemod-mcp.md](docs/codemod-mcp.md) — MCP tools and agent workflow

## License

BSD-3-Clause — see repository license file.
