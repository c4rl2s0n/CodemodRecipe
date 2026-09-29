# Operation: `addStoreField`

On `flutter.model`, `operations.addStoreField` inserts a store field for a model property that is missing from the related `flutter.state` instance (resolved via `graph:incoming(target.id, "stores")`):

```json
{
  "instanceId": "flutter.model/recipe",
  "operation": "addStoreField",
  "args": { "fieldName": "subtitle" }
}
```

Preview/apply via `componentInvokeOperation` (host dispatch). Signature: `apply(c, args, target, graph)`. After apply, incremental rescan clears `model.field_missing_in_store` when the store now tracks the field.
