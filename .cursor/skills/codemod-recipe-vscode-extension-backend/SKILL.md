---
name: codemod-recipe-vscode-extension-backend
description: Use when implementing or debugging the VS Code extension backend (TypeScript). Covers activation, recipe discovery/refresh, the webview view provider message loop, preview/diff/apply flows, and Dart host process management via DartBridge.
disable-model-invocation: true
---

# VS Code Extension Backend (TypeScript)

## When to use

Use this skill when you need to:
- Debug webview interactions (messages from UI not producing expected state)
- Change preview/diff/apply behavior
- Understand how the persistent Dart host is started, queued, and parsed
- Update protocol glue after modifying the Dart host or shared types

## Where to look (entry points)

### Extension activation + commands
- `vscode_extension/src/extension.ts`
  - `activate(...)`: wires discovery + bridge + view provider
  - Commands:
    - `codemodRecipe.refresh`
    - `codemodRecipe.bootstrap`
    - `codemodRecipe.runRecipe`
    - `codemodRecipe.runFromCursorContext`
    - `codemodRecipe.configureHost`

### Persistent Dart host bridge + protocol framing
- `vscode_extension/src/host/dartBridge.ts`
  - `list`, `describe`, `preview`, `diff`, `apply`
  - persistent host:
    - queued stdin writes to keep requests ordered
    - stdout framed parsing using markers (`extractHostResultFrame`)
  - fallback:
    - if persistent host fails, it retries in one-shot mode

### Recipe discovery + staleness refresh
- `vscode_extension/src/recipes/recipeRepository.ts`
  - caches recipes + last refresh time + last error
  - `shouldRefresh(maxAgeMs)` drives auto refresh on run/entry

### Webview view provider + message handling
- `vscode_extension/src/views/recipeRunnerViewProvider.ts`
  - `resolveWebviewView`: sets CSP/script options, sends initial state, registers message handler
  - `handleMessage(message)`:
    - `WEBVIEW_TO_EXTENSION.ready`: re-send state (webview may miss early messages)
    - `selectRecipe`, `refreshRecipes`, `configureHost`, `pickFile/pickDirectory`
    - `preview`: calls `preview(args, requestId)`
    - `openDiff`: calls `openDiffByPath(path)`
    - `apply`: calls `apply(selection)`
  - `postState()`: pushes `EXTENSION_TO_WEBVIEW.state` to the webview

### Native diff content materialization
- `vscode_extension/src/diff/diffContentProvider.ts`
  - `DiffContentProvider.store(key, content)` stores content for the `vscode.diff` view

## Webview message loop (key behaviors)

### Typed message filtering
- Message input is unknown at runtime; it is type-narrowed via:
  - `vscode_extension/src/views/recipeRunnerMessages.ts`
  - `isWebviewToExtensionMessage(...)`

### State send timing
- The view provider acknowledges `WEBVIEW_TO_EXTENSION.ready` and re-sends latest state.
- This prevents “script loaded late” bugs where the webview misses the initial `state` message.

## Preview flow (what happens on `preview`)

In `recipeRunnerViewProvider.ts`:
1. Guard: `this.previewInFlight` prevents multiple concurrent preview calls.
2. Immediately post `EXTENSION_TO_WEBVIEW.previewState` with `inFlight: true` and optional `requestId`.
3. Save args (`this.state.lastArgs = args`), compute `argsKey`, then:
   - `await this.bridge.preview(recipe.id, args, this.config.previewSnippetLines)`
4. If `response.ok`:
   - update `this.state.lastFiles = response.files ?? []`
   - post `EXTENSION_TO_WEBVIEW.previewResult` with `files`, `requestId`, `argsKey`
5. Finally (always):
   - post `previewState` with `inFlight: false`

This means the webview should consider `inFlight` as a UI/ordering signal, not as a final truth source.

## Diff flow (what happens on `openDiff`)

In `recipeRunnerViewProvider.ts`:
1. Find the target `FilePreview` in `this.state.lastFiles`.
2. If `original`/`modified` are missing, it calls:
   - `ensureDiffMaterialized(file)` which:
     - invokes `bridge.diff(recipe.id, this.state.lastArgs, file.path)`
     - updates `this.state.lastFiles[index] = response.file`
3. Then it calls `openDiff(file)` which:
   - stores original/modified text into `DiffContentProvider` under stable keys
   - calls VS Code built-in `vscode.diff` with those URIs

## Apply flow (what happens on `apply`)

In `recipeRunnerViewProvider.ts`:
1. Calls `bridge.apply(recipe.id, this.state.lastArgs, selection)`
2. If `ok`:
   - shows an info message “Applied …”
   - posts `EXTENSION_TO_WEBVIEW.applyResult` with the applied file list
3. If `not ok`:
   - posts `EXTENSION_TO_WEBVIEW.error`

## Red flags (stop and fix)

- Editing `vscode_extension/media/*` directly:
  - those are compiled artifacts.
- Changing message/response shapes without updating unions/types:
  - `vscode_extension/src/types.ts`
  - `vscode_extension/src/constants.ts`
  - `vscode_extension/src/views/recipeRunnerMessages.ts`
  - `vscode_extension/webview-ui/src/messages.ts`
  - `vscode_extension/webview-ui/src/types.ts`
- Breaking request/response ordering in DartBridge:
  - never remove the `queue` chaining (`this.queue = this.queue.then(...)`) without replacing with an equivalent ordering mechanism
  - never change stdout marker handling without updating frame extraction logic
- Removing the `WEBVIEW_TO_EXTENSION.ready` re-sync behavior:
  - makes early-state race conditions likely
- Altering `previewInFlight` guard without introducing correct webview-side ordering:
  - preview can otherwise interleave and “latest wins” semantics break

## Safe change workflow

When modifying backend behavior:

1. Update protocol/types in `vscode_extension/src/types.ts` and message unions.
2. Update Dart host JSON encoding/parsing if needed:
   - `lib/src/vscode/codemod_host.dart` (+ helpers in `lib/src/vscode/*`)
3. Update extension message handler:
   - `vscode_extension/src/views/recipeRunnerViewProvider.ts`
4. Update webview controller:
   - `vscode_extension/webview-ui/src/composables/useRunnerController.ts`
5. Only then adjust UI components (Vue).

