# codemod_recipe

Deterministic codemods for Dart (and growing multi-language support) using
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
| `.codemod/recipes/` | Shipped YAML recipes (query DSL v2) |
| `.codemod/maps/` | Reusable string maps for `{{$map 'id' key}}` templates |
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
              (class_declaration
                name: (identifier) @className
                body: (class_body
                  (class_member
                    (method_signature
                      (function_signature
                        name: (identifier) @methodName))
                    (function_body
                      (block) @body)))
                (#eq? @className "{{className}}")
                (#eq? @methodName "{{methodName}}"))
            capture: body
            anchor: end
            text: "    print('codemod');\n"
postExecution:
  - dartFormat
```

### Template helpers

| Syntax | Meaning |
|--------|---------|
| `{{argName}}` | Replace with argument value |
| `{{$camel field}}` | camelCase of argument `field` |
| `{{$snake field}}` | snake_case |
| `{{$pascal field}}` | PascalCase |
| `{{$map 'mapId' keyArg}}` | Lookup in `.codemod/maps/` (key from arg value) |

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

**Dart** — `CodemodRecipe.compose(steps: [...])` merges recipes, inline
operations, and post-execution actions. The Rust YAML layer provides the same
semantics for `recipe:` steps and a `compose_recipe` API in `codemod_recipe_yaml`.

## Rust engine status

| Feature | Status |
|---------|--------|
| insert / replace / remove (tree-sitter Dart) | Done |
| Query file paths (`.scm`) | Done |
| Maps registry + `{{$map}}` | Done |
| Template casing helpers | Done |
| Host: preview / apply / diff / validate | Done |
| previewToken + patch selection | Done |
| Atomic multi-file apply | Done |
| MCP subprocess | Done |
| Recipe composition (`recipe:` steps) | Done |
| `generateAstPath` | Not planned (v1) |
| Create / template file steps | Not yet |
| TypeScript grammar | Planned |

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
- [docs/codemod-mcp.md](docs/codemod-mcp.md) — MCP tools and agent workflow
- [vscode_extension/README.md](vscode_extension/README.md) — extension setup
- [example/README.md](example/README.md) — Dart API examples

## License

BSD-3-Clause — see repository license file.
