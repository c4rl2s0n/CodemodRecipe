# Layout

```
test/fixtures/flutter_poc/
  workspace/
    lib/models/recipe.dart      # 2 fields + copy() method
    lib/models/user.dart
    lib/stores/recipe_store.dart  # missing subtitle (intentional drift)
    lib/stores/user_store.dart
  .codemod/
    stack.luau                  # roots: flutter.model + flutter.state
    lib/platform.luau           # reference helpers (authoring)
    queries/flutter.luau
    components/
      flutter.model.luau      # root scan; graph-aware verify
      flutter.state.luau      # root scan; relate emits `stores` edge → flutter.model
      flutter.property.luau   # child (nested under model)
      flutter.method.luau     # child (nested under model)
```
