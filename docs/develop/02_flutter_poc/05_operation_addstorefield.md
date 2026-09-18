# Operation: `addStoreField`

On `flutter.model`, `operations.addStoreField` inserts a store field for a model property that is missing from the nested `flutter.state` child:

```json
{
  "instanceId": "flutter.model/recipe",
  "operation": "addStoreField",
  "args": { "fieldName": "subtitle" }
}
```

Preview/apply via `componentInvokeOperation` (host dispatch). Signature: `apply(c, args, identity)`. After apply, incremental rescan clears `model.field_missing_in_store` when the store now tracks the field.
