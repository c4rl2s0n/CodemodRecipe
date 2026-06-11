---
name: codemod-recipe-tool-usage
description: Use when helping users run codemod_recipe recipes (CLI or VS Code extension). Covers how to define a recipe, run previews/applies, and wire a Dart host entrypoint for the extension.
disable-model-invocation: true
---

# Codemod Recipe Tool Usage

## When to use

Use this skill when the user asks:
- “How do I run a recipe?” / “How do I preview vs apply?”
- “How do I expose my recipes to the VS Code extension?”
- “Why does the extension say no host entrypoint found?”
- “How do I add argument metadata (file picker, suggestions, context prefill)?”

## Quick pointers (source of truth)

- Package overview and concepts: `README.md`
- VS Code extension usage + dev workflow: `vscode_extension/README.md`
- CLI runner: `lib/src/runner.dart`
- Recipe definition primitives: `lib/src/recipe.dart`
- VS Code host entrypoint + protocol: `lib/src/vscode/codemod_host.dart`
- Example host: `example/vscode_host_example/bin/codemod_host.dart`

## CLI usage (headless)

### Define a recipe

- Core type: `CodemodRecipe` (`lib/src/recipe.dart`)
- Run it via: `CodemodRunner(recipe).run(args)` (`lib/src/runner.dart`)

Example pattern is shown in `README.md` and `example/add_method_example/bin/add_method.dart`.

### Preview vs apply

- Default run is **dry-run** (preview printed).
- Use `--apply` (or `-a`) to write changes and run post-execution actions.

Runner behavior is implemented in `lib/src/runner.dart`:
- `collectChanges(context)` builds and merges file changes
- in dry-run it prints previews
- in apply it calls `change.apply()` then runs `postExecution`

## VS Code Extension usage (GUI)

### How it works (one sentence)

The extension launches a Dart **host** and sends JSON commands over stdio (persistent `--stdio-server` when possible), then renders preview/diff/apply results in a Vue webview.

### Setup: provide a host entrypoint

You must add a small Dart program in the **target workspace** that registers recipes and forwards args to `CodemodHost`.

See:
- docs: `vscode_extension/README.md`
- example host: `example/vscode_host_example/bin/codemod_host.dart`

Minimal pattern:
- import `package:codemod_recipe/codemod_recipe_vscode.dart`
- `CodemodHost.fromList([...recipes...]).run(args)`

### Host entrypoint discovery

If `codemodRecipe.hostEntrypoint` is empty, the extension tries common candidates:
`tool/codemod_host.dart`, `tool/codemods/codemod_host.dart`, `bin/codemod_host.dart`, etc.

See: `vscode_extension/src/constants.ts` (`DEFAULT_HOST_CANDIDATES`) and `vscode_extension/src/host/hostDiscovery.ts`.

### Daily usage flow (end user)

From `vscode_extension/README.md`:
- Open **Codemod Recipe** view
- Pick recipe in **Recipes** tab
- Fill parameters in **Recipe Runner**
- Preview updates automatically as args change
- Review per-file and per-patch checkboxes
- Navigate changes (Previous/Next or click patch)
- Apply selected patches

### Cursor-context shortcut

Command: **Codemod Recipe: Run From Cursor Context**
- Picks recipes whose args declare `contextKey`
- Prefills values from the active editor

Entry point: `vscode_extension/src/extension.ts`

## Argument UX metadata (for better UI)

In recipe args (`CodemodArg`), you can set:
- `inputKind`: file/directory/enum/dartType/symbol to influence UI control
- `options`: suggestions for combobox-like inputs
- `allowCustomValue`: restrict to options if false
- `contextKey`: enable “run from cursor context” prefills

See:
- `lib/src/recipe.dart` (`CodemodArgInputKind`, `CodemodContextKey`, `CodemodArg`)
- schema serialization for the extension: `lib/src/vscode/recipe_schema.dart`

## Troubleshooting checklist

### “No codemod host entry point found”

- Ensure a host file exists at a default candidate path OR set `codemodRecipe.hostEntrypoint` (workspace setting).
- Ensure `dart` executable path is correct (`codemodRecipe.dartPath`).
- See: `vscode_extension/src/host/hostDiscovery.ts`, `vscode_extension/src/config/extensionConfig.ts`.

### “Preview failed” / “Host error”

- The extension uses a JSON-over-stdio protocol.
- The host wraps its JSON output in markers to avoid corruption by stdout noise (post-execution output, formatting).

See:
- Dart framing markers: `lib/src/vscode/codemod_host.dart` (`kResultBegin`, `kResultEnd`)
- TS parsing: `vscode_extension/src/host/hostProtocol.ts` and `vscode_extension/src/host/dartBridge.ts`

### “Diff opens but shows empty content”

- Preview results may omit full contents; diff requests materialize `original`/`modified` from the host.
- See: `vscode_extension/src/views/recipeRunnerViewProvider.ts` (`ensureDiffMaterialized`).

## Red flags

- Editing `vscode_extension/media/*` directly (these are built artifacts).
- Changing protocol JSON shapes without updating TypeScript types (`vscode_extension/src/types.ts`) and webview types (`vscode_extension/webview-ui/src/types.ts`).
- Adding new arg `inputKind` values without updating the schema mapping in `lib/src/vscode/recipe_schema.dart` and webview rendering logic.

