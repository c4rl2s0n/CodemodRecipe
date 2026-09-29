# Try it (agent experiment)

1. Point `codemod_mcp` at the fixture workspace and codemod root:

   ```bash
   cargo run -q --manifest-path rust/Cargo.toml -p codemod_recipe_host --bin codemod_mcp -- \
     --workspace-root test/fixtures/flutter_poc/workspace \
     --codemod-root test/fixtures/flutter_poc/.codemod
   ```

   (Or set those paths in `.cursor/mcp.json`.)

2. Call `component_scan` (or `list_component_instances`). Expect **4** instances
   (`flutter.model/recipe`, `flutter.model/user`, `flutter.state/recipe`,
   `flutter.state/user`), two `stores` edges in `edges` (plus `kindGraph`), and finding
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

Read [components.md](../../usage/05_components_and_stacks/01_components/00_index.md) and [codemod-mcp.md](../../usage/06_mcp_and_clients/01_codemod_mcp/00_index.md) for the agent workflow.
