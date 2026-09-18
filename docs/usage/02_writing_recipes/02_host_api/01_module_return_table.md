# Module return table

| Field | Required | Role |
|-------|----------|------|
| `id` | yes | Unique recipe id (dotted ids nest in VS Code Recipes tab) |
| `apply` | yes | `function(c)` — all file mutations |
| `name` | no | Display name |
| `description` | no | Short description |
| `args` | no | List of arg definitions (see below) |
| `maps` | no | Recipe-local map tables `{ mapId = { key = "value" } }` |
| `types` | no | Recipe-local named types `{ TypeId = { fields = { ... } } }` |
| `queries` | no | Inline query strings `{ queryId = "(...) @cap" }` |
| `postExecution` | no | Shell commands after successful apply (Jinja-rendered) |
| `explorerMenu` | no | VS Code Explorer submenu entries |
