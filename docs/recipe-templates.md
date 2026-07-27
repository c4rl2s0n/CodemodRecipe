# Recipe templates (Jinja2 / MiniJinja)

The Rust host renders recipe strings and file-backed templates with
[MiniJinja](https://docs.rs/minijinja/) (Jinja2-compatible syntax).

## Where templates apply

| Location | Render path |
|----------|-------------|
| `edit.path`, `edit.ops[].query/capture/text` | Inline `render_str`; each `query` step (string or list item) after file load |
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

Paths are relative to `.codemod/` (codemod root). Supports inheritance:

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
| `showcase_casing` | [`test/fixtures/jinja_examples/.../showcase_casing.yaml`](../test/fixtures/jinja_examples/.codemod/recipes/showcase_casing.yaml) | All casing filters + both map syntaxes |
| `conditional_create` (fixture) | [`test/fixtures/jinja_examples/.../conditional_create.yaml`](../test/fixtures/jinja_examples/.codemod/recipes/conditional_create.yaml) | Bool conditional in template file |
| `create_with_layout` | [`test/fixtures/jinja_examples/.../create_with_layout.yaml`](../test/fixtures/jinja_examples/.codemod/recipes/create_with_layout.yaml) | `extends` + `include` |
| `defaults_orchestrator` | [`test/fixtures/jinja_examples/.../defaults_orchestrator.yaml`](../test/fixtures/jinja_examples/.codemod/recipes/defaults_orchestrator.yaml) | `defaultsTo` via recipe composition |
| `with_bind_orchestrator` | [`test/fixtures/jinja_examples/.../with_bind_orchestrator.yaml`](../test/fixtures/jinja_examples/.codemod/recipes/with_bind_orchestrator.yaml) | `recipe.with` forward + hardcode |
| `with_partial_orchestrator` | [`test/fixtures/jinja_examples/.../with_partial_orchestrator.yaml`](../test/fixtures/jinja_examples/.codemod/recipes/with_partial_orchestrator.yaml) | Partial `with` + parent fallthrough |

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
  - recipe: defaults_child   # child declares verbose with defaultsTo: "false"
```

**Template inheritance** (`create_with_layout.yaml`):

```yaml
steps:
  - create:
      templateFile: templates/layout_widget.dart.template
```

Child template uses `{% extends "templates/layouts/base.template" %}` and
`{% include "templates/partials/header.template" %}` (paths relative to `.codemod/`).

## Related docs

- [codemod-mcp.md](codemod-mcp.md) — MCP tools and host protocol
- [recipe-design-patterns.md](recipe-design-patterns.md) — scaffold orchestrators
- Skill `codemod-yaml-dsl-v2` — full YAML DSL reference
