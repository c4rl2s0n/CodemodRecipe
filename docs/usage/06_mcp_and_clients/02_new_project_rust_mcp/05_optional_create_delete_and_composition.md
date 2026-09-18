# Optional: create, delete, and composition

```lua
-- create
c:create({ path = "lib/new.dart", content = c:template({ path = "templates/x.template" }) })

-- delete
c:delete({ path = args.legacyPath, ifMissing = "skip" })

-- compose
c:run("other.recipe.id", { file = args.file })
```
