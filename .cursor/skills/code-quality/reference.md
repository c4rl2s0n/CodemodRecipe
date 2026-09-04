# Code Quality Reference

Detailed standards for the code-quality skill. Read when running structure or
practices review.

## Rust crate ownership

| Crate | Owns | Must not |
|-------|------|----------|
| `core` | patches, atomic apply, `resource_path` | recipe DSL parsing, tree-sitter edits |
| `lua` | recipe module parse/validate, map/var/query-lib loaders | host I/O, MCP |
| `engine` | tree-sitter query ops | recipe authoring surface |
| `host` | registry, dispatch, MCP, bootstrap | ad-hoc path joins outside `resource_path` |

Workspace lints ([`rust/Cargo.toml`](../../../rust/Cargo.toml)):

- `unsafe_code = forbid`
- Clippy `all = warn`; `pedantic = allow` (do not enable pedantic globally unless asked)
- CI: `cargo clippy --all-targets -- -D warnings`

### Luau recipes

Prefer centralized owners:

- `rust/crates/lua/src/load.rs` — parse recipe/map/variable/query-lib tables
- `rust/crates/host/src/lua_runtime.rs` — sandboxed `apply(c, args)` host API
- `rust/crates/host/src/protocol_keys.rs` — host transport keys only

YAML DSL codegen (`scripts/generate-dsl-artifacts.sh`) is **deprecated**.

### Path / resource resolution

Use `rust/crates/core/src/resource_path.rs`:

- Workspace file targets resolve under the workspace root
- Recipe resources (`.scm`, templates, `postExecution`, extends/include) resolve
  recipe-local first, then `.codemod/`
- Bare query names also try `queries/` under each root
- Luau query libraries under `.codemod/queries/*.luau`

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

Structural completions come from `language/hostApiSurface.ts`, which is generated
from annotated Rust handler functions in `rust/crates/host/src/host_api/handlers.rs`
via `scripts/generate-host-api-surface.sh`. The spec structs and macros live in
`rust/crates/lua/src/host_api/specs.rs` and `rust/crates/lua_macros/`. Brace-aware
`c:method({…})` context lives in `luauContext.ts`. Recipe ids, languages,
maps/vars/queries, and `c:run` args come from the host catalog at runtime
(`RecipeRepository`).

Do **not** revive YAML DSL surface / `yamlContext` — recipes are Luau-only.

### Protocol sync

Changing host JSON shapes requires coordinated updates:

- Rust: `rust/crates/host/src/dispatch.rs`, `protocol.rs`
- TS: `vscode_extension/src/shared/messages.ts`, host protocol helpers
- Webview handlers that consume those messages

Rebuild via repo-root `./build.sh` (full product) or Vite for webview-only —
never patch `media/recipeView.*` directly. Extension packaging alone is
`vscode_extension/build.sh` when `bin/codemod_host` already exists.

## Practices checklist

### Structure

- [ ] Module/function has one clear responsibility
- [ ] No god files mixing I/O + parsing + policy without seams
- [ ] Dependencies point toward stable owners (`resource_path`, `luauContext`, host catalog)
- [ ] New types live in the owning crate, not duplicated across boundaries

### DRY / KISS

- [ ] Magic strings for protocol go through existing constant modules
- [ ] No copy-pasted path helpers
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
