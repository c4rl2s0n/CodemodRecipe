# Recipe templates (Jinja2 / MiniJinja)

The Rust host renders recipe strings and file-backed templates with
[MiniJinja](https://docs.rs/minijinja/) (Jinja2-compatible syntax).

## Where templates apply

| Location | Render path |
|----------|-------------|
| `edit.path`, `edit.when` / `edit.whenNot`, `edit.let[].query` / `capture` / `as`, `edit.ops[].query/capture/text` | Inline `render_str`; guard queries and `let` bindings use the same query resolution as ops. Op `text`/`query`/`capture` re-render **per op** when `let` or guards are present (locals merged with recipe args). Each `query` step (string or list item) after file load |
| `edit.if` / `edit.ifNot`, `create.if` / `create.ifNot`, `delete.if` / `delete.ifNot`, `recipe.if` / `recipe.ifNot`, `if.if` / `if.ifNot` (group step) | MiniJinja **expressions** (not `{{ … }}` templates) over recipe args; optional `file_exists` filter. Skip step or entire `if` group when the gate fails |
| `create.path`, `create.template` | Inline `render_str` |
| `create.templateFile` | File loader (`extends` / `include` supported) |
| `delete.path` | Inline `render_str` |

All templated fields share the same argument namespace (`args` + merged `maps`).

## Canonical syntax

### Variables

```jinja
{{ file }}
{{ className }}
```

### Casing filters

```jinja
{{ feature | snake_case }}
{{ feature | camel_case }}
{{ feature | pascal_case }}
{{ feature | lower }}
{{ feature | upper }}
{{ feature | screaming_snake }}
{{ feature | kebab_case }}
```

### Path filters

Pure string transforms on file/directory path args (no workspace I/O):

```jinja
{{ featureDir | parent }}
{{ featureDir | basename }}
{{ featureDir | parent | basename }}
{{ file | stem }}
```

| Filter | Result for `lib/features/feed/widgets` / `lib/foo.dart` |
|--------|--------------------------------------------------------|
| `parent` | `lib/features/feed` / `lib` |
| `basename` | `widgets` / `foo.dart` |
| `stem` | `widgets` / `foo` |

Trailing slashes are stripped; `\` separators are normalized to `/`.

### Numeric helpers (edit `let` / `as` templates)

Use on string locals from `extract: text` or recipe args:

```jinja
{{ count | int | add(1) | string }}
{{ offset | int | sub(2) | string }}
```

Also available: `trim`, `string` / `str` (alias).

### Maps

Workspace maps (schema: `id` + `map:`, anywhere under `.codemod/`) and recipe-local
`maps:` merge into the render context under **`map`**. Lookup via filter:

```jinja
{{ fieldName | map('field_kind') }}
```

Or nested context:

```jinja
{{ map.field_kind[fieldName] }}
```

### Variables

Workspace variables (schema: `id` + `values:`) are exposed under **`var`**:

```jinja
{{ var.paths.feature_root }}
```

### Conditionals

Boolean args: pass `"true"` / `"false"` from MCP or VS Code — the host coerces
them for `{% if %}`:

```jinja
{% if include_repository %}
import '{{ feature | snake_case }}_repository.dart';
{% endif %}
```

### File-backed templates (`create.templateFile`)

File-backed templates use the shared resolver in
`rust/crates/core/src/resource_path.rs`.

- First try the path relative to the referencing recipe file.
- If not found, fall back to `.codemod/`.
- `extends` / `include` use the same policy.

This keeps template lookup consistent with query-file and `postExecution`
resource resolution while preserving strict traversal checks.

Supports inheritance:

```jinja
{% extends "layouts/base.template" %}
{% block body %}
class {{ className | pascal_case }} {}
{% endblock %}
```

```jinja
{% include "partials/header.template" %}
```

## Legacy helpers (deprecated)

These still work via a pre-pass shim but emit `W_LEGACY_TEMPLATE` during
validation:

| Legacy | Canonical |
|--------|-----------|
| `{{$snake x}}` | `{{ x \| snake_case }}` |
| `{{$camel x}}` | `{{ x \| camel_case }}` |
| `{{$pascal x}}` | `{{ x \| pascal_case }}` |
| `{{$map 'id' key}}` | `{{ key \| map('id') }}` |

## `defaultsTo`

Recipe args may declare `defaultsTo`. The host applies defaults before required-arg
checks and before rendering. Child recipe defaults are visible after orchestrator
expansion (parent arg definitions win on name collision).

## Validation

Call `validate_recipes` (MCP), `{ "command": "validate" }` (stdio host), or
**Codemod Recipe: Validate Recipes** in VS Code.

| Code | Severity | Meaning |
|------|----------|---------|
| `E_UNDECLARED_ARG` | error | Template references unknown variable |
| `E_MISSING_TEMPLATE` | error | `templateFile` path not found |
| `E_TEMPLATE_SYNTAX` | error | MiniJinja parse error |
| `E_COMPOSE_CYCLE` | error | Recipe reference cycle |
| `W_MAP_ID_NOT_FOUND` | warning | Unknown map id in template |
| `W_LEGACY_TEMPLATE` | warning | Legacy `{{$…}}` helper detected |

Diagnostics include optional `hint` and `relatedRecipe` fields.

## Examples index

Runnable recipes demonstrating each feature:

| Recipe | Location | Demonstrates |
|--------|----------|--------------|
| `add_counter_field` | [`.codemod/recipes/add_counter_field.yaml`](../.codemod/recipes/add_counter_field.yaml) | `{{ field \| camel_case }}` in edit text |
| `conditional_create` | [`.codemod/recipes/conditional_create.yaml`](../.codemod/recipes/conditional_create.yaml) | `templateFile`, `{% if %}`, `defaultsTo` |
| `patch_counter` | [`test/fixtures/scaffold_project/.../patch_counter.yaml`](../test/fixtures/scaffold_project/.codemod/recipes/patch_counter.yaml) | `\| map('field_kind')` + `\| camel_case` |
| `create_repository` | [`test/fixtures/scaffold_project/.../create_repository.yaml`](../test/fixtures/scaffold_project/.codemod/recipes/create_repository.yaml) | `templateFile` with casing filters |
| `jinja.casing.showcase` | [`test/fixtures/jinja_examples/.../showcase_casing.yaml`](../test/fixtures/jinja_examples/.codemod/recipes/showcase_casing.yaml) | All casing filters + both map syntaxes |
| `jinja.path.showcase` | [`test/fixtures/jinja_examples/.../showcase_path.yaml`](../test/fixtures/jinja_examples/.codemod/recipes/showcase_path.yaml) | `parent`, `basename`, `stem` path filters |
| `jinja.create.conditional` | [`test/fixtures/jinja_examples/.../conditional_create.yaml`](../test/fixtures/jinja_examples/.codemod/recipes/conditional_create.yaml) | Bool conditional in template file |
| `jinja.create.layout` | [`test/fixtures/jinja_examples/.../create_with_layout.yaml`](../test/fixtures/jinja_examples/.codemod/recipes/create_with_layout.yaml) | `extends` + `include` |
| `jinja.defaults.orchestrator` | [`test/fixtures/jinja_examples/.../defaults_orchestrator.yaml`](../test/fixtures/jinja_examples/.codemod/recipes/defaults_orchestrator.yaml) | `defaultsTo` via recipe composition |
| `jinja.bind.orchestrator` | [`test/fixtures/jinja_examples/.../with_bind_orchestrator.yaml`](../test/fixtures/jinja_examples/.codemod/recipes/with_bind_orchestrator.yaml) | `recipe.with` forward + hardcode |
| `jinja.bind.orchestrator_partial` | [`test/fixtures/jinja_examples/.../with_partial_orchestrator.yaml`](../test/fixtures/jinja_examples/.codemod/recipes/with_partial_orchestrator.yaml) | Partial `with` + parent fallthrough |
| `jinja.step_if.orchestrator` | [`test/fixtures/jinja_examples/.../step_if_orchestrator.yaml`](../test/fixtures/jinja_examples/.codemod/recipes/step_if_orchestrator.yaml) | Step `if` / `ifNot` + `file_exists` on recipe/create (also see group `if:` step kind in yaml-dsl skill) |

Integration tests: [`rust/crates/host/tests/jinja_examples_test.rs`](../rust/crates/host/tests/jinja_examples_test.rs).

### Quick snippets

**Conditional create** (`conditional_create.yaml`):

```yaml
args:
  - name: includeTests
    defaultsTo: "false"
steps:
  - create:
      templateFile: templates/conditional_widget.dart.template
```

Template (`conditional_widget.dart.template`):

```jinja
class {{ className | pascal_case }}Widget {
{% if includeTests %}
  void testHook() {}
{% endif %}
}
```

**defaultsTo in orchestrator** (`defaults_orchestrator.yaml`):

```yaml
args:
  - name: label
    required: true
steps:
  - recipe: jinja.defaults.child   # child declares verbose with defaultsTo: "false"
```

**Template inheritance** (`create_with_layout.yaml`):

```yaml
steps:
  - create:
      templateFile: templates/layout_widget.dart.template
```

Child template uses `{% extends "templates/layouts/base.template" %}` and
`{% include "templates/partials/header.template" %}`. These paths resolve
recipe-local first, then fall back to `.codemod/`.

## Related docs

- [codemod-mcp.md](codemod-mcp.md) — MCP tools and host protocol
- [recipe-design-patterns.md](recipe-design-patterns.md) — scaffold orchestrators
- Skill `codemod-yaml-dsl` — full YAML DSL reference
