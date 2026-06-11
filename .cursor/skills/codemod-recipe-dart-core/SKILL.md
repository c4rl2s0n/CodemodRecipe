---
name: codemod-recipe-dart-core
description: Use when implementing or modifying the Dart `codemod_recipe` package (core APIs: recipes, runner, context, operations, patches, transforms, templates, post-execution). Provides the main abstractions, where patch merging happens, and safe extension points.
disable-model-invocation: true
---

# codemod_recipe Dart Core

## When to use

Use this skill when you are working on:
- The Dart package API surface (`lib/` / `lib/src/`)
- How recipes/operations/transforms generate changes
- Patch semantics (ordering, overlap validation, previews)
- Runner behavior (dry-run vs apply) and post-execution actions

## Public API entry points

- Main library exports: `lib/codemod_recipe.dart`
- VS Code host exports: `lib/codemod_recipe_vscode.dart`

## Core concepts (data flow)

From `README.md`:

`CLI args -> CodemodContext -> CodemodRecipe -> CodemodOperation -> FileChange -> preview/apply -> PostExecution`

## Key types and where they live

### `CodemodRecipe`

- File: `lib/src/recipe.dart`
- Responsibilities:
  - declares args (`CodemodArg`)
  - declares ordered operations (`CodemodOperation`)
  - declares post-execution actions (`PostExecution`)
  - optionally declares template previews (`RecipeTemplatePreview`)
  - supports composition (`CodemodRecipe.compose(...)`)

### `CodemodContext`

- File: `lib/src/context.dart`
- Responsibilities:
  - stores raw named values (`set/get/require`)
  - naming helpers: `snake`, `camel`, `pascal` (backed by `lib/src/dart/naming.dart`)
  - template rendering convenience: `render(...)`

Preferred extension point:
- Add project-specific context helpers via Dart `extension` on `CodemodContext` (see `README.md` and `lib/src/context.dart` docs).

### `CodemodRunner`

- File: `lib/src/runner.dart`
- Responsibilities:
  - parse CLI options for recipe args
  - collect planned file changes
  - dry-run preview vs `--apply` behavior
  - execute post-execution actions after successful apply

Important behavior:
- `collectChanges(context)` is exposed so non-CLI frontends (like the VS Code host) can reuse the exact collection + merge logic.

### Operations & file changes

- File: `lib/src/operation.dart`

Core interfaces:
- `CodemodOperation.collect(context) -> List<FileChange>`
- `FileChange` has:
  - `path`
  - `hasChanges`
  - `shouldFormat`
  - `preview()`
  - `apply()`

Built-in operations:
- `EditDartFileOperation` (patch-based edits)
- `CreateFileOperation` (template-based generation) including `templatePath(...)`

Built-in file change implementations:
- `PatchFileChange`: applies a list of `SourcePatch` to `source`
- `CreateFileChange`: writes full file content (or skip strategy)

### Transforms

- Transform interface: `lib/src/transform.dart` (`CodeTransform.apply`)
- Patch helpers / patch model: `lib/src/patch_helpers.dart` (`SourcePatch`, overlap validation, apply order)

Important invariants:
- Patches must be non-overlapping (`validateNonOverlappingPatches`)
- Patch application order is stable and offset-safe (typically end-to-start)

### Templates

- Files: `lib/src/template.dart` (rendering), `lib/src/context.dart` convenience `render(...)`
- Placeholders support casing filters mentioned in `README.md`: `snake`, `camel`, `pascal`
- Missing variables and unsupported placeholders should fail (deterministic behavior).

### Post-execution

- Interfaces and result: `lib/src/post_execution.dart`
- Some built-ins are under: `lib/src/generic/post_execution/*`
- Contract:
  - post-execution runs only after apply
  - receives `CodemodContext` + `CodemodRunResult` (with `formattablePaths`)

## Patch merging behavior (critical)

Patch merging is performed by the runner:
- File: `lib/src/runner.dart`
- Method: `_mergePatchChanges(...)`

Rules enforced:
- Cannot combine patch and full-file changes for the same path
- Cannot have multiple full-file changes for the same path
- Patch file changes targeting the same file are merged, then validated for overlap

If you change operation behavior, ensure you don’t accidentally violate these invariants.

## Common extension patterns

- Prefer new operations/transforms rather than adding project conventions into the core package.
- Keep naming/path conventions in consuming projects via `CodemodContext` extensions.
- For new “generic” transforms, keep them deterministic and testable (pure patch generation).

## Red flags

- Introducing nondeterminism (timestamps, random ids) into transforms/templates.
- Allowing silent template rendering with missing variables.
- Changing patch ordering/overlap rules without updating tests and preview/diff serialization expectations.
- Mixing full-file and patch changes for a single path (runner treats this as an error).

