---
name: codemod-recipe-webview-ui-development
description: Use when implementing or refactoring the Vue 3 webview UI in this repo (`vscode_extension/webview-ui`). Covers the project’s component binding patterns (`defineModel`, typed `defineProps`/`defineEmits`), state/controller composables, and persisted UI state via vsCodeApi.
disable-model-invocation: true
---

# Vue Webview UI Development

## When to use

Use this skill when you need to:
- Add/modify Vue components in `vscode_extension/webview-ui`
- Understand how webview state is stored, updated, and persisted
- Keep UI event contracts consistent with typed extension messaging
- Debug why UI doesn’t update (or updates with stale data)

## Where to look (main structure)

### App bootstrap + routing between tabs
- `vscode_extension/webview-ui/src/App.vue`
  - holds `hostState` (recipes, bootstrap state, active tab)
  - creates/uses `useRunnerController`
  - listens to `onExtensionMessage(...)` and routes messages

### Controller composable (runner state)
- `vscode_extension/webview-ui/src/composables/useRunnerController.ts`
  - owns:
    - `argValues`
    - `files` + `fileSelections`
    - `activeChangeIndex`
    - `previewStatus` / `previewStatusKind`
    - `previewInFlight` and stale request suppression
    - `errorMessage` / `showReview`
  - persists and restores UI state via vsCodeApi

### Host state composable (bootstrap + readiness)
- `vscode_extension/webview-ui/src/composables/useHostState.ts`
  - posts `WEBVIEW_TO_EXTENSION.ready`
  - listens for `EXTENSION_TO_WEBVIEW.state`
  - tracks bootstrap flags

### VS Code webview persistence + message plumbing
- `vscode_extension/webview-ui/src/vsCodeApi.ts`
  - `postToExtension(...)`
  - `onExtensionMessage(...)` subscription system
  - `getPersistedState()` / `setPersistedState()` wrappers

## Component binding patterns (project-specific)

### Typed props + typed emits
- Components in this repo generally use:
  - `defineProps<...>()` for props
  - `defineEmits<...>()` for emitted events

Examples:
- [`RecipeArgForm.vue`](vscode_extension/webview-ui/src/components/RecipeArgForm.vue)
- [`FileCard.vue`](vscode_extension/webview-ui/src/components/FileCard.vue)
- [`PatchRow.vue`](vscode_extension/webview-ui/src/components/PatchRow.vue)
- [`RunnerView.vue`](vscode_extension/webview-ui/src/views/RunnerView.vue)

### Two-way binding with `defineModel`
- For user-editable values, use `defineModel` rather than manual `modelValue` plumbing.

Examples:
- `RecipeArgForm.vue`: `defineModel<Record<string, string>>('argValues', ...)`
- `ArgField.vue`: `defineModel<string>({ required: true })`

Important:
- `RecipeArgForm` watches `argValues` and emits `args-changed` when inputs mutate.

## Controlled state updates: `update:*` vs local refs

This project uses `update:*` events for parent-owned controlled values:
- `RunnerView` emits:
  - `'update:argValues'`
  - `'update:fileSelections'`
  - `'update:activeChangeIndex'`

- Parent `App.vue` receives those events and updates local refs:
  - `@update:arg-values="argValues = $event"`

This keeps state ownership explicit and avoids “hidden coupling” between nested components.

## Persisted UI state contract

Persistence is handled in `useRunnerController.ts`:
- persists:
  - `recipeId`, `activeTab`
  - `argValues`, `files`, `activeChangeIndex`
  - `lastPreviewArgsKey`, `lastPreviewSuccess`
- restores on mount:
  - `restorePersistedOnMount()`

Always use `setPersistedState(...)` and `getPersistedState()`:
- `vscode_extension/webview-ui/src/vsCodeApi.ts`

## Stale request/preview ordering (UI side)

Preview flow:
- webview triggers preview requests (debounced) from `useRunnerController.ts`
- it tracks:
  - `latestRequestId`
  - `latestHandledRequestId`
- it ignores:
  - messages with `requestId < latestHandledRequestId`

Red flag:
- If you add new preview-like messaging, you must preserve this stale suppression pattern.

## Red flags (stop and fix)

- Updating message protocol types without updating UI handler:
  - `vscode_extension/webview-ui/src/messages.ts`
  - `vscode_extension/webview-ui/src/composables/useRunnerController.ts`
- Bypassing vsCode persistence:
  - calling `window.acquireVsCodeApi()` directly in random components
  - instead use `vsCodeApi.ts` wrappers
- Moving UI side-effects into composables:
  - composables should expose error text / flags; components render
- Breaking `defineModel` contracts:
  - changing the model name (`'argValues'`) without updating parent `v-model` wiring

