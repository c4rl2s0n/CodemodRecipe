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

## Source of truth

All protocol types live in **`vscode_extension/src/shared/messages.ts`** (re-exported from `src/shared/index.ts`).

- `WEBVIEW_TO_EXTENSION` / `WebviewToExtensionMessage` / `isWebviewToExtensionMessage`
- `EXTENSION_TO_WEBVIEW` / `ExtensionToWebviewMessage` / `isExtensionToWebviewMessage`

Domain types (`RecipeViewState`, `FilePreview`, …): **`vscode_extension/src/shared/types.ts`**.

## Webview interface layers

| Layer | File |
|-------|------|
| Transport | `vscode_extension/src/webview/src/vsCodeApi.ts` |
| Outbound API | `vscode_extension/src/webview/src/extensionClient.ts` |
| Inbound router | `vscode_extension/src/webview/src/extensionInbound.ts` |
| Host state | `vscode_extension/src/webview/src/composables/useHostState.ts` |
| Runner session | `vscode_extension/src/webview/src/composables/useRunnerController.ts` |

See **`vscode_extension/src/webview/ARCHITECTURE.md`** for state ownership and dev workflow.

Vue components must use **`useExtensionClient()`** — not `postToExtension` directly.

## Extension dispatch

- Orchestration: `vscode_extension/src/extension/views/recipeRunnerViewProvider.ts`
- Per-message handlers: `vscode_extension/src/extension/views/recipeRunnerHandlers.ts`

## requestId + stale ordering (critical)

- Webview: `useRunnerController.ts` → `client.requestPreview(args, requestId)`
- Extension: `recipeRunnerHandlers.ts` → `handlePreview` echoes `requestId` on `previewState`, `previewResult`, `error`
- Stale drop: `latestHandledRequestId` in `useRunnerController.ts`

## Checklist: add or change a capability

1. **`src/shared/messages.ts`** — constants, union arms, guards (`messages.test.ts`)
2. **`recipeRunnerHandlers.ts`** — extension behavior
3. **`extensionClient.ts`** — one new method per webview → extension action
4. **`useHostState` or `useRunnerController`** — subscribe via `extensionInbound` for new extension → webview events
5. **UI** — `useExtensionClient()` only
6. **Tests** — guard + client unit tests when non-trivial

Host-state-only features may only extend `RecipeViewState` and `postState()` without new webview → extension messages.

## Red flags

- Updating protocol in only one of extension vs webview (single `messages.ts` — but still update client, handlers, composables).
- Calling `postToExtension` from Vue components.
- Removing stale suppression in `useRunnerController.ts`.
- Weak guards that accept any `type` string (guards must whitelist discriminants).
