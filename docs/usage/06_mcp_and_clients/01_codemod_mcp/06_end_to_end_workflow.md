# End-to-end workflow

0. `bootstrap_project` (once per workspace)
1. `validate_recipes` after recipe edits
2. `list_recipes` / `describe_recipe`
3. `preview_recipe` (save `previewToken`; inspect `files` + `transcript`)
4. `apply_recipe` with the same recipe/args and that token
5. Optional: preview again to verify idempotency (`files: []`)
