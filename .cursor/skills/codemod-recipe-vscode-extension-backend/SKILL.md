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

### Recipe Language Toolkit
- `vscode_extension/src/extension/language/session.ts` — provider registration (dynamic selectors)
- `yamlContext.ts` — indent parent/siblings (pure)
- `dslSurface.ts` — loads `schemas/generated-dsl-surface.json`
- `providers/` — completion, hover, definition, codelens
- `diagnostics.ts` — host `RecipeDiagnostic` → Problems panel

Structural completions come from codegen surface; recipe ids / languages / `with:` args come from the host catalog at runtime. Do **not** hardcode DSL container children in TypeScript.

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

## DSL codegen (maintainers)

Structural SSOT is `rust/crates/yaml/src/model.rs` (`JsonSchema`). After changing `model.rs`, `dsl::`, or `ENTRIES`, run:

```bash
scripts/generate-dsl-artifacts.sh
```

Outputs under `vscode_extension/schemas/`: JSON Schema files, `generated-dsl-surface.json`, `generated-keyword-docs.json`, plus TextMate patches. Do not hand-edit those outputs or maintain a parallel container inventory.
