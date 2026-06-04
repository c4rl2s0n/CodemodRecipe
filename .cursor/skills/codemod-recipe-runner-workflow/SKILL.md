---
name: codemod-recipe-runner-workflow
description: Use when explaining or debugging the end-to-end runner workflow across the VS Code webview UI, the extension backend, and the Dart host. Covers the preview → user review → selection → apply cycle, plus diff materialization.
disable-model-invocation: true
---

# codemod-recipe Runner Workflow (End-to-End)

## What this covers

This skill documents the actual runtime sequence for:
1. Selecting/running a recipe from the UI
2. Auto preview as args change
3. Stepping through review diffs and selectively applying patches
4. Opening a native diff for a specific file

## Where the workflow lives (key files)

### Webview UI (Vue)
- App + tab logic: `vscode_extension/webview-ui/src/App.vue`
- Runner controller + preview/apply logic:
  - `vscode_extension/webview-ui/src/composables/useRunnerController.ts`
- Views/components:
  - `vscode_extension/webview-ui/src/views/RunnerView.vue`
  - `vscode_extension/webview-ui/src/components/RecipeArgForm.vue`
  - `vscode_extension/webview-ui/src/components/ArgField.vue`
  - `vscode_extension/webview-ui/src/components/ReviewPanel.vue`
  - `vscode_extension/webview-ui/src/components/FileCard.vue`
  - `vscode_extension/webview-ui/src/components/PatchRow.vue`

### Extension backend (TypeScript)
- Webview view provider + message routing:
  - `vscode_extension/src/views/recipeRunnerViewProvider.ts`
- Persistent host bridge:
  - `vscode_extension/src/host/dartBridge.ts`

### Dart host + core
- Host entrypoint + JSON command dispatch:
  - `lib/src/vscode/codemod_host.dart`
- Diff/pick/selection:
  - `lib/src/vscode/diff_service.dart`
  - `lib/src/vscode/patch_selector.dart`

## End-to-end flow (step-by-step)

### 1. Extension boot + recipe list
1. Extension `activate()` triggers `bootstrap()`:
   - `vscode_extension/src/extension.ts`
   - ensures host and refreshes recipes
2. Extension pushes `EXTENSION_TO_WEBVIEW.state` to webview:
   - `RecipeRunnerViewProvider.postState()` (state includes recipes + bootstrap flags)
3. Webview App:
   - shows `BootstrapView` overlay until `bootstrapInFlight/phase` indicates ready

### 2. User selects a recipe
1. Webview sends `WEBVIEW_TO_EXTENSION.selectRecipe`:
   - from `vscode_extension/webview-ui/src/components/RecipesTab.vue`
2. Extension receives it and runs recipe:
   - `RecipeRunnerViewProvider.handleMessage(...)` -> `selectRecipe(...)`
   - `run(recipe)` triggers `runInternal(...)`
   - `revealAndPostState()` ensures the webview HTML is loaded and then posts state
3. Webview receives new `state` and controller resets:
   - `useRunnerController.applyHostState(...)`
   - initializes `argValues` (from `initialArgs`) and clears `files/fileSelections/showReview`

### 3. User edits args → auto preview
1. `RecipeArgForm` uses `defineModel('argValues')`.
2. It watches and emits `args-changed`.
3. `RunnerView` emits `argsChanged(immediate)` to `useRunnerController.onArgsChanged(...)`.
4. `useRunnerController` triggers preview:
   - debounced via `setTimeout` unless `immediate` is true
   - sends `WEBVIEW_TO_EXTENSION.preview` with:
     - `args: collectArgs(...)`
     - `requestId` for ordering
5. Extension handles `preview`:
   - guards with `previewInFlight`
   - posts `previewState(inFlight: true)`
   - calls `DartBridge.preview(...)`
   - on success posts `previewResult(files, requestId, argsKey)`
   - always posts `previewState(inFlight: false)`
6. Webview handles preview result:
   - `useRunnerController.handleExtensionMessage(...)` updates:
     - `files`
     - `fileSelections`
     - `activeChangeIndex`
     - `showReview`
   - drops stale results via `latestHandledRequestId`

### 4. User reviews patches and selection state
1. `ReviewPanel` renders `FileCard` entries.
2. `FileCard` shows checkbox + patch rows:
   - `FileCard` emits `update:selection` when file/patch toggles change
3. `ReviewPanel` updates the parent-owned `fileSelections`.
4. `activeChangeIndex` tracks which patch row is currently “active”.

### 5. Open native diff for current file
1. When a patch is selected or the user clicks patch rows:
   - webview sends `WEBVIEW_TO_EXTENSION.openDiff` with `path`
2. Extension `openDiffByPath`:
   - finds the file in `this.state.lastFiles`
   - if missing contents, calls `bridge.diff(...)` via `ensureDiffMaterialized`
3. Extension stores original/modified text in `DiffContentProvider` and triggers `vscode.diff`.

### 6. Apply Selected
1. User clicks “Apply Selected” in `ReviewPanel`.
2. Webview sends `WEBVIEW_TO_EXTENSION.apply` with:
   - `selection: buildSelection(props.fileSelections)`
3. Extension `apply(selection)` calls `bridge.apply(...)`.
4. On success extension posts `EXTENSION_TO_WEBVIEW.applyResult`.
5. Webview controller resets runner state:
   - hides review and clears files/selections.

## Failure modes to anticipate (where to debug)

- Preview results show “wrong” file set:
  - check request ordering:
    - webview stale suppression in `useRunnerController`
    - extension `previewInFlight` guard
- Diff shows empty content:
  - ensure diff materialization runs:
    - `ensureDiffMaterialized` in `recipeRunnerViewProvider.ts`
- Apply seems to do nothing:
  - verify `selection` shape:
    - TS `buildSelection(...)` in `webview-ui/src/lib/selection.ts`
    - Dart `_parseSelection(...)` and `PatchSelector.apply(...)`

