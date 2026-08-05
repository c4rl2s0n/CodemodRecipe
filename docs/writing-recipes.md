# Writing recipes

How to author YAML recipes for codemod-recipe: the usual workflow, what
`query` / `capture` / `anchor` mean, step types, and where to go next.

Day-1 product overview: [getting-started.md](getting-started.md).
Complete field inventory (generated from the schema): [generated/dsl-vocabulary.md](generated/dsl-vocabulary.md).

## Authoring workflow

1. **Pick the target** — which file and AST node should change (method body,
   class member, import, …). Inspect the tree in a tree-sitter playground or by
   iterating with preview.
2. **Write a query** — S-expression pattern with `@names` for nodes you filter
   on and the node you will edit.
3. **Set `capture:`** — the `@name` whose **text span** the op edits (omit the
   `@` in YAML: `@body` → `capture: body`).
4. **For `insert` only** — set `anchor: start` or `end` (before or after that
   span).
5. **Fill `text` / `args`** — declare dynamics as recipe args (`{{name}}` in
   path, query, and text).
6. **Validate → preview → apply** — then re-preview; an idempotent recipe
   returns no patches.

Dart is the default language. For other files, set `language:` on the `edit`
step — see [language-support.md](language-support.md).

## Recipe shape

```yaml
id: feature.area.recipe_id    # dotted ids nest in the VS Code Recipes tab
name: Human-readable name     # optional
description: What it does     # optional
args: []                      # optional parameters
steps: []                     # required — ordered operations
postExecution:                # optional — after successful apply
  - "dart format ."
```

Every field and enum value: [generated/dsl-vocabulary.md](generated/dsl-vocabulary.md).

### Arguments (`args`)

```yaml
args:
  - name: file
    required: true
    inputKind: file
  - name: className
    required: true
    inputKind: symbol
```

Common `inputKind` values: `text`, `file`, `directory`, `choice`. Defaults use
`defaultsTo` (not `defaultValue`). In the VS Code extension, `from:` can derive
args from the active editor — see [recipe-shortcuts.md](recipe-shortcuts.md).

## Step types

Each entry in `steps` is a single-key object:

| Step | Role |
|------|------|
| `edit` | AST `insert` / `replace` / `remove` in an existing file |
| `create` | New file from inline `template` or `templateFile` |
| `delete` | Remove a file (`ifMissing: fail` \| `skip`) |
| `recipe` | Compose another recipe by id (optional `with:` bindings) |
| `if` | Conditional group of nested steps |

### Edit

```yaml
- edit:
    path: "{{file}}"
    language: dart          # optional for .dart; required when ambiguous
    when: …                 # optional — all must match or step is skipped
    whenNot: …              # optional — none may match
    let: …                  # optional — locals for later ops
    ops:
      - insert: { … }
```

### Create / delete

```yaml
- create:
    path: "lib/new_file.dart"
    template: "class X {}\n"
    ifExists: fail          # or skip

- delete:
    path: "lib/legacy.dart"
    ifMissing: skip         # or fail
```

File-backed templates use `templateFile` (recipe-local first, then `.codemod/`).
Jinja details: [recipe-templates.md](recipe-templates.md).

### Composition

```yaml
steps:
  - recipe: dart.settings.add_counter_field
  - recipe:
      id: dart.logging.add_log_line
      with:
        file: "{{file}}"
```

## Query, capture, and anchor

This is the mental model behind every AST op.

```text
source file  →  parse  →  query match  →  capture span  →  patch
```

### Captures in the query

In the tree-sitter query, `@name` labels a matched node:

```scheme
(class_definition
  name: (identifier) @className
  body: (class_body
    (function_body (block) @body))
  (#eq? @className "{{className}}"))
```

You often have **several** captures: some for filters (`#eq?`), one for the
edit target.

### The `capture` field

On the op, `capture:` picks **which** named match is edited. Use the name
**without** `@`:

| In query | On op |
|----------|--------|
| `(block) @body` | `capture: body` |
| `(declaration) @member` | `capture: member` |

`insert` / `replace` / `remove` all require `capture`. If the name is missing,
unmatched, or ambiguous, the run fails.

### The `anchor` field (insert only)

`anchor` chooses where `text` is inserted relative to the **captured node’s
byte span** — not tree-sitter’s `.` operator (that constrains sibling position
inside the pattern).

```text
  …[===== capture span =====]…
     ^                       ^
  anchor: start           anchor: end
```

| Goal | `anchor` |
|------|----------|
| Insert before the captured node | `start` |
| Insert after the captured node | `end` |

Example: capture a method’s `(block) @body` and `anchor: end` to append a
statement at the end of the body.

Full query language: [tree-sitter-queries.md](tree-sitter-queries.md).

## Edit ops

| Op | Required | Notes |
|----|----------|--------|
| `insert` | `query`, `capture`, `anchor`, `text` | Insert at capture boundary |
| `replace` | `query`, `capture`, `text` | Replace the capture span |
| `remove` | `query`, `capture` | Delete the capture span |

Optional on `replace` / `remove`: `includeLeadingTrivia` (e.g. leading doc
comments).

### Worked insert

```yaml
- edit:
    path: "{{file}}"
    ops:
      - insert:
          query: |
            (class_definition
              name: (identifier) @className
              body: (class_body
                (method_signature
                  (function_signature
                    name: (identifier) @methodName))
                (function_body
                  (block) @body))
              (#eq? @className "{{className}}")
              (#eq? @methodName "{{methodName}}"))
          capture: body
          anchor: end
          text: "    print('codemod');\n"
```

- `@className` / `@methodName` — filter which method
- `@body` + `capture: body` — edit the method body block
- `anchor: end` — insert after the closing of that block’s content span

### Insert before a member

Capture the member node and use `anchor: start`:

```yaml
capture: member
anchor: start
text: "  final int count = 0;\n"
```

## Guards, locals, and ordering

- **`when` / `whenNot`** — evaluated once on the file before any op; if guards
  fail, the edit step is skipped.
- **`let`** — bindings recomputed before each op; available in later op
  templates (see vocabulary for `extract`, `onNoMatch`, …).
- Steps and ops run **in order** against an in-memory tree; disk writes only on
  apply. `create` then `edit` on the same path is supported.

## Idempotency

Prefer recipes that produce **no patches** on a second apply:

- **replace** — idempotent when `text` is already the desired final state
- **remove** — idempotent when the target is already gone
- **insert** — often needs a `whenNot` (or a query that only matches when the
  snippet is absent) so a second run does nothing

## Naming and organization

| Prefix | Purpose |
|--------|---------|
| `create_` | Greenfield files from templates |
| `add_` / `patch_` | Brownfield AST edits (one change each) |
| `scaffold_` | Feature workflow (compose atomics) |
| `remove_` | Tear down members or files |

**One recipe = one coherent change.** Details:
[recipe-design-patterns.md](recipe-design-patterns.md).

## Validate and run

| Client | Flow |
|--------|------|
| VS Code | Recipes tab → fill args → review diffs → Apply Selected |
| MCP | `validate_recipes` → `preview_recipe` → `apply_recipe` with `previewToken` |

See [getting-started.md](getting-started.md#day-1-success-run-a-recipe) and
[codemod-mcp.md](codemod-mcp.md).

## Further reading

| Doc | Contents |
|-----|----------|
| [generated/dsl-vocabulary.md](generated/dsl-vocabulary.md) | Every DSL field and enum (generated) |
| [tree-sitter-queries.md](tree-sitter-queries.md) | Query syntax, captures, predicates |
| [recipe-templates.md](recipe-templates.md) | MiniJinja, maps, variables |
| [language-support.md](language-support.md) | `language:` and grammars |
| [recipe-design-patterns.md](recipe-design-patterns.md) | create / add / scaffold taxonomy |
| [recipe-shortcuts.md](recipe-shortcuts.md) | Slots, `from:`, explorer menu |
