# Query hits

`c:query` returns hits shaped:

```lua
{ span = { from = 10, to = 20 }, text = "...", kind = "identifier" }
```

(`from` / `to` because Lua reserves `end`.)

`c:queryMatches` returns one row per tree-sitter match; keys are `@capture` names. One capture → a hit; multiple same-name captures → array of hits.
