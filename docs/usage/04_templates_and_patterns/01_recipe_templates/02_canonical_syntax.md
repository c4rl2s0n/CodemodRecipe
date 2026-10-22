# Canonical syntax

## Variables

```jinja
{{ file }}
{{ className }}
```

## Casing filters

```jinja
{{ feature | snake_case }}
{{ feature | camel_case }}
{{ feature | pascal_case }}
{{ feature | lower }}
{{ feature | upper }}
{{ feature | screaming_snake }}
{{ feature | kebab_case }}
```

## Path filters

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

## Numeric helpers

Use on recipe args inside templates:

```jinja
{{ count | int | add(1) | string }}
```

## Maps

Workspace maps (Luau modules under `.codemod/maps/`) and recipe-local `maps` on the
return table merge into the render context under **`map`**.

```jinja
{{ fieldName | map('field_kind') }}
```

Or nested context:

```jinja
{{ map.field_kind[fieldName] }}
```

## Variables

Workspace variables (Luau modules under `.codemod/variables/`) are exposed under **`var`**:

```jinja
{{ var.paths.feature_root }}
```

## Conditionals

Boolean args: pass `"true"` / `"false"` from MCP or VS Code — the host coerces
them for `{% if %}`:

```jinja
{% if include_repository %}
import '{{ feature | snake_case }}_repository.dart';
{% endif %}
```

## File-backed templates (`c:template`)

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
