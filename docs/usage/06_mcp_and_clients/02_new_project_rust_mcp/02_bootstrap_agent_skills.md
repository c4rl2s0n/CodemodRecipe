# Bootstrap agent skills

After connecting the MCP server, call **`bootstrap_project`** once. It installs:

- `.agents/skills/codemod-recipe/` — Architect / Builder skill indexes plus `docs/` (usage manual)
- `.cursor/rules/codemod-recipe.mdc` — always-on orientation
- `.codemod/recipes/`, `.codemod/maps/`, `.codemod/types/`, `.codemod/variables/` scaffolding

```json
{ "force": false, "edit_policy": "recommend", "companions": [] }
```
