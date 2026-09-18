# Where templates apply

Templates are rendered via **`c:template`** inside Luau recipes (MiniJinja). Typical uses:

| Call | Role |
|------|------|
| `c:template({ inline = "..." })` | Inline Jinja over recipe args + maps/vars |
| `c:template({ path = "templates/x.template" })` | File-backed template (`extends` / `include`) |
| `c:create({ content = c:template({ path = "..." }) })` | New file body from template |
| `postExecution` / `c:postExec` | Jinja-rendered shell commands |

Maps and variables registered under `.codemod/maps/` and `.codemod/variables/` (Luau modules)
merge into the MiniJinja context automatically.
