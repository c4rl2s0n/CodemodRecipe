# codemod_recipe

Deterministic codemods for Dart, Rust, Java, Kotlin, SQL/SQLite, and 300+ languages using
declarative YAML recipes, tree-sitter queries, and a VS Code extension for
preview/apply with selective patches.

## Quick start

### VS Code (recommended)

1. Build the extension (bundles the Rust `codemod_host` binary):

   ```bash
   cd vscode_extension && ./build.sh
   ```

2. Open a workspace with a `.codemod/` directory and install the VSIX.

3. Use the **Codemod Recipe** activity bar: pick a recipe, fill args, preview
   diffs, select patches, apply.

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
| `rust/` | Rust workspace: tree-sitter engine, YAML model, stdio host, MCP |
| `.codemod/recipes/` | Shipped YAML recipes (schema: `id` + `steps`) |
| `.codemod/maps/` | Recommended location for maps (`id` + `map:`) |
| `.codemod/variables/` | Recommended location for variables (`id` + `values:`) |
| `vscode_extension/` | VS Code / Codium extension |
| `test/fixtures/` | Integration and golden fixtures for the Rust engine |

## YAML recipe format

Recipes declare tree-sitter queries directly under `edit.ops`:

```yaml
id: dart.logging.add_log_line
args:
  - name: file
    required: true
    inputKind: file
  - name: className
    required: true
    inputKind: symbol
  - name: methodName
    required: true
    inputKind: symbol
steps:
  - edit:
      path: "{{file}}"
      ops:
        - insert:
            query: |
              (class_definition
                name: (identifier) @className
                body: (class_body
                  (method_signature
                    (function_signature
                      name: (identifier) @methodName))
                  (function_body
                    (block) @body))
                (#eq? @className "{{className}}")
                (#eq? @methodName "{{methodName}}"))
            capture: body
            anchor: end
            text: "    print('codemod');\n"
postExecution:
  - "dart format ."
```

### Template syntax (Jinja2)

Recipes use [MiniJinja](https://docs.rs/minijinja/) (Jinja2-compatible). See
[docs/recipe-templates.md](docs/recipe-templates.md) for the full guide.

| Syntax | Meaning |
|--------|---------|
| `{{argName}}` | Replace with argument value |
| `{{ field \| camel_case }}` | camelCase |
| `{{ field \| snake_case }}` | snake_case |
| `{{ field \| pascal_case }}` | PascalCase |
| `{{ path \| parent }}` | Parent directory of a path arg |
| `{{ path \| basename }}` | Final path component |
| `{{ path \| stem }}` | Filename without extension |
| `{{ key \| map('mapId') }}` | Lookup in workspace/recipe maps |
| `{{ map.mapId.key }}` | Nested map access |
| `{{ var.varId.key }}` | Shared variable (`id` + `values:`) |
| `{% if flag %}…{% endif %}` | Conditional (bool args: `"true"` / `"false"`) |

Legacy `{{$camel field}}` helpers still work but are deprecated.

See [docs/recipe-templates.md](docs/recipe-templates.md) for the full Jinja guide and
[`test/fixtures/jinja_examples/`](test/fixtures/jinja_examples/) for feature showcase recipes.

Query values can be inline (above) or a path to a `.scm` file relative to the
recipe or `.codemod/queries/`.

### Centralized path resolution

File-backed recipe resources use the shared resolver in
`rust/crates/core/src/resource_path.rs`.

- Workspace mutation targets (`edit.path`, `create.path`, `delete.path`) are
  resolved exactly under the workspace root.
- Recipe resources (`query` `.scm` files, `create.templateFile`,
  `postExecution` scripts, and template `extends` / `include`) resolve
  relative to the referencing recipe first, then fall back to `.codemod/`.
- Bare query file names also check `queries/` under each root as a convention.
- YAML query libraries remain id-based under `.codemod/queries/*.yaml`; they
  are not loaded directly via `query: path/to/file.yaml`.

### Composing recipes

**YAML** — reference other recipes by id:

```yaml
steps:
  - recipe: dart.settings.add_counter_field
  - recipe: dart.logging.add_log_line
```

Referenced edit steps are inlined; args are merged (first definition wins).
Use top-level `postExecution` on the parent recipe.

**Delete files** — remove an existing file:

```yaml
steps:
  - delete:
      path: "{{legacyPath}}"
      ifMissing: skip   # fail | skip (default fail)
```

**Inline recipes** — pass a full recipe object to the host/MCP as `inlineRecipe` (no file on disk required).

Recipe composition (`recipe:` steps) is implemented in `codemod_recipe_yaml` (`compose.rs`).

## Rust engine status

| Feature | Status |
|---------|--------|
| insert / replace / remove (tree-sitter, multi-language) | Done |
| Language registry (language-pack + sqlite native) | Done |
| Query file paths (`.scm`) | Done |
| Maps registry + Jinja `map` filter | Done |
| Template engine (MiniJinja) + conditionals | Done |
| Template inheritance (`extends` / `include`) | Done |
| Comprehensive validate API | Done |
| Host: preview / apply / diff / validate | Done |
| previewToken + patch selection | Done |
| Atomic multi-file apply | Done |
| MCP subprocess | Done |
| Recipe composition (`recipe:` steps) | Done |
| Multi-file edit/create/delete in one recipe | Done |
| `create:` / `delete:` file steps | Done |
| `inlineRecipe` (host + MCP) | Done |
| Full MCP tool parity (minus `generate_astPath`) | Done |
| `generateAstPath` | Not planned |
| Legacy navigate+anchor DSL | Removed |
| TypeScript grammar | Planned |

### Multi-language support

Set `language:` on `edit` steps for non-Dart files. Queries must match that grammar’s node names. `.sql` defaults to `sqlite`; override with `language: sql` or `--sql-default`.

See [docs/language-support.md](docs/language-support.md) and agent skill `codemod-languages` (in `export/.agents/skills/`).

Run tests:

```bash
cd rust && cargo test --all && cargo clippy --all-targets -- -D warnings
```

Integration smoke (host protocol):

```bash
node vscode_extension/scripts/smoke.mjs
```

## Shipped recipes

| Recipe | Description |
|--------|-------------|
| `insert_log_line` | Insert log line in `Settings.update()` (fixed target) |
| `add_log_line` | Same as above with `className` / `methodName` args |
| `add_counter_field` | Insert field before a method |

## Documentation

- [docs/getting-started.md](docs/getting-started.md) — **start here** (human walkthrough)
- [docs/README.md](docs/README.md) — documentation map / index
- [ARCHITECTURE.md](ARCHITECTURE.md) — design overview
- [docs/new-project-rust-mcp.md](docs/new-project-rust-mcp.md) — Rust MCP quickstart for new projects
- [docs/tree-sitter-queries.md](docs/tree-sitter-queries.md) — tree-sitter query language for recipes
- [docs/language-support.md](docs/language-support.md) — multi-language tree-sitter support
- [docs/recipe-templates.md](docs/recipe-templates.md) — MiniJinja / Jinja2 templates
- [docs/recipe-design-patterns.md](docs/recipe-design-patterns.md) — create vs modify recipe taxonomy
- [docs/codemod-mcp.md](docs/codemod-mcp.md) — MCP tools and agent workflow
- [docs/recipe-shortcuts.md](docs/recipe-shortcuts.md) — VS Code shortcuts, slots, and `from` arg derivation
- [vscode_extension/README.md](vscode_extension/README.md) — extension setup

## License

BSD-3-Clause — see repository license file.
