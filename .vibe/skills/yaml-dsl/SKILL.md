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

Optional **`when` / `whenNot`** (query guards) and **`let`** (per-op locals for templates) — see export skill `codemod-yaml-dsl-v2` reference.

Steps and ops apply **sequentially** against an in-memory working tree (later steps see
earlier mutations on the same path). Same-path `create` then `edit` and dependent
`edit` then `edit` are supported. See export skill `codemod-yaml-dsl-v2` reference
(Sequential staging).

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

Call-site bindings (`with`) forward, remap, or hardcode child args. Values are
templates in the parent context; unbound child args still fall through by name.
`with` only applies to the directly referenced recipe (each recipe owns its own
children); unknown keys error with `E_RECIPE_WITH`.

```yaml
steps:
  - recipe:
      id: create_repository
      with:
        className: "{{ featureName }}"
```

Composition behavior (Rust YAML compose):
- referenced recipe steps are expanded in order
- args listed in `with` are not unioned into the parent
- unbound child args are merged by name (first definition wins)
- create/edit/delete steps are inlined (scoped when `with` is non-empty)
- recipe cycles are rejected
- unknown `with` keys (not in the child’s declared `args`) are errors

Example: [`test/fixtures/scaffold_project/.codemod/recipes/scaffold_feature.yaml`](../../../test/fixtures/scaffold_project/.codemod/recipes/scaffold_feature.yaml)
Bindings: [`test/fixtures/jinja_examples/.codemod/recipes/with_bind_orchestrator.yaml`](../../../test/fixtures/jinja_examples/.codemod/recipes/with_bind_orchestrator.yaml)

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

## Workspace maps and variables

Shared maps (`id` + `map:`) and variables (`id` + `values:`) are discovered by
schema anywhere under `.codemod/` (dirs like `maps/` / `variables/` are convention).
Maps merge with recipe-local `maps:`.

## Template syntax (Jinja2 / MiniJinja)

Canonical syntax for Rust host recipes. Full guide: [`docs/recipe-templates.md`](../../../docs/recipe-templates.md).

Supported in paths, queries, inline `create.template`, and `create.templateFile`:

- `{{ argName }}`
- `{{ arg | camel_case }}`, `snake_case`, `pascal_case`, `lower`, `upper`, `screaming_snake`, `kebab_case`
- `{{ keyArg | map('mapId') }}` or `{{ map.mapId[keyArg] }}`
- `{{ var.varId.key }}` for shared variables (`id` + `values:` YAML under `.codemod/`)

- `{% if includeTests %}…{% endif %}` (bool args as `"true"` / `"false"`)

Legacy `{{$camel x}}` helpers still work via a shim but emit `W_LEGACY_TEMPLATE` — prefer Jinja filters.

Examples:
- [`.codemod/recipes/add_counter_field.yaml`](../../../.codemod/recipes/add_counter_field.yaml)
- [`.codemod/recipes/conditional_create.yaml`](../../../.codemod/recipes/conditional_create.yaml)
- [`test/fixtures/scaffold_project/.codemod/recipes/patch_counter.yaml`](../../../test/fixtures/scaffold_project/.codemod/recipes/patch_counter.yaml)
- [`test/fixtures/jinja_examples/`](../../../test/fixtures/jinja_examples/) — full feature showcase

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
- Create with conditional template: [`.codemod/recipes/conditional_create.yaml`](../../../.codemod/recipes/conditional_create.yaml)
- Jinja feature showcase: [`test/fixtures/jinja_examples/`](../../../test/fixtures/jinja_examples/)
- Remove step: [`test/fixtures/rust_oracle/remove_count_field.recipe.yaml`](../../../test/fixtures/rust_oracle/remove_count_field.recipe.yaml)
- Replace step: [`test/fixtures/rust_oracle/replace_count_field.recipe.yaml`](../../../test/fixtures/rust_oracle/replace_count_field.recipe.yaml)
