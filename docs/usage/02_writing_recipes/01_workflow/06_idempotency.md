# Idempotency

Prefer recipes that produce **no patches** on a second apply:

- **replace** — idempotent when `text` is already the desired final state
- **remove** — idempotent when the target is already gone
- **insert** — guard with `c:queryExists` / `c:queryCount` or a query that only
  matches when the snippet is absent

```lua
if c:queryExists({ path = args.file, query = alreadyThereQuery }) then
  return
end
c:insert({ ... })
```
