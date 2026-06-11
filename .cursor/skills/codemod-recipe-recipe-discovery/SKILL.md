---
name: codemod-recipe-recipe-discovery
description: Use when understanding or debugging how recipes are discovered (list/describe) and how recipe argument metadata (args, inputKind, options, allowCustomValue, contextKey, template previews) is serialized from Dart and rendered by the VS Code webview UI.
disable-model-invocation: true
---

# Recipe Discovery & Argument Metadata

## When to use

Use this skill when you are:
- Adding new args or metadata to a Dart `CodemodRecipe`.
- Debugging why the UI isn’t showing file/directory pickers, suggestions, or context prefill.
- Debugging recipe list refresh or “templates not loaded” behavior.
- Updating the serialized recipe schema for the VS Code extension/webview.

## Where discovery happens (extension → host)

### Recipe list
- Extension calls:
  - `DartBridge.list()` → host command `{ command: "list" }`
- Cache/staleness:
  - `vscode_extension/src/recipes/recipeRepository.ts`

Key files:
- `vscode_extension/src/host/dartBridge.ts` (`list()`)
- `vscode_extension/src/recipes/recipeRepository.ts` (refresh + `shouldRefresh`)
- `vscode_extension/src/extension.ts` (bootstrap/refresh flow)

### Recipe details (describe)
- Extension calls:
  - `DartBridge.describe(recipeId)`
  - host command `{ command: "describe", recipe: "<id>" }`
- View provider:
  - `vscode_extension/src/views/recipeRunnerViewProvider.ts`
  - `ensureRecipeDetails(...)` runs `bridge.describe` when `templatesLoaded !== false`

## Where discovery happens (host → Dart schema serialization)

- Host dispatch:
  - `lib/src/vscode/codemod_host.dart` (`dispatch` switch on `command`)
- Recipe serialization:
  - `lib/src/vscode/recipe_schema.dart`
    - `recipeEntryToJson(...)` and `recipeToJson(...)`

What `describe` returns:
- `id` + recipe metadata:
  - `name`, `description`
  - `args`: serialized `CodemodArg` descriptors
  - `templatesLoaded`: included when describing (template content optional)
  - `previewTemplates`: UI metadata for templates

## Arg metadata: `CodemodArg` fields and UI mapping

### Dart source of truth
- `CodemodArg` and `CodemodArgInputKind`:
  - `lib/src/recipe.dart` (definitions)

Key serialized fields (recipe schema):
- `name`, `abbr`, `help`, `required`, `defaultsTo` (wire string via `ArgCodec`)
- `inputKind` (includes `bool` for checkboxes)
- `options`
- `allowCustomValue`
- `contextKey`

Hidden/fixed args (`CodemodArg.fixed`, or `optional(..., hidden: true)`) are omitted from serialized `args`.

Typed authoring: `CodemodArg<T>` with `T` in `String | bool | int | double | Enum`. Use `CodemodArg<String>.required(...)` syntax.

Serialization logic:
- `lib/src/vscode/recipe_schema.dart`
  - `_inputKindToJson(...)` maps enum values → strings like `file`, `directory`, `enum`, etc.

### Extension/webview TypeScript types
- `vscode_extension/webview-ui/src/types.ts`
  - defines `RecipeArg` + `RecipeSchema` + `RecipeViewState`
- `vscode_extension/webview-ui/src/constants.ts`
  - tab names etc.

### UI behavior based on arg metadata

In the webview:
- The `RecipeArgForm` renders arg fields based on `recipe.args`
- Each arg field uses a computed “effective input kind”:
  - `vscode_extension/webview-ui/src/lib/args.ts`
  - `effectiveInputKind(arg)`:
    - if `arg.inputKind` exists and is not `text`, it uses that
    - otherwise it falls back to heuristics (`looksLikePath` based on arg name)

Where pickers/suggestions are enabled:
- `vscode_extension/webview-ui/src/components/ArgField.vue`
  - `inputKind` computed from `effectiveInputKind(...)`
  - shows datalist options when `arg.options?.length`
  - if `inputKind` is `file` or `directory`, shows “Browse…” button
  - if `inputKind` is `bool`, shows a checkbox (`'true'` / `'false'` wire values)

Context prefill:
- The command palette “Run From Cursor Context” prefills arguments using:
  - `vscode_extension/src/recipes/recipeContext.ts`

## Template previews (if/when used)

This repo includes template preview metadata in the Dart recipe schema:
- `lib/src/vscode/recipe_schema.dart` includes `previewTemplates`

If the UI begins rendering these:
- ensure the webview `RecipeSchema` type includes the fields you need
- mirror changes in any existing components that render template previews

## Checklist: adding new recipe arg metadata safely

1. Update Dart:
   - extend `CodemodArg` (or use existing fields)
   - ensure it is serialized in `lib/src/vscode/recipe_schema.dart`
2. Update extension/webview types:
   - `vscode_extension/webview-ui/src/types.ts`
3. Update UI rendering rules:
   - `vscode_extension/webview-ui/src/lib/args.ts` (`effectiveInputKind` heuristics if needed)
   - `vscode_extension/webview-ui/src/components/ArgField.vue`

## Red flags

- Changing the serialized key names in `recipe_schema.dart` without updating TS types (`RecipeArg`, `RecipeSchema`).
- Adding new `CodemodArgInputKind` values but not updating the webview `effectiveInputKind(...)` logic.
- Forgetting that the extension may use cached recipe list entries that omit template content unless it calls `describe`.

