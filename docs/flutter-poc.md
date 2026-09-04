# Flutter POC fixture

Read-path proof of concept for the platform layer: model root scan with a **parse tree** (property, method, state children), tree-sitter queries, cross-file verify, and `toSummary`.

## Layout

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

## Run tests

```bash
cd rust && cargo test -p codemod_recipe_host --test flutter_poc_test
cd rust && cargo test -p codemod_recipe_host --test component_incremental_test
```

## What success looks like

- **2 root instances**: `flutter.model/recipe`, `flutter.model/user`
- **Parse tree**: each model has `children` with `flutter.property`, `flutter.method`, and nested `flutter.state`
- **Summaries**: `title`, `facts`, `fields` (no `source` spans)
- **Findings**: `model.field_missing_in_store` for `subtitle` on recipe

## Incremental rescan

After the first `component_scan`, the host keeps an in-process cache (path index + fingerprints). Use `component_incremental_scan` instead of a full rescan when files change:

| Tier | Request | When |
|------|---------|------|
| 1 | `{ "dirtyPaths": ["lib/stores/recipe_store.dart"] }` | Caller knows what changed (e.g. after `apply_recipe`) |
| 2 | `{ "detectChanges": true }` | Host stats indexed paths and glob-diffs root patterns (new/deleted models) |
| 3 | `{ "force": true }` or cold cache | Full `component_scan` |

`componentInvokeOperation` uses tier 1 internally: dirty paths from the working tree trigger incremental rescan after apply.

## Operation: `addStoreField`

On `flutter.model`, `operations.addStoreField` inserts a store field for a model property that is missing from the nested `flutter.state` child:

```json
{
  "instanceId": "flutter.model/recipe",
  "operation": "addStoreField",
  "args": { "fieldName": "subtitle" }
}
```

Preview/apply via `componentInvokeOperation` (host dispatch). Signature: `apply(c, args, identity)`. After apply, incremental rescan clears `model.field_missing_in_store` when the store now tracks the field.

## Scaffold

`flutter.model` also defines top-level `scaffold.apply(c, args)` — create a new model + store from `args.name` via host `componentScaffold` (no `instanceId`).

Child nodes include `parseContext` (the context table passed to `c:component(...):parse`) for future child-level incremental reparse.

## MCP

With `codemod-mcp` configured:

- `component_scan` / `list_component_instances` — full scan (populates cache)
- `component_incremental_scan` — `{ dirtyPaths?, detectChanges?, force? }` (see tiers above)

Point at a workspace that has a stack + components (fixture: `test/fixtures/flutter_poc/workspace` with `.codemod` beside it).

## Try it (agent experiment)

1. Point `codemod_mcp` at the fixture workspace and codemod root:

   ```bash
   cargo run -q --manifest-path rust/Cargo.toml -p codemod_recipe_host --bin codemod_mcp -- \
     --workspace-root test/fixtures/flutter_poc/workspace \
     --codemod-root test/fixtures/flutter_poc/.codemod
   ```

   (Or set those paths in `.cursor/mcp.json`.)

2. Call `component_scan` (or `list_component_instances`). Expect **2** instances
   (`flutter.model/recipe`, `flutter.model/user`) and finding
   `model.field_missing_in_store` for recipe `subtitle`.

3. Edit `lib/stores/recipe_store.dart` (add `String subtitle = '';`), then either:
   - `component_incremental_scan` with `dirtyPaths: ["lib/stores/recipe_store.dart"]`, or
   - `component_incremental_scan` with `detectChanges: true`.

   The drift finding should clear without a full rescan.

4. For `addStoreField`: not an MCP tool yet. Use host `componentInvokeOperation`
   (dispatch) or run:

   ```bash
   cd rust && cargo test -p codemod_recipe_host --test component_incremental_test \
     invoke_add_store_field_clears_drift_via_incremental_rescan -- --nocapture
   ```

Read [components.md](./components.md) and the MCP playbook skill
(`codemod-mcp-playbook`) for the agent workflow.

## vs reference_stack

| | `reference_stack` | `flutter_poc` |
|--|-------------------|---------------|
| Purpose | Runtime smoke test | Contract reference (tree + context) |
| Discover | Component `discover` | `stack.globs` + host file loop |
| Parse | Regex | Tree-sitter (`flutter.*` query lib) |
| Structure | Flat `fields` | `children[]` with `source` spans |
| Verify | Single warning | Cross-file model ↔ store via nested state child |

See [component-contract.md](./component-contract.md).
