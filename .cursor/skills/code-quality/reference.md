# Code Quality Reference

Detailed standards for the code-quality skill. Read when running structure or
practices review.

## Rust crate ownership

| Crate | Owns | Must not |
|-------|------|----------|
| `core` | patches, atomic apply, `resource_path` | recipe DSL parsing, tree-sitter edits |
| `yaml` | model, validate, `dsl::`, `dsl_structure`, vocabulary/codegen | host I/O, MCP |
| `engine` | tree-sitter query ops | YAML schema surface |
| `host` | registry, dispatch, MCP, bootstrap | ad-hoc path joins outside `resource_path` |

Workspace lints ([`rust/Cargo.toml`](../../../rust/Cargo.toml)):

- `unsafe_code = forbid`
- Clippy `all = warn`; `pedantic = allow` (do not enable pedantic globally unless asked)
- CI: `cargo clippy --all-targets -- -D warnings`

### DSL and vocabulary

Prefer centralized owners over string literals:

- `rust/crates/yaml/src/dsl/` — wire constants (`dsl::recipe::…`, maps, variables)
- `rust/crates/yaml/src/dsl_structure.rs` — container→child inventory (drives schema + surface)
- `rust/crates/yaml/src/model.rs` — runtime parse AST
- `rust/crates/yaml/src/dsl_vocabulary.rs` — author descriptions (`ENTRIES`)
- `rust/crates/host/src/protocol_keys.rs` — host transport keys only

After changing vocabulary, structure, or model fields:

```bash
scripts/generate-dsl-artifacts.sh
```

Commit refreshed `vscode_extension/schemas/*`, `generated-dsl-surface.json`,
`generated-keyword-docs.json`, and TextMate lists. CI fails if generated files drift.

### Path / resource resolution

Use `rust/crates/core/src/resource_path.rs`:

- Workspace file targets resolve under the workspace root
- Recipe resources (`.scm`, templates, `postExecution`, extends/include) resolve
  recipe-local first, then `.codemod/`
- Bare query names also try `queries/` under each root
- YAML query libraries are id-based under `.codemod/queries/*.yaml`

Do not add new ad-hoc `canonicalize`, `join("..")`, or absolute-path checks in
individual crates.

### Clippy allows

Prefer fixing the warning. If an allow is necessary, match existing style:

```rust
#[allow(clippy::too_many_arguments)] // preview/apply signature mirrors host protocol
```

One-line reason required. Avoid blanket file-level allows.

### Rust style notes

- Prefer small modules and clear `Result` / existing error types (`thiserror` where used)
- No `unsafe`
- `cargo fmt` before submit
- Behavioral changes need tests under the owning crate (`cargo test --all`)

## TypeScript / Vue

Layout:

| Area | Path |
|------|------|
| Extension backend | `vscode_extension/src/extension/` |
| Shared protocol types | `vscode_extension/src/shared/messages.ts` |
| Webview UI | `vscode_extension/src/webview/src/` |
| Built artifacts | `vscode_extension/media/` (do not edit by hand) |

Gates:

```bash
cd vscode_extension && npx tsc -p ./ --noEmit
cd vscode_extension && npm run test:webview   # when webview or shared/ changed
```

There is no ESLint in this repo; rely on `tsc`, Vitest, and pattern conformity.

### Language toolkit

Structural completions come from `schemas/generated-dsl-surface.json` via
`dslSurface.ts` / `yamlContext.ts`. Recipe ids, languages, and `with:` args come
from the host catalog at runtime.

Do **not** hardcode DSL container children in TypeScript completion tables.

### Protocol sync

Changing host JSON shapes requires coordinated updates:

- Rust: `rust/crates/host/src/dispatch.rs`, `protocol.rs`
- TS: `vscode_extension/src/shared/messages.ts`, host protocol helpers
- Webview handlers that consume those messages

Rebuild webview via `vscode_extension/build.sh` or Vite — never patch
`media/recipeView.*` directly.

## Practices checklist

### Structure

- [ ] Module/function has one clear responsibility
- [ ] No god files mixing I/O + parsing + policy without seams
- [ ] Dependencies point toward stable owners (`dsl::`, `resource_path`, `yamlContext`)
- [ ] New types live in the owning crate, not duplicated across boundaries

### DRY / KISS

- [ ] Magic strings for DSL/protocol go through existing constant modules
- [ ] No copy-pasted path or YAML-indent helpers
- [ ] No speculative generics/traits “for later”
- [ ] Names match neighboring code in the same crate/folder

### Errors and tests

- [ ] Error messages are actionable (what failed, which path/id)
- [ ] New behavior has unit or integration coverage
- [ ] Tests are deterministic (no reliance on wall-clock or ambient cwd without setup)

### Anti-patterns

- Ad-hoc recipe keyword strings outside `yaml` DSL modules
- Path traversal checks reimplemented in `host` / `engine`
- Hardcoded completion child lists in the extension
- Editing generated schemas or `media/` without regenerating
- Broad `#[allow(clippy::all)]` or silencing `-D warnings` locally

## Report template

```markdown
## Quality pass report

### Commands
- cargo fmt …
- cargo clippy …
- tsc …
- (optional) npm run test:webview …

### Fixed
- …

### Must-fix
- …

### Suggestions
- …
```
