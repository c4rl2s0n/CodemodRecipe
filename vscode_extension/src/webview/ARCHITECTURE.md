# Codemod Recipe webview architecture

The recipe runner UI is a Vue 3 app bundled into `vscode_extension/media/` and loaded in a VS Code webview. Communication with the extension host uses a typed message protocol in `src/shared/messages.ts`.

## Layers

| Layer | Path | Responsibility |
|-------|------|----------------|
| Protocol | `src/shared/` | Message unions, guards, domain types |
| Transport | `src/webview/src/vsCodeApi.ts` | `acquireVsCodeApi`, `postToExtension`, `onExtensionMessage`, persistence |
| Outbound API | `src/webview/src/extensionClient.ts` | Typed methods for every webview → extension action |
| Inbound router | `src/webview/src/extensionInbound.ts` | Subscribe by `EXTENSION_TO_WEBVIEW` message type |
| Host state | `src/webview/src/composables/useHostState.ts` | Catalog, bootstrap, selected recipe from `state` messages |
| Runner session | `src/webview/src/composables/useRunnerController.ts` | Args, preview, review, local persistence |
| Extension dispatch | `src/extension/views/recipeRunnerHandlers.ts` | One handler per webview → extension message |

Vue components call **`useExtensionClient()`** (provided from `App.vue`). Do not import `vsCodeApi` from components.

## State ownership

| Store | Source of truth | Updated by |
|-------|-----------------|------------|
| **Host state** | `RecipeRunnerState` in the extension | `EXTENSION_TO_WEBVIEW.state` |
| **Runner session** | Webview refs in `useRunnerController` | User input, preview/apply responses |
| **Persisted cache** | `vscode.getState()` | `persistUiState()` — UI survival only; does not replace host recipe list |

## Adding a feature

1. Extend `src/shared/messages.ts` (constants, unions, guards).
2. Add handler in `recipeRunnerHandlers.ts`.
3. Add method on `ExtensionClient`.
4. Subscribe in `useHostState` or `useRunnerController` (or a new composable) via `extensionInbound`.
5. Wire UI through `useExtensionClient()`.
6. Add tests for guards and non-trivial client behavior.

## Dev loop

From `vscode_extension/`: run `./build.sh` or `npm run compile` and the webview build script, then reload the Extension Development Host.

## Picker cancel behavior

`ExtensionClient.pickPath()` resolves when the extension posts `filePicked`. If the user dismisses the native dialog, no message is sent and the Promise does not settle.
