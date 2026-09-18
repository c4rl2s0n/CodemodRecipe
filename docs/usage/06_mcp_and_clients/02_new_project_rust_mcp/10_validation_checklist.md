# Validation checklist

- `validate_recipes` returns `ok: true`
- `preview_recipe` returns a `previewToken`
- `apply_recipe` succeeds with that token
- Re-preview produces no files if idempotent
