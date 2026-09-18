# What success looks like

- **2 root instances**: `flutter.model/recipe`, `flutter.model/user`
- **Parse tree**: each model has `children` with `flutter.property`, `flutter.method`, and nested `flutter.state`
- **Summaries**: `title`, `facts`, `fields` (no `source` spans)
- **Findings**: `model.field_missing_in_store` for `subtitle` on recipe
