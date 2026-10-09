# Query hits

| API | Shape |
|-----|--------|
| `c:query` + `capture` | Flat array of hits for **one** capture |
| `c:queryMatches` | One row per match; keys are all `@capture` names |

```lua
{ span = { from = 10, to = 20 }, text = "...", kind = "identifier" }
```

`from` / `to` are **0-based byte offsets** (Lua reserves `end`). Prefer `hit.text` over slicing the source yourself. For a span without a hit, use `c:spanText({ path, span })` — see [host context](../../02_writing_recipes/02_host_api/03_host_context_c.md).

`c:queryMatches` rows: one capture → a hit; multiple same-name captures → array of hits.
