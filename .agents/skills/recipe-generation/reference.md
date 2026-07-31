# Recipe Generation — Reference

## Parameter heuristics

| Kind of value in the selection | Treat as | `inputKind` |
|--------------------------------|----------|-------------|
| File path the edit targets | Arg `file` (or path arg) | `file` |
| Class / method / field / type name that **locates** the edit | Arg + `{{name}}` in query predicates | `symbol` |
| Payload that should vary across runs (inserted text, message, type literal, field name being added) | Required arg + `{{…}}` in `text` / template | `symbol` or omit kind |
| Stable AST / grammar shape (node types, structural anchors) | Hardcode in `query` | — |
| Project convention that never changes | Hardcode or map filter | — |

Prefer parameterizing identifiers the user would change next time. Prefer hardcoding grammar structure.

When unsure whether a literal is a parameter, include it in the confirmation table as a proposed arg and let the user drop it.

### Naming

- Paths: `file` (or `path`) when a single edit target
- Locators: `className`, `methodName`, `fieldName`, …
- Payload: domain names (`eventName`, `message`, …) — not vague `value` / `text` unless that is the domain

Declare every dynamic value under `args` and reference with `{{name}}` (and filters like `| camel_case` when needed).

## Reuse / compose / new decision tree

```text
list_recipes → skim descriptions
  │
  ├─ One recipe does the full change?
  │    → REUSE: propose id + mapped args from selection
  │
  ├─ Several atomics together cover the change?
  │    → COMPOSE: scaffold_<feature> with - recipe: child_id steps
  │      Args = union of children; pass through
  │
  └─ Otherwise
       → NEW atomic (one coherent change)
         create_  | greenfield file(s)
         add_ / patch_  | brownfield insert/replace
         remove_  | tear down member or file
```

Do not invent a near-duplicate of an existing recipe. Prefer extending composition or documenting reuse.

If the user referenced multiple files that are one feature workflow, prefer `scaffold_*` over one mega-recipe with unrelated edits.

## Confirmation template

Present this (adapt fields) and wait for approval before writing:

```markdown
## Recipe proposal

- **Decision:** reuse | compose | new
- **Recipe id(s):** …
- **Path:** `.codemod/recipes/…` (omit if reuse-only)
- **Prefix / kind:** create_ | add_ | patch_ | remove_ | scaffold_
- **Language:** dart | … (if non-Dart)

### Args

| Name | inputKind | Sample from selection | Required |
|------|-----------|------------------------|----------|
| file | file | lib/foo.dart | yes |
| className | symbol | Foo | yes |

### Steps (sketch)

- edit / create / delete / recipe / if: …

OK to write, or tell me which args to drop/rename?
```

For **reuse**, replace “Steps” with the existing recipe id and how selection maps to its args.

For **compose**, list child recipe ids and the orchestrator `scaffold_*` id.

## After confirmation

1. Write YAML only if decision is compose or new (and orchestrator/children as needed).
2. `validate_recipes`
3. `preview_recipe` with the sample args from the table
4. Stop — apply only if the user asks

## Related skills

- `codemod-recipe-design-patterns` — taxonomy and directory layout
- `codemod-yaml-dsl` — YAML / templates / composition
- `codemod-recipe-authoring` — Dart queries and idempotency
- `codemod-mcp-playbook` — MCP tool envelopes
