---
name: codemod-recipe-repo-orientation
description: Use when you need to understand where to make changes in this repo (Rust host/engine, VS Code extension backend, Vue webview UI). Provides a fast “where to look” map and common entry points.
disable-model-invocation: true
---

# Codemod Recipe Repo Orientation

## When to use

Use this skill when you are:
- New to the repo and need the **high-level architecture**.
- Trying to answer “where is X implemented?” across Rust host vs VS Code extension vs Vue UI.
- Planning changes that cross boundaries (protocol, schema, preview/diff/apply, selection).

## Repo map (fast)

- **Rust workspace**: `rust/`
  - YAML model + validation: `rust/crates/yaml/`
  - Tree-sitter engine: `rust/crates/engine/`
  - Host + MCP + bootstrap: `rust/crates/host/` (`dispatch.rs`, `protocol.rs`, `runner.rs`, `registry.rs`)
  - Binaries: `codemod_host`, `codemod_mcp` (`rust/crates/host/src/bin/`)

- **VS Code extension (TypeScript)**: `vscode_extension/src/`
  - Activation: `vscode_extension/src/extension/extension.ts`
  - Host spawn + protocol: `vscode_extension/src/extension/host/`
  - Recipe runner UI: `vscode_extension/src/extension/views/`
  - Shared message types: `vscode_extension/src/shared/messages.ts`

- **Webview UI (Vue 3 + Vite)**: `vscode_extension/src/webview/src/`
  - Entry: `main.ts` → `App.vue`
  - Extension client: `extensionClient.ts`, `composables/useHostState.ts`
  - Pure helpers: `webview/src/lib/*`

- **Built webview artifacts (don’t edit directly)**: `vscode_extension/media/`

- **Recipes and bootstrap export**: `.codemod/`, `export/.agents/skills/`

## Primary READMEs

- Human onboarding: `docs/getting-started.md` (map: `docs/README.md`)
- Package overview: `README.md`
- Extension usage + dev workflow: `vscode_extension/README.md`
- Architecture: `ARCHITECTURE.md`

## “Where should I change X?” cheatsheet

- **Recipe YAML parsing / validation**: `rust/crates/yaml/src/model.rs`, `validate.rs`
- **Query ops (insert/replace/remove)**: `rust/crates/engine/`
- **Registry, templates, maps**: `rust/crates/host/src/registry.rs`, `template.rs`, `map_registry.rs`
- **Host protocol** (list/describe/preview/diff/apply):
  - Rust: `rust/crates/host/src/dispatch.rs`, `protocol.rs`
  - TS: `vscode_extension/src/extension/host/hostProtocol.ts`
- **Recipe JSON schema for extension**: `vscode_extension/schemas/recipe.schema.json`
- **Webview ↔ extension messages**: `vscode_extension/src/shared/messages.ts`
- **Preview/apply UI**: `vscode_extension/src/webview/src/views/RunnerView.vue`, `composables/useRunnerController.ts`

## Red flags

- Changing host JSON shapes without updating `messages.ts` and webview handlers.
- Editing `vscode_extension/media/*` instead of rebuilding from `webview/`.
- Updating `.agents/skills/` without mirroring `export/.agents/skills/` (bootstrap source).

## Maintenance rule (docs/skills/rules)

When you change protocol, schema, or UI contracts, update READMEs and the skills listed in `.cursor/skills/codemod-recipe-change-checklist/SKILL.md`. When day-1 product concepts change, also update `docs/getting-started.md` using `.cursor/skills/codemod-recipe-human-docs/`.
