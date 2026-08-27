# Recipe templates (Jinja2 / MiniJinja)

The Rust host renders recipe strings and file-backed templates with
[MiniJinja](https://docs.rs/minijinja/) (Jinja2-compatible syntax).

## Where templates apply

Templates are rendered via **`c:template`** inside Luau recipes (MiniJinja). Typical uses:

| Call | Role |
|------|------|
| `c:template({ inline = "..." })` | Inline Jinja over recipe args + maps/vars |
| `c:template({ path = "templates/x.template" })` | File-backed template (`extends` / `include`) |
| `c:create({ content = c:template({ path = "..." }) })` | New file body from template |
| `postExecution` / `c:postExec` | Jinja-rendered shell commands |

Maps and variables registered under `.codemod/maps/` and `.codemod/variables/` (Luau modules)
merge into the MiniJinja context automatically.

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

### Numeric helpers

Use on recipe args inside templates:

```jinja
{{ count | int | add(1) | string }}
```

### Maps

Workspace maps (Luau modules under `.codemod/maps/`) and recipe-local `maps` on the
return table merge into the render context under **`map`**.

```jinja
{{ fieldName | map('field_kind') }}
```

Or nested context:

```jinja
{{ map.field_kind[fieldName] }}
```

### Variables

Workspace variables (Luau modules under `.codemod/variables/`) are exposed under **`var`**:

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

### File-backed templates (`c:template`)

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
| `E_MISSING_TEMPLATE` | error | Template path not found (`c:template`) |
| `E_TEMPLATE_SYNTAX` | error | MiniJinja parse error |
| `E_COMPOSE_CYCLE` | error | Recipe reference cycle |
| `W_MAP_ID_NOT_FOUND` | warning | Unknown map id in template |
| `W_LEGACY_TEMPLATE` | warning | Legacy `{{$…}}` helper detected |

Diagnostics include optional `hint` and `relatedRecipe` fields.

## Examples index

Runnable recipes demonstrating each feature:

| Recipe | Location | Demonstrates |
|--------|----------|--------------|
| `add_counter_field` | [`.codemod/recipes/add_counter_field.luau`](../.codemod/recipes/add_counter_field.luau) | `c:template` + `{{ field \| camel_case }}` in insert text |
| `conditional_create` | [`.codemod/recipes/conditional_create.luau`](../.codemod/recipes/conditional_create.luau) | `c:template`, `{% if %}`, `defaultsTo` |
| `patch_counter` | [`test/fixtures/scaffold_project/.../patch_counter.luau`](../test/fixtures/scaffold_project/.codemod/recipes/patch_counter.luau) | `\| map('field_kind')` + `\| camel_case` |
| `create_repository` | [`test/fixtures/scaffold_project/.../create_repository.luau`](../test/fixtures/scaffold_project/.codemod/recipes/create_repository.luau) | File template with casing filters |
| `jinja.casing.showcase` | [`test/fixtures/jinja_examples/.../showcase_casing.luau`](../test/fixtures/jinja_examples/.codemod/recipes/showcase_casing.luau) | All casing filters + both map syntaxes |
| `jinja.path.showcase` | [`test/fixtures/jinja_examples/.../showcase_path.luau`](../test/fixtures/jinja_examples/.codemod/recipes/showcase_path.luau) | `parent`, `basename`, `stem` path filters |
| `jinja.create.conditional` | [`test/fixtures/jinja_examples/.../conditional_create.luau`](../test/fixtures/jinja_examples/.codemod/recipes/conditional_create.luau) | Bool conditional in template file |
| `jinja.create.layout` | [`test/fixtures/jinja_examples/.../create_with_layout.luau`](../test/fixtures/jinja_examples/.codemod/recipes/create_with_layout.luau) | `extends` + `include` |
| `jinja.defaults.orchestrator` | [`test/fixtures/jinja_examples/.../defaults_orchestrator.luau`](../test/fixtures/jinja_examples/.codemod/recipes/defaults_orchestrator.luau) | `defaultsTo` via `c:run` |
| `jinja.bind.orchestrator` | [`test/fixtures/jinja_examples/.../with_bind_orchestrator.luau`](../test/fixtures/jinja_examples/.codemod/recipes/with_bind_orchestrator.luau) | `c:run` remap + hardcode |
| `jinja.bind.orchestrator_partial` | [`test/fixtures/jinja_examples/.../with_partial_orchestrator.luau`](../test/fixtures/jinja_examples/.codemod/recipes/with_partial_orchestrator.luau) | Partial args + parent fallthrough |
| `jinja.step_if.orchestrator` | [`test/fixtures/jinja_examples/.../step_if_orchestrator.luau`](../test/fixtures/jinja_examples/.codemod/recipes/step_if_orchestrator.luau) | Luau `if` + file-exists gate on `c:run` / `c:create` |

Integration tests: [`rust/crates/host/tests/jinja_examples_test.rs`](../rust/crates/host/tests/jinja_examples_test.rs).

### Quick snippets

**Conditional create** (`conditional_create.luau`):

```lua
args = {
  { name = "includeTests", defaultsTo = "false" },
}
apply = function(c)
  c:create({
    path = c:template({ inline = "lib/{{ className | snake_case }}/{{ className | snake_case }}_widget.dart" }),
    content = c:template({ path = "templates/conditional_widget.dart.template" }),
    ifExists = "fail",
  })
end
```

Template (`conditional_widget.dart.template`):

```jinja
class {{ className | pascal_case }}Widget {
{% if includeTests %}
  void testHook() {}
{% endif %}
}
```

**defaultsTo in orchestrator** (`defaults_orchestrator.luau`):

```lua
args = {
  { name = "label", required = true },
}
apply = function(c)
  c:run("jinja.defaults.child")  -- child declares verbose with defaultsTo = "false"
end
```

**Template inheritance** (`create_with_layout.luau`):

```lua
c:create({
  path = c:template({ inline = "lib/generated/{{ className | snake_case }}_layout.dart" }),
  content = c:template({ path = "templates/layout_widget.dart.template" }),
  ifExists = "fail",
})
```

Child template uses `{% extends "templates/layouts/base.template" %}` and
`{% include "templates/partials/header.template" %}`. These paths resolve
recipe-local first, then fall back to `.codemod/`.

## Related docs

- [codemod-mcp.md](codemod-mcp.md) — MCP tools and host protocol
- [recipe-design-patterns.md](recipe-design-patterns.md) — scaffold orchestrators
- Skill `codemod-luau-recipes` — host API and template calls
