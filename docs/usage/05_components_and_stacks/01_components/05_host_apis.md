# Host APIs

| Method | Purpose |
|--------|---------|
| `c:listFiles(pattern)` | Workspace-relative `*` glob → path strings |
| `c:matchFiles(pattern)` | `{placeholder}` path pattern → `{ path, …captures }` rows |
| `c:filename(path)` | Basename without extension |
| `c:resolve(template, vars?)` | Replace `{key}` in a path template (no FS / no MiniJinja; use `c:template` for content) |
| `c:query` / `c:queryMatches` | Hits `{ span = { from, to }, text, kind }` / per-match capture rows |
| `c:component(id)` | Delegate with `:parse`, `:verify`, `:readSource` (cycle-safe) |

All recipe host APIs (`c:read`, `c:insert`, `c:run`, maps/vars, …) are available in component hooks. Recipe and nested `c:run` use `apply(c, args)`.
