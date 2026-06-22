---
name: codemod-recipe-repo-orientation
description: Use when you need to understand where to make changes in this repo (Dart codemod_recipe core, VS Code extension backend, Vue webview UI, and the Dart stdio host). Provides a fast “where to look” map and common entry points.
disable-model-invocation: true
---

# Codemod Recipe Repo Orientation

## When to use

Use this skill when you are:
- New to the repo and need the **high-level architecture**.
- Trying to answer “where is X implemented?” across Dart tool vs VS Code extension vs Vue UI.
- Planning changes that cross boundaries (protocol, schema, preview/diff/apply, selection).

## Repo map (fast)

- **Dart library (core API)**: `lib/` and `lib/src/`
  - Main exports: `lib/codemod_recipe.dart`
  - Core primitives: context/recipe/runner/operations/transforms/patches

- **Dart library (VS Code host bridge)**: `lib/src/vscode/`
  - Host process & protocol: `lib/src/vscode/codemod_host.dart`
  - JSON recipe schema: `lib/src/vscode/recipe_schema.dart`
  - Diff/preview serialization: `lib/src/vscode/diff_service.dart`
  - Patch selection semantics: `lib/src/vscode/patch_selector.dart`

- **VS Code extension (TypeScript)**: `vscode_extension/src/`
  - Activation + commands: `vscode_extension/src/extension.ts`
  - Host process bridge: `vscode_extension/src/host/dartBridge.ts`
  - Webview view provider: `vscode_extension/src/views/recipeRunnerViewProvider.ts`
  - Diff content provider: `vscode_extension/src/diff/diffContentProvider.ts`
  - Constants (commands, message types, protocol markers): `vscode_extension/src/constants.ts`

- **Webview UI (Vue 3 + Vite)**: `vscode_extension/webview-ui/src/`
  - Entry: `main.ts` → `App.vue`
  - UI components: `components/*.vue`
  - State/controller composables: `composables/*`
  - Webview message contract: `messages.ts`
  - Persisted state + VS Code API wrapper: `vsCodeApi.ts`
  - Pure helpers + tests: `lib/*` + `lib/*.test.ts` (Vitest)

- **Built webview artifacts (don’t edit directly)**: `vscode_extension/media/`
  - `recipeView.js`, `recipeView.css`, `recipeView.html` (compiled output)

- **Examples**: `example/`
  - VS Code host example entrypoint: `example/vscode_host_example/bin/codemod_host.dart`

## Primary READMEs

- Package overview: `README.md`
- Extension usage + dev workflow: `vscode_extension/README.md`
- Example hosts: `example/*/README.md` (if present)

## “Where should I change X?” cheatsheet

- **CLI behavior** (args parsing, apply vs dry-run): `lib/src/runner.dart`
- **Recipe definition** (args/operations/postExecution/templates): `lib/src/recipe.dart`
- **Planned changes model** (`FileChange`, patch vs create): `lib/src/operation.dart`
- **Patch correctness/order**: `lib/src/patch_helpers.dart`
- **Host protocol shape** (list/describe/preview/diff/apply; persistent mode; stdout framing):
  - Dart side: `lib/src/vscode/codemod_host.dart`
  - TS side: `vscode_extension/src/host/hostProtocol.ts` + `vscode_extension/src/host/dartBridge.ts`
- **Recipe metadata shown in UI** (inputKind/options/contextKey/templates): `lib/src/vscode/recipe_schema.dart`
- **Preview cards/snippets/patch rows UI**: `vscode_extension/webview-ui/src/components/*`
- **Apply selection semantics** (include/patch indices): `lib/src/vscode/patch_selector.dart` + `vscode_extension/webview-ui/src/lib/selection.ts`
- **Native diff open**: `vscode_extension/src/views/recipeRunnerViewProvider.ts` + `vscode_extension/src/diff/diffContentProvider.ts`

## Red flags

- Changing the host’s JSON response shape without updating TypeScript types:
  - `vscode_extension/src/types.ts`
  - `vscode_extension/webview-ui/src/types.ts`
- Adding/changing a message type in one place but not the other:
  - Extension: `vscode_extension/src/constants.ts` + `vscode_extension/src/views/recipeRunnerMessages.ts`
  - Webview: `vscode_extension/webview-ui/src/messages.ts`
- Editing `vscode_extension/media/*` directly instead of `vscode_extension/webview-ui/*`.

## Maintenance rule (docs/skills/rules)

When a code change affects usage, protocol, configuration, or conventions, update:
- `README.md` and/or `vscode_extension/README.md`
- relevant `.cursor/skills/*`
- relevant `.cursor/rules/*` (if present)
- `.cursor/skills/codebase-memory/reference.md` when subsystems or primary entry points change materially
- `.vibe.md` Agent tooling examples when exploration entry points change
- `.vibe/prompts/cli.md` when upgrading `mistral-vibe` (re-diff bundled `cli.md`)

