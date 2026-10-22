# Host API

Access arguments as `args.name`. All mutation methods are on `c`:

## AST edits

```lua
c:insert({
  path = args.file,
  language = "dart",       -- optional when extension is unambiguous
  query = "(class_definition ...) @body",
  capture = "body",
  anchor = "end",          -- "start" | "end" (default "end")
  text = "    print('codemod');\n",
})

c:replace({ path = args.file, query = "...", capture = "member", text = "  final int x = 0;\n" })
c:remove({ path = args.file, query = "...", capture = "member", includeLeadingTrivia = true })
```

## File create/delete

```lua
c:create({
  path = "lib/new_file.dart",
  content = c:template({ path = "templates/widget.dart.template" }),
  ifExists = "fail",   -- or "skip"
})

c:delete({ path = "lib/legacy.dart", ifMissing = "skip" })
```

## Read-only helpers

```lua
local text = c:read(args.file)
local spans = c:query({ path = args.file, query = "...", capture = "body" })
if c:queryExists({ path = args.file, query = "..." }) then return end
```

## Composition

```lua
c:run("dart.settings.add_counter_field")
c:run("dart.logging.add_log_line", { file = args.file, className = args.className })
```

## Templates

```lua
local rendered = c:template({
  inline = "class {{ name | pascal_case }} {}\n",
  env = { name = args.className },
})
```

See [recipe-templates.md](../../04_templates_and_patterns/01_recipe_templates/00_index.md).

## Logging

```lua
c:log("args", args)
```

Log lines appear in the Runner transcript. On preview/apply failure, the partial transcript (including logs) is still returned.

## Post-execution

Top-level `postExecution` or inline:

```lua
c:postExec({ command = "dart format {{file}}" })
```

Preview records postExec in the **transcript** but does not run it. Apply flushes
the WorkingTree first, then executes commands.
