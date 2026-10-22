# Troubleshooting

| Symptom | Likely cause | Fix |
|---------|--------------|-----|
| `language not supported: foo` | Invalid `language` id | Pick id from pack catalog or native list |
| `file type not supported: path` | No `language` and extension unknown | Set `language` or use a known extension |
| `Invalid node type "class_declaration"` | Old Dart query on pack grammar | Use `class_definition` (see above) |
| `failed to load language` / download error | Network blocked on first use | Pre-download/cache grammars in CI |
| `query matched no nodes` | Wrong language id or query for grammar | Set `language` explicitly; fix node names |
| Generic SQL on SQLite file | Used `sql` instead of `sqlite` | `language = "sqlite"` or fix `--sql-default` |

---
