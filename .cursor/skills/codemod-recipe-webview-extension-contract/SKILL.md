---
name: codemod-recipe-webview-extension-contract
description: Use when modifying or debugging the message contract between the VS Code extension webview and the extension backend. Covers WEBVIEW_TO_EXTENSION/EXTENSION_TO_WEBVIEW constants, typed union message definitions, state shape, and request/response stale ordering via requestId.
disable-model-invocation: true
---

# Webview ↔ Extension Message Contract

## When to use

Use this skill when you:
- Add a new webview action or host response type
- Change the payload shape for preview/diff/apply related UI flows
- Debug cases where the UI shows stale preview results or mismatched error messages
- Update shared types and need to keep both TS sides consistent

## Source of truth: message constants + typed unions

### Webview → Extension
- `vscode_extension/src/constants.ts`
  - `WEBVIEW_TO_EXTENSION` string constants
- `vscode_extension/src/views/recipeRunnerMessages.ts`
  - `WebviewToExtensionMessage` typed union
  - `isWebviewToExtensionMessage(...)` runtime guard

### Extension → Webview
- `vscode_extension/src/constants.ts`
  - `EXTENSION_TO_WEBVIEW` string constants
- `vscode_extension/src/webview/extensionToWebviewMessages.ts`
  - `ExtensionToWebviewMessage` typed union

### Webview UI message types (mirrored)
- `vscode_extension/webview-ui/src/messages.ts`
  - `WEBVIEW_TO_EXTENSION` / `EXTENSION_TO_WEBVIEW` constants
  - `WebviewToExtensionMessage` / `ExtensionToWebviewMessage` equivalents

## Shared state shape (big picture)

State is pushed as `EXTENSION_TO_WEBVIEW.state` and stored as `RecipeViewState`.

- Extension-side state type:
  - `vscode_extension/src/webview/webviewState.ts` (`RecipeViewState`)
- Webview-side state type:
  - `vscode_extension/webview-ui/src/types.ts` (`RecipeViewState`)

Webview controller updates its local state based on:
- `vscode_extension/webview-ui/src/App.vue` (bootstraps and routes tab changes)
- `vscode_extension/webview-ui/src/composables/useRunnerController.ts`
  - `handleExtensionMessage(msg, state?)`
  - applies `previewResult`, `error`, `previewState`, and `applyResult` updates

## requestId + stale ordering (critical)

This repo uses `requestId` for preview ordering and stale suppression.

Where requestId is carried:
- Webview sends `preview` and includes optional `requestId`:
  - `vscode_extension/webview-ui/src/messages.ts` (`WEBVIEW_TO_EXTENSION.preview`)
  - webview controller sends it:
    - `vscode_extension/webview-ui/src/composables/useRunnerController.ts`
      - `triggerPreview(...)` increments and passes `requestId`

- Extension forwards `requestId` back in responses:
  - `vscode_extension/src/views/recipeRunnerViewProvider.ts`
    - `preview(...)` posts:
      - `previewState` (`inFlight: true/false`)
      - `previewResult` on success
      - `error` on failure

Where stale results are handled:
- Webview controller ignores older requests:
  - `vscode_extension/webview-ui/src/composables/useRunnerController.ts`
    - tracks `latestRequestId` vs `latestHandledRequestId`
    - drops messages with `requestId < latestHandledRequestId.value`

Red flag:
- If you add `requestId` to a new message type, you must also implement stale suppression logic on the webview side (and/or extension side) as appropriate.

## UI request in-flight signaling

Even with `requestId`, the extension also emits an `inFlight` signal to support UI feedback:
- `EXTENSION_TO_WEBVIEW.previewState` with `inFlight: true|false`

Webview uses this to:
- set `previewInFlight`
- when `inFlight` ends, it may trigger another queued preview:
  - `useRunnerController.ts` (`pendingAutoPreview`)

## Checklist: when you change the protocol

When adding/changing any message type or payload:

1. Update both sides’ constants:
   - `vscode_extension/src/constants.ts`
   - `vscode_extension/webview-ui/src/messages.ts`
2. Update union types + guards:
   - `vscode_extension/src/views/recipeRunnerMessages.ts`
   - `vscode_extension/src/webview/extensionToWebviewMessages.ts`
   - `vscode_extension/webview-ui/src/messages.ts`
3. Update extension handler and response serialization:
   - `vscode_extension/src/views/recipeRunnerViewProvider.ts`
4. Update webview controller state handling:
   - `vscode_extension/webview-ui/src/composables/useRunnerController.ts`
5. Update any message validators:
   - `isWebviewToExtensionMessage(...)`
6. Update UI components that depend on state/events:
   - `vscode_extension/webview-ui/src/App.vue`
   - `vscode_extension/webview-ui/src/views/*`
   - `vscode_extension/webview-ui/src/components/*`

## Red flags (stop and fix)

- Updating only one side’s message union/constants.
- Changing preview/diff/apply payload keys without updating:
  - extension types
  - webview types
  - UI controller mappings
- Removing stale suppression in `useRunnerController.ts`.
- Changing marker strings or typed union discriminators without frame extraction updates.

