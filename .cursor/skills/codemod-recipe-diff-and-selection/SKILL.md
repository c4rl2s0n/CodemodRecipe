---
name: codemod-recipe-diff-and-selection
description: Use when implementing or debugging how the extension renders previews/diffs and how user selections (file include + patch indices) map to Dart `PatchSelector.apply`. Covers preview vs diff payload differences and the `include`/`patches` semantics across webview and host.
disable-model-invocation: true
---

# Diff Rendering & Patch Selection Semantics

## When to use

Use this skill when you:
- See UI selection not matching applied changes
- Debug “Apply Selected” behavior dropping wrong files/patches
- Need to understand why preview snippets differ from native diffs
- Change selection payload shape or how patch indices are interpreted

## Preview vs diff: what’s different

### Preview request/response

- Extension receives `WEBVIEW_TO_EXTENSION.preview`
  - `vscode_extension/src/views/recipeRunnerViewProvider.ts`
- It calls:
  - `DartBridge.preview(...)` → host command `{ command: "preview" }`

Host preview serialization:
- `lib/src/vscode/codemod_host.dart` `_preview(...)`
- It calls `DiffService.changesToJson(...)` with:
  - `includeContents: false`
  - `includePatchReplacements: false`

Implication:
- Preview payload includes patch metadata suitable for snippets:
  - `patches[].replacementPreview` (not the full replacement text)
- UI displays compact snippet text; full original/modified text is not sent.

### Diff request/response (materializing full contents)

- Extension receives `WEBVIEW_TO_EXTENSION.openDiff`
  - `openDiffByPath(...)` calls `ensureDiffMaterialized(file)`
- It calls:
  - `DartBridge.diff(...)` → host command `{ command: "diff" }`

Host diff serialization:
- `lib/src/vscode/codemod_host.dart` `_diff(...)`
- `DiffService.changeToJson(...)` includes full contents by default

Implication:
- Diff payload includes `original` and `modified` fields
- Diff rendering can show correct native side-by-side text

## User selection model (webview)

Selection is built in the webview and sent on apply.

- Webview selection state:
  - `vscode_extension/webview-ui/src/lib/selection.ts`
    - `FileCardSelection`, `PatchSelection`

Data model:
- `FileCardSelection` contains:
  - `path`
  - `include` (include entire file change)
  - `patches[]` entries with:
    - `index` (patch index)
    - `include` (include that patch)

Special case: whole-file changes
- When a `FilePreview` has `patches.length === 0`, the UI treats it as a
  “whole file change” using a synthetic patch row with `patch.index = -1`.
- See:
  - `defaultFileSelections(...)` in `selection.ts`
  - `FileCard.vue` uses `patch.index < 0` to render “Whole-file change”

## Building the `apply` payload

When user clicks “Apply Selected”:
- webview calls `buildSelection(fileSelections)`
  - `vscode_extension/webview-ui/src/lib/selection.ts`

`buildSelection` semantics:
- For each file in `fileSelections`:
  - `include` is taken from `FileCardSelection.include`
  - `patches` is included only if there are patch toggles with `index >= 0`
  - When a patch list is omitted, host keeps all patches for that file.

This omission behavior is important.

## Dart host apply mapping (PatchSelector)

Host parsing:
- `lib/src/vscode/codemod_host.dart` `_parseSelection(...)`
  - reads `selection['files']` map into `FileSelection`

Selection shape:
- Dart `FileSelection` in `lib/src/vscode/patch_selector.dart`
  - `include` boolean
  - optional `patchIndices` list (nullable)

Filtering/apply:
- `PatchSelector.apply(changes, selection)`:
  - if `include` is false, the file change is dropped
  - if `patchIndices` is null/omitted:
    - keep all patches for that patch-based file change
  - if patchIndices is provided:
    - keep only patches at those indices

## Where patch indices come from

Patch indices are NOT stable ids; they’re positional:
- They refer to the order of the host-generated patch list for that file.

Host generation order comes from:
- patch order produced by transforms during preview/collect
- merged patch lists for a file (runner merge logic)

Therefore:
- You should not reorder patch lists in the UI independently.
- Always treat `patches[]` array order as authoritative.

## Red flags (stop and fix)

- “Apply Selected” drops nothing / everything:
  - likely wrong `include` or patch indices being sent
  - check `buildSelection(...)` in webview and `_parseSelection(...)` in host
- “Patch selection includes wrong patches”:
  - patch index mismatch due to reordering
  - verify UI doesn’t mutate patch arrays
- Whole-file changes treated like patch-based changes:
  - ensure synthetic index `-1` never leaks into Dart patchIndices
  - `buildSelection` should only send patches with `index >= 0`
- Preview UI expects `replacement` but payload only contains `replacementPreview`:
  - preview uses `replacementPreview`; diff needs `replacement`/full contents

