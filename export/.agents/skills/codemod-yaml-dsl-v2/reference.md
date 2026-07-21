# YAML DSL v2 Reference

Recipes live in `.codemod/recipes/*.yaml` and use **dslVersion 2** with tree-sitter queries.

Rust implementation: `rust/crates/yaml/src/model.rs`, `validate.rs`, `compose.rs`.

## DSL version

```yaml
dslVersion: 2
```

All recipes should declare `dslVersion: 2`.

## Top-level structure

```yaml
dslVersion: 2
id: recipe_id
name: Human-readable name
description: What this recipe does

args: []
maps: {}
steps: []
postExecution:
  - dartFormat
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

### `insert`

Required fields:

- `query` (tree-sitter query string)
- `capture` (capture name used as insertion target)
- `anchor` (`start` or `end`)
- `text` (inserted text)

### `replace`

Required: `query`, `capture`, `text`

Optional: `includeLeadingTrivia` (bool, default false)

### `remove`

Required: `query`, `capture`

Optional: `includeLeadingTrivia` (bool, default false)

### Query + capture semantics

- `query` is a tree-sitter query and may define multiple captures (`@className`, `@body`, etc.).
- `capture` chooses which captured node span is edited.
- If the named capture is missing, unmatched, or ambiguous, execution fails.

Example insert query:

```yaml
query: |
  (class_declaration
    name: (identifier) @className
    body: (class_body
      (class_member
        (method_signature
          (function_signature
            name: (identifier) @methodName))
        (function_body
          (block) @body)))
    (#eq? @className "{{className}}")
    (#eq? @methodName "{{methodName}}"))
capture: body
anchor: end
text: "    print('codemod');\n"
```

## Create steps (`create`)

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

## Delete steps (`delete`)

```yaml
- delete:
    path: "lib/legacy/stale.dart"
    ifMissing: skip
```

Rules:

- `path` required
- `ifMissing`: `fail` (default) or `skip`

## Recipe composition (`recipe`)

Reference another recipe by id:

```yaml
steps:
  - recipe: create_repository
  - recipe: patch_counter
  - recipe: patch_app
```

Composition behavior:

- Referenced recipe steps are expanded in order
- Args are merged by name (first definition wins)
- Create/edit/delete steps are inlined
- Recipe cycles are rejected

## Maps

### Recipe-local maps

```yaml
maps:
  field_kind:
    tickCount: int
    label: String
```

### Workspace maps

Load shared maps from `.codemod/maps/*.yaml`. These merge with recipe-local maps.

Example workspace map file:

```yaml
id: field_kind
entries:
  tickCount: int
  label: String
```

Use in recipe text:

```yaml
text: "  final {{$map 'field_kind' fieldName}} {{$camel fieldName}} = 0;\n\n"
```

## Template syntax

Supported in paths, queries, and text:

- `{{argName}}`
- `{{$camel argName}}`
- `{{$snake argName}}`
- `{{$pascal argName}}`
- `{{$map 'mapId' keyArg}}`

Example path: `lib/features/{{$snake feature}}/{{$snake feature}}_state.dart`

Example typed field insert:

```yaml
text: "  final {{$map 'field_kind' fieldName}} {{$camel fieldName}};\n\n"
```

## Validation rules

Call `validate_recipes` after editing YAML.

- Recipe must have `steps`
- Arg names must be unique
- Unsupported step kinds are errors
- `edit.path` required; `edit.ops` must be non-empty
- `insert`: `query` and `capture` required
- `replace`: `query`, `capture`, `text` required
- `remove`: `query`, `capture` required
- `create.path` required; exactly one of `template` or `templateFile`
- `delete.path` required

Common errors: empty `ops`, missing `capture`, duplicate arg names.

## Practical checklist

1. Start from a known-good example recipe in `.codemod/recipes/`.
2. Keep query captures explicit and stable.
3. Use `preview_recipe` before `apply_recipe`.
4. Require arguments for all dynamic values.
5. Re-run preview after apply; expect no further changes for idempotent recipes.

## Related skills

- `codemod-recipe-design-patterns` — create vs modify taxonomy
- `codemod-recipe-authoring` — tree-sitter query patterns
- `codemod-mcp-playbook` — preview/apply workflow
