# Hooks

| Hook | When | Returns |
|------|------|---------|
| `identify(c)` | Root scan (required for stack roots) | Freeform list of locators |
| `parse(c, locator)` | Per locator (or child context) | Node table; `nil` to skip; or `{ instances = { ... } }` |
| `verify(c, instance)` | After parse | Array of findings `{ severity, code, message, instanceId?, locator? }` |
| `scaffold.apply(c, args)` | `componentScaffold` | Side effects via `c:insert` / `c:create` / … |
| `operations.*.apply(c, args, target)` | `componentInvokeOperation` | Side effects via `c:insert` / `c:replace` / … |

**`c`** is the host sandbox only (same as recipes) — **not** argument storage. **`args`** is always an explicit table. **`target`** is only on mutate ops (scanned instance: `id`, `componentId`, `locator`, `node`).

**Child components** are invoked with `c:component("child.id"):parse(context)` inside parent `parse`. They need not be listed in `stack.components` and typically omit `identify`.
