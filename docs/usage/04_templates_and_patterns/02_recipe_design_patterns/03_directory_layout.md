# Directory layout

Group recipes by **concept** (bloc, repository, routing), not by file type. Adopt
incrementally — flat `recipes/` is fine for small projects.

Dirs under `.codemod/` are **convention only**; the host discovers Luau modules
anywhere under `.codemod/`: recipes (`return { id, apply, ... }`), maps
(`{ id, map = { ... } }`), variables (`{ id, values = { ... } }`).

```text
.codemod/
  recipes/
    bloc/
      scaffold_bloc_feature.luau    # orchestrator
      create_bloc.luau
      create_bloc_state.luau
      add_bloc_state_field.luau
      add_bloc_event_handler.luau
    shared/
      add_class_field.luau          # cross-cutting atomic edits
  maps/
    dart_types.luau                 # fieldName -> Dart type
    bloc_event_kinds.luau
  variables/
    paths.luau                      # shared constants
  templates/
    bloc/
      bloc.dart.template
      bloc_state.dart.template
      bloc_event.dart.template
```

## Naming rules

- Recipe `id` is the stable API (`list_recipes` / `describe_recipe`). Use `snake_case`.
- Prefix encodes kind: `create_`, `add_`, `patch_`, `remove_`, `scaffold_`.
- Maps are keyed by **allowed symbolic names** (not free-form strings).

Example workspace map (`.codemod/maps/field_kind.luau`):

```lua
return {
  id = "field_kind",
  map = {
    tickCount = "int",
    label = "String",
  },
}
```
