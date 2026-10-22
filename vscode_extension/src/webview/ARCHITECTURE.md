# Codemod Recipe webview architecture

The recipe runner UI is a Vue 3 app bundled into `vscode_extension/media/` and loaded in a VS Code webview. Communication with the extension host uses a typed message protocol in `src/shared/messages.ts`.

## Layers

| Layer | Path | Responsibility |
|-------|------|----------------|
| Protocol | `src/shared/` | Message unions, guards, domain types, shared `argsKey` |
| Transport | `src/webview/src/vsCodeApi.ts` | `acquireVsCodeApi`, `postToExtension`, `onExtensionMessage`, persistence |
| Outbound API | `src/webview/src/extensionClient.ts` | Typed methods + ACK-required `request()` RPC helper |
| Inbound router | `src/webview/src/extensionInbound.ts` | Subscribe by `EXTENSION_TO_WEBVIEW` message type |
| Host state | `src/webview/src/composables/useHostState.ts` | Catalog, bootstrap, selected recipe, session snapshot from `state` messages |
| Runner session | `src/webview/src/composables/useRunnerController.ts` | Form draft, preview debounce/queue, review UI |
| Extension dispatch | `src/extension/views/recipeRunner/index.ts` | One handler per webview → extension message |

Vue components call **`useExtensionClient()`** (provided from `App.vue`). Do not import `vsCodeApi` from components.

## State ownership

| Store | Source of truth | Updated by |
|-------|-----------------|------------|
| **Host session** | `RecipeRunnerState` (+ `session` snapshot) | Host ops; pushed via `EXTENSION_TO_WEBVIEW.state` |
| **Runner draft** | Webview refs in `useRunnerController` | User input; preview/apply terminal replies |
| **Persisted cache** | `vscode.getState()` | Draft UX only (args, tab, last files for display) — never Apply-ready / in-flight |

### ACK-required protocol

Request-carrying messages (**preview**, **pickFile** / **pickDirectory**, and future apply RPCs) include a `requestId`. The host **must** send exactly one terminal reply for that id:

| Op | Terminal |
|----|----------|
| preview | `previewResult` or `error` (plus `previewState` for in-flight signals) |
| pickPath | `filePicked` (`value` or `cancelled: true`) or `error` |

**Never silent-return** from a request handler. If a preview is already running, still ACK with `previewState { inFlight: true, requestId }` so the webview can queue.

Webview `ready` and webview resolve/dispose **bump `session.id`** and clear host `_previewInFlight`. Stale replies for older sessions/requests are ignored.

Use `createRequestRpc` / `ExtensionClient.pickPath` for Promise-based ops so cancel and errors always settle.

### Session phases

Host `session.phase`: `idle` → `pickingPath` | `previewing` → `reviewing` → `applying` → `idle`. The webview may keep a local debounce/`pendingAutoPreview` policy, but authoritative in-flight is the host phase + terminal ACKs.

## Adding a feature

1. Extend `src/shared/messages.ts` (constants, unions, guards) — include `requestId` for any request/response op.
2. Add handler in `recipeRunner/index.ts` that **always** posts a terminal reply.
3. Add method on `ExtensionClient` (prefer `request()` for Promise APIs).
4. Subscribe in `useHostState` or `useRunnerController` via `extensionInbound`.
5. Wire UI through `useExtensionClient()`.
6. Add tests for guards and non-trivial client/host behavior.

## Dev loop

From `vscode_extension/`: run `./build.sh` or `npm run compile` and the webview build script, then reload the Extension Development Host.

## Path picker

`ExtensionClient.pickPath()` waits for `filePicked` with a matching `requestId`. Dialog dismiss posts `{ cancelled: true }`; the Promise resolves to `undefined` and the Browse button does not overwrite the arg.
