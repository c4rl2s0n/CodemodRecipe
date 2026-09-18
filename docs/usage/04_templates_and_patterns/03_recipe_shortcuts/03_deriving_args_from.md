# Deriving args (`from`)

Put derivation on the **recipe**, not in keybindings. Every entry point (shortcut,
CodeLens, context menu) shares the same rules.

## Builtin keys (string form)

```lua
args = {
  { name = "file", required = true, from = "file" },
  { name = "className", from = "word" },
}
```

`contextKey` is a deprecated alias of string `from`.

| Builtin | Meaning |
|---------|---------|
| `file` | Workspace-relative path (or absolute if outside root) |
| `fileBasename` | Filename with extension |
| `fileDirname` | Directory of `file` |
| `fileStem` | Filename without extension |
| `fileExt` | Extension without leading `.` |
| `selection` | Selected text |
| `word` | Word at cursor |
| `line` | Full text of the active line |
| `lineNumber` | 1-based line number (string) |
| `languageId` | VS Code language id |

## Path / template form

Evaluated against the builtin map (simple `{{ name }}` / `{{ name \| filter }}` in
the extension; host `deriveArgs` uses full MiniJinja when query forms are involved):

```lua
{ name = "feature", from = { template = "{{ fileDirname | basename }}" } }
```

Filters available in the extension shortcut path: `basename`, `dirname`, `stem`.

## Tree-sitter form

```lua
{
  name = "className",
  required = true,
  from = {
    query = [[
      (class_definition
        name: (identifier) @name)
    ]],
    capture = "name",
    extract = "text",          -- text | kind | exists | count
    scope = "enclosing",       -- enclosing | selection | first (default: enclosing)
    language = "dart",         -- optional; else editor language / file extension
    onNoMatch = "omit",        -- omit | empty — omit leaves arg unset → runner
  },
}
```

| `scope` | Match selection |
|---------|-----------------|
| `enclosing` | Captures whose range contains the cursor; tightest wins |
| `selection` | Captures intersecting the selection |
| `first` | First match in the file |

Query evaluation runs in the Rust host (`deriveArgs`). If the host is down or the
query fails, that arg is left unset (partial → open runner).
