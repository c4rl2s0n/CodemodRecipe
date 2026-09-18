# Global types (definition files)

`.codemod/stubs/codemod-context.d.luau` exposes ambient types such as `CodemodContext`, `QueryCaptureSpec`, `QueryTableSpec`, `InsertSpec`, `QueryHit`, …

**Recipes:**

```lua
--!strict

return {
  id = "my.recipe",
  apply = function(c: CodemodContext)
    c:query({ path = "lib/foo.dart", query = "...", capture = "x" })
  end,
}
```

**Components** — same `c` type on every hook:

```lua
--!strict

return {
  id = "my.component",
  parse = function(c: CodemodContext, context: MyParseContext): MyNode?
    -- ...
  end,
  verify = function(c: CodemodContext, instance: ComponentInstance): { Finding }
    return {}
  end,
}
```
