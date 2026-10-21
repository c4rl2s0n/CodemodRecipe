# Author a component (minimum path)

Read **these four pages** before writing a new component. Skip the rest of the tree until you need relations, coverage, or fixtures.

1. **[Module shape](01_module_shape.md)** — `id` + `parse` required; `identify` only for stack roots
2. **[Hooks](02_hooks.md)** — phase order and signatures (**canonical** apply forms)
3. **[Apply signatures](../03_component_contract/02_apply_signatures_recipes_components.md)** — recipe vs scaffold vs operation
4. **[Hook signatures: `c` and locator](../03_component_contract/03_hook_signatures_c_and_locator.md)** — what `c` is / is not

## Canonical apply signatures

| Surface | Signature |
|---------|-----------|
| Recipe | `apply(c, args)` |
| Scaffold | `scaffold.apply(c, args, graph)` — `graph` is the pre-op forest snapshot (may be unused) |
| Operation | `operations.<name>.apply(c, args, target, graph)` |

Do **not** invent `c.args`. Args are always the explicit `args` table.

## Checklist

1. Place under `.codemod/components/` (or a nested path); `return { id = "…", parse = … }`
2. If it is a **stack root**, add `identify` and list the id in `stack.components`
3. `validate_recipes` → `component_dry_run_hook` (`identify` → `parse` → op) → inspect `files` / `logs`
4. Apply only with `previewToken` from dry-run (`component_apply_hook`) when the user asks

Stack fan-out ops: [stacks](../02_stacks/00_index.md). Relations when you need edges: [relations](10_relations.md).
