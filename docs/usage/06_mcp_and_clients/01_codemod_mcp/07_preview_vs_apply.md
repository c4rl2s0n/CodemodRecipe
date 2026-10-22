# Preview vs apply

## Recipes

- Both run `apply(c, args)`. Preview uses a WorkingTree; **postExec is skipped** (listed in transcript).
- Apply **re-runs** Luau and flushes before postExec / `postExecution`.
- `previewToken` = hash(recipe or `inlineRecipe` + args + starting file contents of planned paths).
- `apply_recipe` **requires** `previewToken` from a matching `preview_recipe` (same recipe/args). Stale if files changed.

`inlineRecipe` is a Luau **source string** (not a JSON object).

## Components and stacks

Same gate for mutating tools:

| Dry-run | Apply | Token |
|---------|-------|-------|
| `component_dry_run_hook` | `component_apply_hook` | Required on apply; hash of hook identity + args + planned paths |
| `stack_dry_run_operation` | `stack_apply_operation` | Required on apply; hash of operation + args + planned paths |

- Dry-run returns `previewToken` on success (also `files`, `transcript`, `logs`).
- Apply validates the token with a fresh dry-run of the same args, then runs apply. Missing/stale token → error (`Missing previewToken` / `Stale previewToken`).
- Read-only hooks (`identify` / `parse` / …) still participate when called via apply tools; prefer dry-run for authoring.

## Selection

Optional `selection` on `apply_recipe` limits which previewed paths flush (recipe only).
