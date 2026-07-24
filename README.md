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

The extension talks to the **Rust host** over JSON stdio. Set
`codemodRecipe.useDartRun: true` only when debugging the legacy Dart host.

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

### Dart library (reference / legacy host)

```bash
dart pub get
dart run bin/codemod_host.dart --stdio-server --workspace-root . --codemod-root .codemod
```

The Dart implementation uses `package:analyzer` and the navigate/anchor DSL.
The Rust engine is the target for new development.

## Project layout

| Path | Purpose |
|------|---------|
| `lib/` | Dart package: analyzer transforms, YAML compiler, VS Code host |
| `rust/` | Rust workspace: tree-sitter engine, YAML model, stdio host, MCP |
| `.codemod/recipes/` | Shipped YAML recipes (query DSL v2; schema: `steps`) |
| `.codemod/maps/` | Recommended location for maps (`id` + `map:`) |
| `.codemod/variables/` | Recommended location for variables (`id` + `values:`) |
| `vscode_extension/` | VS Code / Codium extension |
| `test/fixtures/rust_oracle/` | Golden fixtures for the Rust engine |
| `example/` | Runnable Dart examples |

## YAML recipe format (v2 — Rust engine)

Recipes declare tree-sitter queries directly under `edit.ops`:

```yaml
dslVersion: 2
id: add_log_line
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
  - dartFormat
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
| `{{ key \| map('mapId') }}` | Lookup in workspace/recipe maps |
| `{{ map.mapId.key }}` | Nested map access |
| `{{ var.varId.key }}` | Shared variable (`id` + `values:`) |
| `{% if flag %}…{% endif %}` | Conditional (bool args: `"true"` / `"false"`) |

Legacy `{{$camel field}}` helpers still work but are deprecated.

See [docs/recipe-templates.md](docs/recipe-templates.md) for the full Jinja guide and
[`test/fixtures/jinja_examples/`](test/fixtures/jinja_examples/) for feature showcase recipes.

Query values can be inline (above) or a path to a `.scm` file relative to the
recipe or `.codemod/queries/`.

### Composing recipes

**YAML** — reference other recipes by id:

```yaml
steps:
  - recipe: add_counter_field
  - recipe: add_log_line
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

**Dart** — `CodemodRecipe.compose(steps: [...])` merges recipes, inline
operations, and post-execution actions. The Rust YAML layer provides the same
semantics for `recipe:` steps and a `compose_recipe` API in `codemod_recipe_yaml`.

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
| `generateAstPath` | Not planned (v1) |
| Legacy navigate+anchor DSL | Not planned |
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

- [ARCHITECTURE.md](ARCHITECTURE.md) — design decisions (Dart-centric; being updated)
- [docs/new-project-rust-mcp.md](docs/new-project-rust-mcp.md) — Rust MCP quickstart for new projects
- [docs/tree-sitter-queries.md](docs/tree-sitter-queries.md) — tree-sitter query language for recipes
- [docs/language-support.md](docs/language-support.md) — multi-language tree-sitter support
- [docs/codemod-mcp.md](docs/codemod-mcp.md) — MCP tools and agent workflow
- [vscode_extension/README.md](vscode_extension/README.md) — extension setup
- [example/README.md](example/README.md) — Dart API examples

## License

BSD-3-Clause — see repository license file.
