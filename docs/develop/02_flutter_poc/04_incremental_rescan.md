# Incremental rescan

After the first `component_scan`, the host keeps an in-process cache (path index + fingerprints). Use `component_incremental_scan` instead of a full rescan when files change:

| Tier | Request | When |
|------|---------|------|
| 1 | `{ "dirtyPaths": ["lib/stores/recipe_store.dart"] }` | Caller knows what changed (e.g. after `apply_recipe`) |
| 2 | `{ "detectChanges": true }` | Host stats indexed paths and glob-diffs root patterns (new/deleted models) |
| 3 | `{ "force": true }` or cold cache | Full `component_scan` |

`componentInvokeOperation` uses tier 1 internally: dirty paths from the working tree trigger incremental rescan after apply.
