---
name: codemod-recipe-vscode-extension-backend
description: Use when implementing or debugging the VS Code extension backend (TypeScript). Covers activation, Rust host bridge, Recipe Language Toolkit, recipe discovery/refresh, and the webview view provider message loop.
disable-model-invocation: true
---

# VS Code Extension Backend (TypeScript)

## When to use

Use this skill when you need to:
- Debug webview interactions (messages from UI not producing expected state)
- Change preview/diff/apply behavior
- Understand how the persistent **Rust** host is started, queued, and parsed
- Change editor language features (completions, CodeLens, diagnostics publishing)
- Update protocol glue after modifying the host or shared types

## Where to look (entry points)

### Extension activation + commands
- `vscode_extension/src/extension/extension.ts`
  - `activate(...)`: config, `HostBridge`, `RecipeRepository`, `LanguageSession`, diagnostics, watcher, commands
  - Recreates `LanguageSession` when `codemodRoot` / `workspaceRoot` change

### Recipe language support (Luau)

- Selectors: `.codemod/**/*.{luau,lua}` only (no YAML)
- `language/session.ts` — registers completion / hover / definition / CodeLens
- `language/luauContext.ts` — brace-aware `c:method({…})` context
- `language/hostApiSurface.ts` — generated static host API keys/docs (regenerate via `scripts/generate-host-api-surface.sh`)
- `language/providers/` — catalog-backed completions (`RecipeRepository`)
- Host catalog also returns `queryIds` / `queries` / `types` for capture + type completion
- Stubs: `vscode_extension/stubs/codemod-context.d.luau` (generated via `scripts/generate-host-api-surface.sh`)

Recipe ids / languages / maps / vars / query ids / args come from the host catalog at runtime.

### Persistent Rust host bridge + protocol framing
- `vscode_extension/src/extension/host/hostBridge.ts`
  - `list` / `reload` / `describe` / `preview` / `diff` / `apply` / `validate` / `bootstrap`
  - queued stdin writes; stdout framed parsing (`extractHostResultFrame`)

### Recipe catalog
- `vscode_extension/src/extension/recipes/recipeRepository.ts`
  - Keeps **last-good** recipes/diagnostics on transient host errors

### Webview runner
- `vscode_extension/src/extension/views/recipeRunnerViewProvider.ts`
- Shared messages: `vscode_extension/src/shared/messages.ts`

## Recipe runtime (maintainers)

Luau recipe loading: `rust/crates/lua/src/load.rs`. Host API spec structs: `rust/crates/lua/src/host_api/specs.rs`; macro definitions: `rust/crates/lua_macros/`; handlers/source of truth: `rust/crates/host/src/host_api/handlers.rs`; runtime binding: `rust/crates/host/src/host_api/registry.rs` + `lua_runtime.rs`.
Human/agent docs: skill `codemod-luau-recipes`.
