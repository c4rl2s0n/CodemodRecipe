---
name: codemod-recipe-vue-development
description: Use when implementing or modifying the `codemod-recipe` Vue 3 webview UI to follow TypeScript-first Composition API patterns, type-safe component bindings (prefer `defineModel`), and the project’s Vue-to-extension message protocol. Focuses on `vscode_extension/src/webview` conventions and Vitest unit tests for pure logic.
disable-model-invocation: true
---

# Codemod Recipe Vue Development

This project uses Vue 3 + TypeScript inside a VS Code webview (`vscode_extension/src/webview`). When working on UI components, composables, and message handlers, follow these rules to keep the code type-safe, consistent, and robust to async ordering issues.

## Red Flags (Stop and Fix)

1. **Bypassing the extension client**
   - Importing `vsCodeApi` or calling `postToExtension` from components.
   - Use **`useExtensionClient()`** (provided in `App.vue`) for any VS Code action.

2. **Async ordering bugs in webview logic**
   - Preview flows without `requestId` stale handling in [`useRunnerController.ts`](vscode_extension/src/webview/src/composables/useRunnerController.ts).

3. **Weak or missing TS contracts**
   - Untyped `defineProps()` / `defineEmits()` when the shape is known.

4. **Wrong binding pattern for two-way data**
   - Prefer `defineModel<T>()` over manual `modelValue` + `update:modelValue`.

5. **UI logic leaking into composables**
   - Composables return state + flags; components render UI.

## Quick Rules

### Components

- `<script setup lang="ts">`, typed props/emits/models.
- Extension actions: `const client = useExtensionClient()` then `client.selectRecipe(id)`, `client.openDiff(path)`, `client.pickPath(arg, isDir)`, etc.
- Path/file picks: `await client.pickPath(...)` then update v-model (triggers preview via `RecipeArgForm` watch).

### Composables

- `useHostState(inbound)` — catalog/bootstrap from `state` messages.
- `useRunnerController({ client, inbound, ... })` — preview, review, persistence.
- Do not import `vsCodeApi` outside `extensionClient.ts` / `extensionInbound.ts`.

### Types

- Import from `../shared` (re-exports `src/shared`).

## Layout-only changes

Tabs, CSS, and prop wiring between `App.vue` → views/components usually need **no** protocol changes.

## Testing (Vitest)

- Pure logic: `src/lib/*.test.ts`
- Client/guards: `extensionClient.test.ts`, `src/shared/messages.test.ts`
- Run from `vscode_extension/src/webview`: `npm test`

## Architecture reference

[`vscode_extension/src/webview/ARCHITECTURE.md`](vscode_extension/src/webview/ARCHITECTURE.md)
