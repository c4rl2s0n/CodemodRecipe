---
name: codemod-recipe-vue-development
description: Use when implementing or modifying the `codemod-recipe` Vue 3 webview UI to follow TypeScript-first Composition API patterns, type-safe component bindings (prefer `defineModel`), and the project’s Vue-to-extension message protocol. Focuses on `vscode_extension/webview-ui` conventions and Vitest unit tests for pure logic.
disable-model-invocation: true
---

# Codemod Recipe Vue Development

This project uses Vue 3 + TypeScript inside a VS Code webview (`vscode_extension/webview-ui`). When working on UI components, composables, and message handlers, follow these rules to keep the code type-safe, consistent, and robust to async ordering issues.

## Red Flags (Stop and Fix)

1. **Bypassing the message protocol**
   - Posting messages without using the project’s typed constants in [`vscode_extension/webview-ui/src/messages.ts`](vscode_extension/webview-ui/src/messages.ts).
   - Sending payloads that do not match the corresponding TypeScript union types in [`vscode_extension/webview-ui/src/messages.ts`](vscode_extension/webview-ui/src/messages.ts).

2. **Async ordering bugs in webview logic**
   - Implementing preview/apply-like flows without an ordering strategy (the project guards stale responses using `requestId` and “latest handled” logic in [`vscode_extension/webview-ui/src/composables/useRunnerController.ts`](vscode_extension/webview-ui/src/composables/useRunnerController.ts)).

3. **Weak or missing TS contracts**
   - Untyped `defineProps()` / `defineEmits()` usage when the shape is known.
   - Introducing `any` or unstructured objects into message handling where a project type exists.

4. **Wrong binding pattern for two-way data**
   - Manual `modelValue` + `update:modelValue` wiring (Vue 2 style) instead of using `defineModel()` in components that need v-model behavior.

5. **UI logic leaking into composables**
   - Putting DOM/UI manipulation (toasts/modals/alerts) inside composables.
   - Composables should return state + error info; components render the UI.

6. **Tests that assume a DOM testing stack**
   - Adding Testing Library / MSW patterns without project setup.
   - Writing tests that depend on rendered DOM/UI when this repo currently uses Vitest unit tests for pure helpers (see `src/lib/*/*.test.ts`).

7. **`setTimeout()` inside tests**
   - Using timeouts as a substitute for proper async utilities (prefer Vitest async helpers; keep tests deterministic).

## Quick Rules

### Components

- Use `<script setup lang="ts">`.
- Use `defineProps<...>()` with explicit TypeScript types.
- Use `defineEmits<...>()` with fully typed event signatures (including `'update:*'` events).
- For two-way binding, **prefer `defineModel<T>()`**.
  - In parent templates, you may use either:
    - `v-model:<name>` syntax, or
    - `:prop-name="..."` + `@update:prop-name="..."`.

Examples from this repo:
- `defineModel` used for arg input state in [`vscode_extension/webview-ui/src/components/RecipeArgForm.vue`](vscode_extension/webview-ui/src/components/RecipeArgForm.vue) and [`vscode_extension/webview-ui/src/components/ArgField.vue`](vscode_extension/webview-ui/src/components/ArgField.vue).
- `update:*` typed events for controlled state in [`vscode_extension/webview-ui/src/components/FileCard.vue`](vscode_extension/webview-ui/src/components/FileCard.vue) and [`vscode_extension/webview-ui/src/components/PatchRow.vue`](vscode_extension/webview-ui/src/components/PatchRow.vue).

### Composables

- Name composables with `use*` (e.g. `useRunnerController`, `useHostState`).
- Keep composables focused on:
  - state management,
  - message handling,
  - deriving UI-ready state (statuses/errors/flags).
- Do not embed DOM/UI behaviors.

### Webview ↔ Extension messaging

- Always:
  - import `WEBVIEW_TO_EXTENSION` / `EXTENSION_TO_WEBVIEW` from `src/messages.ts`,
  - send via `postToExtension(...)`,
  - handle messages through the typed unions in `src/messages.ts`.
- If a message can be returned out of order (like preview results), include a `requestId` strategy and ignore stale responses (mirror the approach used in `useRunnerController`).

### Types

- Import and use types from `src/types.ts` rather than re-declaring shapes.
- Prefer `type` imports (`import type { ... }`) for TS-only references.

## Component Implementation Workflow

When implementing a new component or refactoring an existing one:

1. Define the types:
   - Props: `defineProps<{ ... }>()`
   - Emits: `defineEmits<{ ... }>()`
   - Models: `defineModel<T>()` if the component participates in v-model.
2. Wire bindings in the template:
   - Use `v-model`/`defineModel` for two-way data entry.
   - Use `update:*` events for parent-owned controlled arrays/structures (when consistent with the existing component design).
3. Keep side-effects out of templates:
   - Template expressions should not contain complex logic; move logic into functions/computed in `<script setup>`.
4. If the component triggers extension actions:
   - Use the `WEBVIEW_TO_EXTENSION` constants and keep payload shapes in sync with `src/messages.ts`.
5. If you add tests:
   - Keep them focused on pure logic in `src/lib/*` where possible.
   - Do not introduce a DOM-testing stack unless the project already has it configured.

## Composables & Async Messaging Workflow

For composables that trigger host work (preview/apply/bootstrapping):

1. Generate a request identity (usually `requestId`).
2. In the extension message handler:
   - ignore stale responses (based on request ordering),
   - update state only for the newest relevant request.
3. Expose state back to components:
   - statuses, error text, and UI flags (so UI can render appropriate feedback).

## Testing Guidance (Vitest-only, Pure Logic)

This repo currently uses Vitest unit tests for pure helper logic under `src/lib/*`.

- Prefer testing functions that transform data:
  - e.g. arg collection/keys,
  - selection building,
  - include/exclude derivations.
- Use `describe`/`it` + `expect` (Vitest).
- Avoid introducing DOM/UI test tooling in this baseline skill unless the repo setup changes.

## When Not to Use This Skill

- When working outside the Vue webview UI (the VS Code extension backend is covered by other skills).
- When the change introduces an entirely new testing framework or mocking stack; in that case, request a dedicated setup plan first.

