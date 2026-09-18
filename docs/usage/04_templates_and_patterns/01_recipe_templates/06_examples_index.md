# Examples index

Runnable recipes demonstrating each feature:

| Recipe | Location | Demonstrates |
|--------|----------|--------------|
| `add_counter_field` | [`.codemod/recipes/add_counter_field.luau`](../../../../.codemod/recipes/add_counter_field.luau) | `c:template` + `{{ field \| camel_case }}` in insert text |
| `conditional_create` | [`.codemod/recipes/conditional_create.luau`](../../../../.codemod/recipes/conditional_create.luau) | `c:template`, `{% if %}`, `defaultsTo` |
| `patch_counter` | [`test/fixtures/scaffold_project/.../patch_counter.luau`](../../../../test/fixtures/scaffold_project/.codemod/recipes/patch_counter.luau) | `\| map('field_kind')` + `\| camel_case` |
| `create_repository` | [`test/fixtures/scaffold_project/.../create_repository.luau`](../../../../test/fixtures/scaffold_project/.codemod/recipes/create_repository.luau) | File template with casing filters |
| `jinja.casing.showcase` | [`test/fixtures/jinja_examples/.../showcase_casing.luau`](../../../../test/fixtures/jinja_examples/.codemod/recipes/showcase_casing.luau) | All casing filters + both map syntaxes |
| `jinja.path.showcase` | [`test/fixtures/jinja_examples/.../showcase_path.luau`](../../../../test/fixtures/jinja_examples/.codemod/recipes/showcase_path.luau) | `parent`, `basename`, `stem` path filters |
| `jinja.create.conditional` | [`test/fixtures/jinja_examples/.../conditional_create.luau`](../../../../test/fixtures/jinja_examples/.codemod/recipes/conditional_create.luau) | Bool conditional in template file |
| `jinja.create.layout` | [`test/fixtures/jinja_examples/.../create_with_layout.luau`](../../../../test/fixtures/jinja_examples/.codemod/recipes/create_with_layout.luau) | `extends` + `include` |
| `jinja.defaults.orchestrator` | [`test/fixtures/jinja_examples/.../defaults_orchestrator.luau`](../../../../test/fixtures/jinja_examples/.codemod/recipes/defaults_orchestrator.luau) | `defaultsTo` via `c:run` |
| `jinja.bind.orchestrator` | [`test/fixtures/jinja_examples/.../with_bind_orchestrator.luau`](../../../../test/fixtures/jinja_examples/.codemod/recipes/with_bind_orchestrator.luau) | `c:run` remap + hardcode |
| `jinja.bind.orchestrator_partial` | [`test/fixtures/jinja_examples/.../with_partial_orchestrator.luau`](../../../../test/fixtures/jinja_examples/.codemod/recipes/with_partial_orchestrator.luau) | Partial args + parent fallthrough |
| `jinja.step_if.orchestrator` | [`test/fixtures/jinja_examples/.../step_if_orchestrator.luau`](../../../../test/fixtures/jinja_examples/.codemod/recipes/step_if_orchestrator.luau) | Luau `if` + file-exists gate on `c:run` / `c:create` |

Integration tests: [`rust/crates/host/tests/jinja_examples_test.rs`](../../../../rust/crates/host/tests/jinja_examples_test.rs).

## Quick snippets

**Conditional create** (`conditional_create.luau`):

```lua
args = {
  { name = "includeTests", defaultsTo = "false" },
}
apply = function(c, args)
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
apply = function(c, args)
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
