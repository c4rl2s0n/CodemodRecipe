# Layout

```
test/fixtures/flutter_poc/
  workspace/
    lib/models/recipe.dart      # 2 fields + copy() method
    lib/models/user.dart
    lib/stores/recipe_store.dart  # missing subtitle (intentional drift)
    lib/stores/user_store.dart
  .codemod/
    stack.luau                  # globs for flutter.model
    lib/platform.luau           # reference helpers (authoring)
    queries/flutter.luau
    components/
      flutter.model.luau      # root scan
      flutter.state.luau      # child (nested under model)
      flutter.property.luau   # child
      flutter.method.luau     # child
```
