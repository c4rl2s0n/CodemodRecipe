# Validation

Call `validate_recipes` (MCP), `{ "command": "validate" }` (stdio host), or
**Codemod Recipe: Validate Recipes** in VS Code.

| Code | Severity | Meaning |
|------|----------|---------|
| `E_UNDECLARED_ARG` | error | Template references unknown variable |
| `E_MISSING_TEMPLATE` | error | Template path not found (`c:template`) |
| `E_TEMPLATE_SYNTAX` | error | MiniJinja parse error |
| `E_COMPOSE_CYCLE` | error | Recipe reference cycle |
| `W_MAP_ID_NOT_FOUND` | warning | Unknown map id in template |
| `W_LEGACY_TEMPLATE` | warning | Legacy `{{$…}}` helper detected |

Diagnostics include optional `hint` and `relatedRecipe` fields.
