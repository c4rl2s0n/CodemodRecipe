# Path helpers

| Method | Pattern | Returns |
|--------|---------|---------|
| `c:listFiles("lib/models/*.dart")` | shell `*` globs | list of path strings |
| `c:matchFiles("lib/models/{name}.dart")` | `{placeholder}` (one segment each) | `{ path, name, ... }` rows |
| `c:resolve(tmpl, vars)` | `{key}` fill-in | **one** path string (no FS search) |
| `c:filename(path)` | — | basename without extension |
