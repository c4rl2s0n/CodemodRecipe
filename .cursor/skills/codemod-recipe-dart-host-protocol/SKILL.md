---
name: codemod-recipe-dart-host-protocol
description: Use when you need to change or debug the Dart JSON-over-stdio protocol between the VS Code extension and the `codemod_host` process. Covers supported commands, stdout framing markers, persistent `--stdio-server` mode, preview cache behavior, and selection semantics.
disable-model-invocation: true
---

# codemod_recipe Dart Host Protocol

## When to use

Use this skill when:
- You are adding/changing host commands (e.g. preview/diff/apply semantics).
- You see “failed to parse host response” or “preview failed” symptoms.
- You need to update payload/response shapes while keeping TS unions in sync.
- You’re troubleshooting preview ordering / stale results behavior.

## Where to look (Dart host side)

- Host entrypoint + command dispatch:
  - `lib/src/vscode/codemod_host.dart` (`CodemodHost.run`, `dispatch`, `_preview`, `_diff`, `_apply`)
- stdout framing markers:
  - `lib/src/vscode/codemod_host.dart`
    - `kResultBegin = '__CODEMOD_RESULT_BEGIN__'`
    - `kResultEnd = '__CODEMOD_RESULT_END__'`
- recipe serialization used by `describe`:
  - `lib/src/vscode/recipe_schema.dart`
- diff + patch selection helpers:
  - `lib/src/vscode/diff_service.dart`
  - `lib/src/vscode/patch_selector.dart`

## Where to look (TypeScript host client side)

- Message protocol markers + stdout frame extraction:
  - `vscode_extension/src/host/hostProtocol.ts`
    - `extractHostResultFrame(...)`
    - `parseHostResponse<T>(...)`
- Persistent host lifecycle + JSON command queue:
  - `vscode_extension/src/host/dartBridge.ts`
    - `sendPersistent(...)` (queued writes to stdin)
    - `flushFrames()` (parses framed stdout and resolves the next pending request)
    - `sendOneShot(...)` (spawns `dart run entrypoint` and reads stdout)

## Supported commands (host dispatch)

All commands are single JSON objects sent over stdin; the host returns one JSON response wrapped in stdout markers.

### `list`
- Input: `{ "command": "list" }`
- Output: `{ ok: true, recipes: [...] }` or `{ ok: false, error: string }`

### `describe`
- Input: `{ "command": "describe", "recipe": "<id>" }`
- Output: `{ ok: true, recipe: <recipe metadata> }`
- Recipe metadata includes args + optional template previews, but omits operations (closures).
- See: `lib/src/vscode/recipe_schema.dart` (`recipeEntryToJson`, `registryToJson`)

### `preview`
- Input:
  - `{ "command": "preview", "recipe": "<id>", "args": {..}, "snippetLines"?: <number> }`
- Output:
  - `{ ok: true, recipe: "<id>", files: <FilePreview[]> }`
  - plus `_timingsMs` metrics (used only for logging)
- Preview serialization uses:
  - `lib/src/vscode/diff_service.dart` with `includeContents: false` and `includePatchReplacements: false`
  - It emits `patches[]` with `replacementPreview` (for UI snippets).

### `diff`
- Input:
  - `{ "command": "diff", "recipe": "<id>", "args": {..}, "path": "<filePath>" }`
- Output:
  - `{ ok: true, recipe: "<id>", file: <FilePreview> }` where `file` includes `original`/`modified`
- Extension requests `diff` only when it needs native side-by-side content.
- See extension materialization:
  - `vscode_extension/src/views/recipeRunnerViewProvider.ts` (`ensureDiffMaterialized`)

### `apply`
- Input:
  - `{ "command": "apply", "recipe": "<id>", "args": {..}, "selection": { "files": { ... } } }`
- Output:
  - `{ ok: true, recipe: "<id>", applied: [<path>, ...] }`
- Applies:
  - collects changes with cache
  - selects subset via `PatchSelector.apply`
  - runs recipe `postExecution` actions only when `--apply` succeeds
- See: `lib/src/vscode/codemod_host.dart` (`_apply`) and `patch_selector.dart`

## Persistent mode (`--stdio-server`)

If host is launched with `--stdio-server`:
- it reads stdin continuously, one JSON line at a time
- for each line:
  - `_decodeRequest(line)`
  - `_handleRequest(request, fallbackCommand: line)`
  - writes a framed JSON response for that one command

Where:
- `lib/src/vscode/codemod_host.dart`
  - `_runPersistent()` uses a `LineSplitter`

On the extension side:
- `vscode_extension/src/host/dartBridge.ts` uses `spawn(dart, ['run', entrypoint, '--stdio-server'])`

## stdout framing contract (critical)

Protocol is resilient to extra stdout (e.g. `dart format` output) by using explicit markers:
- Host writes:
  - `__CODEMOD_RESULT_BEGIN__`
  - `<json>`
  - `__CODEMOD_RESULT_END__`
- Extension parses:
  - `extractHostResultFrame()` to identify each `{ payload, rest }`
  - `flushFrames()` to match frames to pending requests (FIFO).

Do not change marker strings without updating:
- `lib/src/vscode/codemod_host.dart` (kResultBegin/kResultEnd)
- `vscode_extension/src/constants.ts` (`HOST_PROTOCOL.resultBegin/resultEnd`)
- `vscode_extension/src/host/hostProtocol.ts` (frame extraction)

## Preview cache behavior (critical)

The host caches preview calculations keyed by:
- recipe id
- normalized args map (keys sorted; values stringified)

Where:
- `lib/src/vscode/codemod_host.dart`
  - `_previewCache` is a `Map<String, _CachedPreview>`
  - `_cacheKey(request)` normalizes args and formats: `'$recipeId:<jsonNormalizedArgs>'`

Cache validity:
- on reuse, it compares file snapshots (exists, modifiedMs, size) for each cached file path
- `_isCacheValid(cached)` iterates snapshots and calls `_snapshotForPath(path)`

This means:
- preview cache reuse can become invalid when files change between requests.

## Selection semantics (apply payload)

`apply` receives a `selection.files` map keyed by file path.

Each file selection supports:
- `include` (boolean; default semantics are handled by host decoding)
- optional `patches` (list of patch indices for patch-based changes)

Where:
- TS selection payload:
  - `vscode_extension/src/types.ts` (`SelectionPayload`, `FileSelection`)
- Dart selection parsing:
  - `lib/src/vscode/codemod_host.dart` (`_parseSelection`)
  - `lib/src/vscode/patch_selector.dart` (`FileSelection.fromJson`)

Common pitfall:
- patch indices are indices into the host-generated patch list order (not stable ids).

## Red flags (stop and fix)

- Changing JSON payload fields without updating BOTH sides:
  - Dart response/request parsing
  - TS types + message unions
- Modifying framing markers without updating TS frame extraction.
- Removing FIFO mapping between parsed frames and `pending` queue in `DartBridge.flushFrames()`.
- Adding commands that return data out of order without adding a request correlation strategy.
- Changing cache key normalization rules without understanding that UI preview ordering depends on consistent preview results per args key.

## Implementation workflow (safe changes)

1. Update the Dart request/response contract in `lib/src/vscode/codemod_host.dart` (+ any helper types).
2. Update `vscode_extension/src/types.ts` if shared shapes changed.
3. Update protocol constants/unions:
   - `vscode_extension/src/constants.ts`
   - `vscode_extension/src/views/recipeRunnerMessages.ts` and `vscode_extension/webview-ui/src/messages.ts`
4. Update webview controller logic for request ordering if preview/diff/apply flows changed:
   - `vscode_extension/webview-ui/src/composables/useRunnerController.ts`

