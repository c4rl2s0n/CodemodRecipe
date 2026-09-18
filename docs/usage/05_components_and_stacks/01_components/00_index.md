# Components (platform layer)

Components are Luau modules under `.codemod/components/` that **identify**, **parse**, **verify**, **scaffold**, and **mutate** abstract instances of code structures in your workspace. They complement recipes: recipes and component `scaffold` create or edit unbound; `operations` mutate an existing scanned target. Both use the same host sandbox `c` (no `c.args` — args are an explicit parameter).

| Page | |
|------|---|
| [Module shape](01_module_shape.md) | |
| [Hooks](02_hooks.md) | |
| [Parse tree](03_parse_tree.md) | |
| [Instance IDs](04_instance_ids.md) | |
| [Host APIs](05_host_apis.md) | |
| [Host commands](06_host_commands.md) | |
| [Layout](07_layout.md) | |
| [Reference fixtures](08_reference_fixtures.md) | |

