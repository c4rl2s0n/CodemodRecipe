# YAML DSL Reference

Recipes live in `.codemod/recipes/*.yaml` and use tree-sitter queries under `edit.ops`.

Rust implementation: `rust/crates/yaml/src/model.rs`, `validate.rs`, `compose.rs`.

## Rust ownership notes

For Rust maintenance, recipe/YAML vocabulary is centralized instead of being scattered across host and engine helpers:

- `rust/crates/yaml/src/dsl/`
  - schema-aligned modules: `recipe`, `map_asset`, `variables_asset` (step/op field trees mirror `recipe.schema.json` `$defs`)
- `rust/crates/yaml/src/dsl_vocabulary.rs`
  - `ENTRIES`: author descriptions, optional `schema_path`, enum parents; references `crate::dsl::…` wires
  - `keyword_docs_json()`, `description_for_key`, `description_for_enum` for tooling
- `rust/crates/yaml/src/keywords.rs`
  - re-exports `crate::dsl`; `preview_kinds` uses step `WIRE` constants
  - `query_conventions`: shared query path detection for `.scm` file-backed queries and id-based query-library references
  - `preview_kinds`: serialized file preview kinds used by the host
- Codegen: `cargo run -p codemod_recipe_yaml --bin codemod_dsl_codegen` (or `scripts/generate-dsl-artifacts.sh`) writes `vscode_extension/schemas/generated-keyword-docs.json`, patches JSON Schema `description` fields, and refreshes TextMate keyword alternations. Run after changing `dsl_vocabulary`.
- `rust/crates/core/src/resource_path.rs`
  - shared safe resolver for file-backed resources; update this instead of adding ad hoc `join`, `canonicalize`, or traversal checks in host/engine
- `rust/crates/host/src/protocol_keys.rs`
  - host-only request/response keys such as `inlineRecipe`, `previewToken`, `snippetLines`, `ok`, and `error`

When updating Rust behavior, change these keyword owners first and then update the consuming code. Avoid adding new ad hoc string literals for existing DSL or protocol concepts.

## Top-level structure

```yaml
id: feature.area.recipe_id
name: Human-readable name
description: What this recipe does

args: []
maps: {}
steps: []
postExecution:
  - "dart format ."
```

### Top-level fields

- `id` (required in practice): unique recipe id
- Dotted ids also define nested groups in the VS Code Recipes tab. For example,
  `rust.data.add_log_line` appears under `rust` → `data` with leaf id `add_log_line`.
- `name` (optional): display name
- `description` (optional)
- `args` (optional): list of argument definitions
- `maps` (optional): recipe-local map entries
- `steps` (required): ordered list of operations
- `postExecution` (optional): list of strings run in order after a successful apply.
  Each entry is Jinja-rendered with recipe args. If the result is a path to an
  existing file, it is resolved safely recipe-local first and then under
  `.codemod/`; the script body is Jinja-rendered and executed via bash.
  Otherwise the string is run with `sh -c` (cwd = workspace).
  No builtins and no automatic per-file expansion — recipes own their commands/scripts.

## Arguments (`args`)

Each arg object supports:

- `name` (required)
- `required` (bool, default false)
- `inputKind` (optional string: `text`, `file`, `directory`, `choice`)
- `abbr`, `help`, `defaultsTo`, `options`, `allowCustomValue`, `contextKey` (deprecated alias of string `from`), `from`

`from` derives the arg from the active editor when invoking via the VS Code extension:

```yaml
args:
  - name: file
    required: true
    from: file                    # builtin: file, fileStem, fileDirname, selection, word, …
  - name: feature
    from:
      template: "{{ fileDirname | basename }}"
  - name: className
    from:
      query: |
        (class_definition name: (identifier) @name)
      capture: name
      extract: text
      scope: enclosing            # enclosing | selection | first
```

Human-oriented shortcut / slots guide: [docs/recipe-shortcuts.md](../../../../docs/recipe-shortcuts.md).

Example:

```yaml
args:
  - name: file
    required: true
    inputKind: file
  - name: outputDir
    required: true
    inputKind: directory
  - name: className
    required: true
    inputKind: choice
    options: [Widget, StatelessWidget]
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
    language: dart   # optional; see skill codemod-languages
    ops:
      - insert: { ... }
      - replace: { ... }
      - remove: { ... }
```

- `path`: file path (often templated, e.g. `{{file}}`)
- `language` (optional): tree-sitter language id (`dart`, `rust`, `java`, `kotlin`, `sqlite`, `sql`, …). When omitted, inferred from extension (`.sql` → `sqlite` unless host config overrides). Unknown extensions require an explicit `language:`; unresolved types fail with `file type not supported`.
- `ops`: list of edit operations (must not be empty)

### `when` / `whenNot` (optional guards)

Evaluated **once** on the file before any op. If guards fail, the edit is **skipped** (no error in batch preview).

- `when`: one query or list of queries (each a [`query`](#query--capture-semantics) spec). **All** must match.
- `whenNot`: forbidden patterns; edit runs only if **none** match.

Queries support the same composition as op `query` (inline, `.scm`, `libId.key`, chains).

Rust query-path detection lives in `keywords::query_conventions` (`.scm` paths and path-like strings; **not** `.yaml`/`.yml` query-library paths). Safe file lookup lives in `rust/crates/core/src/resource_path.rs`. Update those shared owners instead of duplicating query/resource path logic in engine/host code.

### `let` (optional step locals)

Bindings recomputed **before each op** on the current source. Names are available in op `text` / `query` / `capture` templates (with recipe args; locals must not collide with recipe arg names).

```yaml
let:
  - name: needsBraces
    query: |
      (constructor_body) @body
      (#match? @body "^\\s*$")
    capture: body
    extract: exists   # text | kind | exists | count
```

Optional `as:` template (Jinja) to derive a value from prior locals. Numeric transforms use filters: `{{ n | int | add(1) | string }}`.

### Sequential staging

Steps (and ops within one `edit`) run **in order** against an in-memory working tree.
Later steps see earlier mutations on the same path — disk is written only on apply.

- `create` then `edit` on the same path is supported (`ifExists: skip` for ensure-then-patch).
- Dependent `edit` then `edit` is supported (e.g. insert a class, then insert a member into it).
- Ops inside one `edit` also apply sequentially (each op reparses the current text).
- Invalid orders error (e.g. `edit` then `create` with `ifExists: fail` on an existing file).
- Preview for a multi-step same-path file is the **final** original vs modified content.
  Per-patch selection on that file is file-level (all-or-nothing); independent single-edit
  files still expose patches.

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

See skill `codemod-tree-sitter-queries` for full query language syntax (captures, predicates, operators, `.scm` files).

- `query` is a tree-sitter query and may define multiple captures (`@className`, `@body`, etc.).
- `capture` chooses which captured node span is edited.
- If the named capture is missing, unmatched, or ambiguous, execution fails.

Example insert query:

```yaml
query: |
  (class_definition
    name: (identifier) @className
    body: (class_body
      (method_signature
        (function_signature
          name: (identifier) @methodName))
      (function_body
        (block) @body))
    (#eq? @className "{{className}}")
    (#eq? @methodName "{{methodName}}"))
capture: body
anchor: end
text: "    print('codemod');\n"
```

For non-Dart files, set `language:` and use that grammar’s node names. See skill `codemod-languages`.

## Create steps (`create`)

```yaml
- create:
    path: "lib/new_file.dart"
    template: "class X {}"
    ifExists: fail
```

or:

```yaml
- create:
    path: "lib/new_file.dart"
    templateFile: templates/new_file.dart.template
    ifExists: fail
```

Rules:

- `path` required
- Exactly one of `template` or `templateFile`
- `ifExists`: `fail` (default) or `skip`
- Formatting is not a create-step flag — use top-level `postExecution` (e.g. `"dart format ."` or a script under the codemod root)

`create.templateFile` uses the shared resolver in
`rust/crates/core/src/resource_path.rs`: first relative to the referencing
recipe, then falling back to `.codemod/`.

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

Or pass call-site bindings (like function arguments). `with` keys are **child** arg
names; values are templates rendered in the **parent** context. Partial `with` is
allowed — unbound child args fall through by name from the parent (and remain on
the parent's public arg schema).

`with` only binds the **directly referenced** recipe. Deeper nesting is compositional:
each intermediate recipe owns its children’s args via its own `args` / `with`. Parents
do not bind grandchildren.

```yaml
args:
  - name: featureName
    required: true
  - name: fieldName
    required: true

steps:
  - recipe:
      id: create_repository
      with:
        className: "{{ featureName }}"
  - recipe:
      id: patch_counter
      with:
        className: "{{ featureName }}"
        # fieldName omitted → resolved from parent args
  - recipe:
      id: jinja.defaults.child
      with:
        verbose: "false"   # hardcode
```

### Step conditionals (`if` / `ifNot`)

Optional MiniJinja **expressions** on `edit`, `create`, `delete`, and `recipe` (object
form). Evaluated at runtime against recipe args (same bool coercion as templates).
Failed gates **skip** the step (or entire inlined recipe subtree) silently — same as
edit AST `when` / `whenNot`.

```yaml
args:
  - name: includeTests
    defaultsTo: "false"
  - name: file
    required: true

steps:
  - recipe:
      id: create_test_harness
      if: includeTests
  - create:
      path: "lib/extra.dart"
      template: "..."
      ifNot: file | file_exists
  - edit:
      path: "{{ file }}"
      if: migrateLegacy
      whenNot: "(...) @already"   # AST guard; both if and whenNot must allow
      ops: [...]
```

| Expression | Meaning |
|------------|---------|
| `includeTests` | Truthy bool/arg |
| `kind == "bloc"` | Comparison / `and` / `or` / `not` |
| `file \| file_exists` | Workspace-relative path exists (WorkingTree, then disk) |

`ifNot: expr` ≡ `if: not (expr)`. Both may be set (must pass `if` and fail `ifNot`).

These are **not** the same as edit `when` / `whenNot` (tree-sitter query guards on file
source).

Composition behavior:

- Referenced recipe steps are expanded in order
- Args listed in `with` are **not** unioned into the parent schema
- Unbound child args are merged by name (first definition wins)
- Create/edit/delete steps are inlined; non-empty `with` or `if`/`ifNot` wraps them in a scoped overlay
- Recipe cycles are rejected
- Unknown `with` keys (not in the child’s declared `args`) are errors (`E_RECIPE_WITH`)

## Maps

### Recipe-local maps

```yaml
maps:
  field_kind:
    tickCount: int
    label: String
```

### Workspace maps

Shared maps are YAML files under `.codemod/` classified by schema (`id` + `map:`),
not by directory. Recommended location: `.codemod/maps/*.yaml`. They merge with
recipe-local maps.

Example workspace map file:

```yaml
id: field_kind
map:
  tickCount: int
  label: String
```

### Workspace variables

Shared constants use `id` + `values:` (also schema-discovered under `.codemod/`):

```yaml
id: paths
values:
  data_root_directory: lib/data
  feature_root: lib/features
```

Use in recipe text:

```yaml
text: "  final {{ fieldName | map('field_kind') }} {{ fieldName | camel_case }} = 0;\n\n"
path: "{{ var.paths.feature_root }}/{{ feature | snake_case }}.dart"
```

## Template syntax (Jinja2)

Canonical syntax for the Rust host (MiniJinja). See [docs/recipe-templates.md](../../../docs/recipe-templates.md).

Supported in paths, queries, inline `create.template`, and `create.templateFile`:

- `{{ argName }}`
- `{{ arg | camel_case }}`, `snake_case`, `pascal_case`, `lower`, `upper`, `screaming_snake`, `kebab_case`
- `{{ pathArg | parent }}`, `basename`, `stem` (path string transforms; chainable, e.g. `{{ dir | parent | basename }}`)
- `{{ keyArg | map('mapId') }}`
- `{{ map.mapId.key }}` / `{{ var.varId.key }}`
- `{% if includeTests %}…{% endif %}` (bool args as `"true"` / `"false"`)

File-backed templates support `{% extends %}`, `{% block %}`, `{% include %}`.

Legacy helpers (`{{$camel x}}`, `{{$map 'id' key}}`) still work but emit `W_LEGACY_TEMPLATE`.

Example path: `lib/features/{{ feature | snake_case }}/{{ feature | snake_case }}_state.dart`

Example typed field insert:

```yaml
text: "  final {{ fieldName | map('field_kind') }} {{ fieldName | camel_case }};\n\n"
```

## Validation rules

Call `validate_recipes` (MCP) or host `{ "command": "validate" }` after editing YAML.
Optional scope: `{ "recipe": "scaffold_feature" }`.

Expanded-recipe checks include undeclared template variables, missing template files,
MiniJinja syntax errors, and map reference warnings. Diagnostics may include `hint`
and `relatedRecipe`.

- Recipe must have `steps`
- Arg names must be unique
- Unsupported step kinds are errors
- `edit.path` required; `edit.ops` must be non-empty
- `edit.language` when set must be a known language id
- `insert`: `query` and `capture` required
- `replace`: `query`, `capture`, `text` required
- `remove`: `query`, `capture` required
- `create.path` required; exactly one of `template` or `templateFile`
- `delete.path` required

Common errors: empty `ops`, missing `capture`, duplicate arg names.

For Rust maintainers, validation and parsing use scoped constants from `rust/crates/yaml/src/dsl/` (see `ENTRIES` in `dsl_vocabulary.rs` for hover text).

## Practical checklist

1. Start from a known-good example recipe in `.codemod/recipes/`.
2. Keep query captures explicit and stable.
3. Use `preview_recipe` before `apply_recipe`.
4. Require arguments for all dynamic values.
5. Re-run preview after apply; expect no further changes for idempotent recipes.

## Related skills

- `codemod-recipe-design-patterns` — create vs modify taxonomy
- `codemod-languages` — `language:` field, SQL dialects, grammar loading
- `codemod-tree-sitter-queries` — query language syntax, captures, predicates
- `codemod-recipe-authoring` — Dart query patterns
- `codemod-mcp-playbook` — preview/apply workflow
