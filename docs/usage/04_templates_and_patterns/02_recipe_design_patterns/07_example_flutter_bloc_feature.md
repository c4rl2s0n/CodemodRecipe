# Example: Flutter Bloc feature

Illustrative sketches — not shipped recipes.

| User intent | Recipe kind | What it does |
|-------------|-------------|--------------|
| New Bloc feature | `scaffold_bloc_feature` | Compose `create_bloc`, `create_bloc_state`, `create_bloc_event`, `patch_app_routes` |
| Add state field | `add_bloc_state_field` | Insert field in state class; update `copyWith` / `props` |
| Add event handler | `add_bloc_event_handler` | Insert `on<Event>` handler in bloc class |
| Add event type | `add_bloc_event` | Insert sealed class variant in event file |

Orchestrator sketch:

```lua
return {
  id = "scaffold_bloc_feature",
  description = "Scaffold a new Bloc feature (state, event, bloc files + app wiring)",
  args = {
    { name = "feature", required = true, inputKind = "text" },
  },
  apply = function(c, args)
    c:run("create_bloc_state")
    c:run("create_bloc_event")
    c:run("create_bloc")
    c:run("patch_app_routes")
  end,
}
```

Atomic modify sketch (state field):

```lua
return {
  id = "add_bloc_state_field",
  description = "Add a typed field to an existing Bloc state class",
  args = {
    { name = "feature", required = true, inputKind = "text" },
    { name = "fieldName", required = true, inputKind = "text" },
  },
  maps = {
    dart_types = { email = "String", count = "int" },
  },
  apply = function(c, args)
    local path = c:template({
      inline = "lib/features/{{ feature | snake_case }}/{{ feature | snake_case }}_state.dart",
    })
    local query = c:template({
      inline = [[
(class_definition
  name: (identifier) @className
  body: (class_body) @body
  (#eq? @className "{{ feature | pascal_case }}State"))
]],
    })
    c:insert({
      path = path,
      query = query,
      capture = "body",
      anchor = "start",
      text = c:template({
        inline = "  final {{ fieldName | map('dart_types') }} {{ fieldName | camel_case }};\n\n",
      }),
    })
  end,
}
```

**Agent workflow:**

1. `describe_recipe` on `scaffold_bloc_feature` for new features.
2. After scaffolding, use atomic `add_*` recipes for incremental changes.
3. Never guess AST paths — use tree-sitter queries with explicit captures.
