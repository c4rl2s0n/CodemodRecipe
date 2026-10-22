# What success looks like

- **4 root instances**: `flutter.model/recipe`, `flutter.model/user`, `flutter.state/recipe`, `flutter.state/user`
- **Parse tree**: each model has `children` with `flutter.property` and `flutter.method`
- **Relations**: each store root emits a `stores` edge to its model (`node.relations.stores` on the store; `edges` + `kindGraph` on the scan response)
- **Summaries**: `title`, `facts`, `fields` (no `source` spans)
- **Findings**: `model.field_missing_in_store` for `subtitle` on recipe (graph-aware verify walks `incoming` `stores` edges)
