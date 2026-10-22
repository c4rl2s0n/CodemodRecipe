# Explorer context menu

Right-click a file or folder in the Explorer → **Codemod Recipe** submenu:

| Menu item | Behavior |
|-----------|----------|
| **Run Recipe Here…** | `mode: auto` — preview→apply when required args are complete; otherwise open the runner |
| **Open in Recipe Runner…** | `mode: open` — always open the runner with the same prefilled args |

Recipes opt in with top-level `explorerMenu` (list of entries; a single object is sugar).
Absent → never listed. Shortcuts and slots ignore this field.

VS Code only passes the clicked URI into the command; all expression context below is
derived from that URI plus the workspace root.

```lua
explorerMenu = {
  {
    kind = "file",
    ["if"] = [[path is startingwith("lib/") and fileExt == "dart"]],
    args = {
      file = "path",
      folderName = "fileBasename",
      featureDir = "fileDirname",
      absFile = "absolutePath",
    },
  },
}
```

Match rules for click kind `K` and path `P`:

1. Consider entries with `kind == K`.
2. Entry matches if `if` is omitted **or** evaluates truthy over the expression context.
3. If **any** entry matches → recipe appears **once** in the QuickPick (OR).
4. The **first** matching entry (list order) supplies arg bindings.
5. Expression errors on `if` or `args` fail closed for that entry / match.

## Expression context

| Name | Meaning |
|------|---------|
| `path` | Workspace-relative click path |
| `absolutePath` | Absolute click path |
| `workspaceRoot` | Absolute workspace root |
| `fileBasename` | Final path component |
| `fileStem` | Basename without extension |
| `fileExt` | Extension without leading `.` |
| `fileDirname` | Parent of `path` (workspace-relative; folder click uses the folder itself) |
| `file` | Same as `path` when kind is file; empty for folder |
| `directory` | Click path when folder; parent dir when file |

Same MiniJinja dialect as other `if` expressions (comparisons, `and` / `or` / `not`,
`| basename` / `| parent` / `| stem`, `is startingwith`). Do not encode
file-vs-folder in `if` — use separate entries.

**Arg bindings (`args` map):** keys are recipe arg names; values are expressions
over the context above (recipe args are not in scope on the RHS).
Example: `path = "path | parent"` sets recipe arg `path` to the parent of the click path.

String concat uses Jinja `~`. Filters bind tighter than `~`, so prefer parentheses or
named builtins:

```lua
sibling = [[fileDirname ~ "/" ~ fileStem ~ "_test.dart"]]
-- or: (path | parent) ~ "/" ~ (path | stem) ~ "_test.dart"
```

When `args` is present on the winning entry, those rendered values are used.
When absent, prefill falls back to first `inputKind: directory` (folder) or
`inputKind: file` (file) ← `path`. Then the usual `from` / `invoke` path.

Empty match list → info toast (no full-catalog fallback).
