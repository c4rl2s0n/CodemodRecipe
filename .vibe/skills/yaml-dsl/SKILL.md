---
name: yaml-dsl
description: Reference for the YAML DSL used by the Rust host (dslVersion 2). Covers edit/create/delete/recipe steps, tree-sitter query ops, templates, maps, and validation.
user-invocable: false
allowed-tools: ["read", "grep"]
---

# YAML DSL v2 Reference (Rust Host)

> **Bootstrap source of truth:** `export/.agents/skills/codemod-yaml-dsl-v2/` (SKILL.md + reference.md).
> This file is kept for repo development; prefer the export skill for agent-facing docs.

This skill documents the YAML recipe DSL consumed by the Rust pipeline:

- `rust/crates/yaml/src/model.rs`
- `rust/crates/yaml/src/validate.rs`
- `rust/crates/yaml/src/compose.rs`
- `rust/crates/host/src/runner.rs`

Use this as the source of truth when writing or reviewing `.codemod/recipes/*.yaml`.

## DSL Version

Use:

```yaml
dslVersion: 2
```

The Rust model does not require a `dslVersion` field at parse time, but all
repository recipes and docs use `dslVersion: 2` as the canonical format.

## Top-level recipe structure

```yaml
dslVersion: 2
id: recipe_id
name: Human-readable name
description: What this recipe does

args: []
maps: {}
steps: []
postExecution: []
```

### Top-level fields

- `id` (required in practice): unique recipe id
- `name` (optional): display name
- `description` (optional)
- `args` (optional): list of argument definitions
- `maps` (optional): recipe-local map entries
- `steps` (required): ordered list of operations
- `postExecution` (optional): post-apply actions (e.g. `dartFormat`)

## Arguments (`args`)

Each arg object supports:

- `name` (required)
- `required` (bool, default false)
- `inputKind` (optional string, e.g. `file`, `symbol`)
- `abbr`, `help`, `defaultsTo`, `options`, `allowCustomValue`, `contextKey`

Example:

```yaml
args:
  - name: file
    required: true
    inputKind: file
  - name: className
    required: true
    inputKind: symbol
```

## Step types

Each entry in `steps` must be a single-key object of one of:

- `edit`
- `create`
- `delete`
- `recipe` (recipe composition/reference)

## Edit steps (`edit`)

### Shape

```yaml
- edit:
    path: "lib/file.dart"
    ops:
      - insert: { ... }
      - replace: { ... }
      - remove: { ... }
```

- `path`: file path (often templated, e.g. `{{file}}`)
- `ops`: list of edit operations (must not be empty)

### Edit operation types

#### `insert`

Required fields:
- `query` (tree-sitter query string)
- `capture` (capture name used as insertion target)
- `anchor` (`start` or `end`)
- `text` (inserted text)

Example: [`.codemod/recipes/add_log_line.yaml`](../../../.codemod/recipes/add_log_line.yaml)

#### `replace`

Required fields:
- `query`
- `capture`
- `text`

Optional fields:
- `includeLeadingTrivia` (bool, default false)

Example: [`test/fixtures/rust_oracle/replace_count_field.recipe.yaml`](../../../test/fixtures/rust_oracle/replace_count_field.recipe.yaml)

#### `remove`

Required fields:
- `query`
- `capture`

Optional fields:
- `includeLeadingTrivia` (bool, default false)

Example: [`test/fixtures/rust_oracle/remove_count_field.recipe.yaml`](../../../test/fixtures/rust_oracle/remove_count_field.recipe.yaml)

### Query + capture semantics

- `query` is a tree-sitter query and may define multiple captures (`@className`, `@body`, etc.).
- `capture` chooses which captured node span is edited.
- If the named capture is missing, unmatched, or ambiguous, execution fails.

## Create steps (`create`)

### Shape

```yaml
- create:
    path: "lib/new_file.dart"
    template: "class X {}"
    ifExists: fail
    format: true
```

or:

```yaml
- create:
    path: "lib/new_file.dart"
    templateFile: templates/new_file.dart.template
    ifExists: fail
    format: false
```

Rules:
- `path` required
- Exactly one of `template` or `templateFile`
- `ifExists`: `fail` (default) or `skip`
- `format`: bool (default true)

Example: [`test/fixtures/scaffold_project/.codemod/recipes/create_repository.yaml`](../../../test/fixtures/scaffold_project/.codemod/recipes/create_repository.yaml)

## Delete steps (`delete`)

### Shape

```yaml
- delete:
    path: "lib/legacy/stale.dart"
    ifMissing: skip
```

Rules:
- `path` required
- `ifMissing`: `fail` (default) or `skip`

Example: [`test/fixtures/rust_oracle/delete_legacy.recipe.yaml`](../../../test/fixtures/rust_oracle/delete_legacy.recipe.yaml)

## Recipe composition (`recipe`)

Reference another recipe by id:

```yaml
steps:
  - recipe: create_repository
  - recipe: patch_counter
  - recipe: patch_app
```

Composition behavior (Rust YAML compose):
- referenced recipe steps are expanded in order
- args are merged by name (first definition wins)
- create/edit/delete steps are inlined
- recipe cycles are rejected

Example: [`test/fixtures/scaffold_project/.codemod/recipes/scaffold_feature.yaml`](../../../test/fixtures/scaffold_project/.codemod/recipes/scaffold_feature.yaml)

## Maps and templates

## Recipe-local maps

Define local maps:

```yaml
maps:
  field_kind:
    tickCount: int
    label: String
```

Example: [`test/fixtures/scaffold_project/.codemod/recipes/patch_counter.yaml`](../../../test/fixtures/scaffold_project/.codemod/recipes/patch_counter.yaml)

## Workspace maps

You can also load shared maps from `.codemod/maps/*.yaml`. These merge with
recipe-local maps.

## Template syntax

Supported in paths, queries, and text:

- `{{argName}}`
- `{{$camel argName}}`
- `{{$snake argName}}`
- `{{$pascal argName}}`
- `{{$map 'mapId' keyArg}}`

Examples:
- [`.codemod/recipes/add_counter_field.yaml`](../../../.codemod/recipes/add_counter_field.yaml)
- [`test/fixtures/scaffold_project/.codemod/recipes/patch_counter.yaml`](../../../test/fixtures/scaffold_project/.codemod/recipes/patch_counter.yaml)

## Validation rules (Rust)

From `rust/crates/yaml/src/validate.rs`:

- Recipe must have `steps`
- Arg names must be unique
- Unsupported step kinds are errors
- `edit.path` required
- `edit.ops` must be non-empty
- `insert.query` and `insert.capture` required
- `replace.query`, `replace.capture`, `replace.text` required
- `remove.query`, `remove.capture` required
- `create.path` required
- `create` must have exactly one of `template` or `templateFile`
- `delete.path` required

## Practical authoring checklist

1. Start from a known-good example recipe.
2. Keep query captures explicit and stable.
3. Use `preview_recipe` before `apply_recipe`.
4. Require arguments for all dynamic values.
5. Re-run preview after apply and expect no further changes for idempotent recipes.

## Canonical examples

- Insert op: [`.codemod/recipes/add_log_line.yaml`](../../../.codemod/recipes/add_log_line.yaml)
- Insert with map/casing helper: [`test/fixtures/scaffold_project/.codemod/recipes/patch_counter.yaml`](../../../test/fixtures/scaffold_project/.codemod/recipes/patch_counter.yaml)
- Create step: [`test/fixtures/scaffold_project/.codemod/recipes/create_repository.yaml`](../../../test/fixtures/scaffold_project/.codemod/recipes/create_repository.yaml)
- Remove step: [`test/fixtures/rust_oracle/remove_count_field.recipe.yaml`](../../../test/fixtures/rust_oracle/remove_count_field.recipe.yaml)
- Replace step: [`test/fixtures/rust_oracle/replace_count_field.recipe.yaml`](../../../test/fixtures/rust_oracle/replace_count_field.recipe.yaml)
