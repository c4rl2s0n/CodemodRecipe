# Idempotency

Agents should always: `preview_recipe` → `apply_recipe` → re-preview.

| Kind | Expected behavior | How |
|------|-------------------|-----|
| `create_*` | Fail if exists | `ifExists = "fail"` (default) |
| `add_*` (insert) | Often **not** idempotent | Guard with `if not c:queryExists(...)` or accept preview failure on re-run |
| `replace_*` | Idempotent | `c:replace` with final desired text |
| `remove_*` | Idempotent | `c:remove` when gone → empty preview |

Plan for non-idempotent `add_*` recipes and document that in the recipe `description`.

After apply, re-run `preview_recipe` — expect `files: []` for idempotent recipes.
