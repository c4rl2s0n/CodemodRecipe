# Path helpers

| Method | Pattern | Returns |
|--------|---------|---------|
| `c:listFiles("lib/models/*.dart")` | shell `*` / `**` globs | list of path strings |
| `c:listFiles({ "lib/**/*.dart", "!**/generated/**" })` | same + `!` excludes (union then subtract) | list of path strings |
| `c:matchFiles("lib/models/{name}.dart")` | `{placeholder}` (one segment each; not `*` globs) | `{ path, name, ... }` rows |
| `c:resolve(tmpl, vars)` | `{key}` fill-in | **one** path string (no FS search) |
| `c:filename(path)` | — | basename without extension |

`listFiles` and `stack.codebase` share `resolve_globs`: includes union, then any `!pattern` drops matching paths (excludes win; order irrelevant).
