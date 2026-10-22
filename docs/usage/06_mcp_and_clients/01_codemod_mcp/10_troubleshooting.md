# Troubleshooting

| Symptom | Action |
|---------|--------|
| `Unknown recipe` | Call `list_recipes`, verify id and workspace root |
| `Missing previewToken` | Call `preview_recipe` first |
| `Stale previewToken` | Re-run preview after file changes |
| `Invalid node type "class_declaration"` | Use `class_definition` in Dart queries |
| `language not supported` | Set valid `language` on the op |
| Empty `files` in preview | Query matched nothing, or recipe already applied |
| Apply differs from preview | Expected when postExec ran on apply only — check transcript |
