# Registered recipe examples

`preview_recipe`:

```json
{
  "recipe": "insert_log_line",
  "args": {
    "file": "lib/settings.dart"
  },
  "snippetLines": 5
}
```

`apply_recipe`:

```json
{
  "recipe": "insert_log_line",
  "args": {
    "file": "lib/settings.dart"
  },
  "previewToken": "<from preview_recipe>",
  "selection": {
    "files": {
      "lib/settings.dart": { "include": true }
    }
  }
}
```
