---
name: codemod-recipe-authoring
description: Practical guide for authoring codemod-recipe YAML v2 recipes — tree-sitter queries, captures, anchors, idempotency, and testing.
---

# Recipe Authoring Guide

## Start from a working pattern

1. Copy an existing recipe with a similar edit (insert field, insert log line, remove member).
2. Change `query` captures and `args` incrementally.
3. `validate_recipes` → `preview_recipe` → inspect patches → `apply_recipe`.

For recipe organization (create vs modify, scaffolds), see skill `codemod-recipe-design-patterns`.
For YAML syntax and templates, see skill `codemod-yaml-dsl-v2`.

## Crafting tree-sitter queries (Dart)

1. Identify the AST node to target (method body, class member, field declaration).
2. Name captures explicitly: `@className`, `@methodName`, `@body`, `@member`.
3. Filter with predicates: `(#eq? @className "{{className}}")`.
4. Set `capture:` to the node whose span you will edit.

### Insert at end of method body

- Query captures `(block) @body` inside the target method.
- `capture: body`, `anchor: end`
- `text` must include correct indentation and trailing newline.

### Insert before a class member

- Capture the member node: `... @member` on the `class_member`.
- `capture: member`, `anchor: start`

### Remove or replace a field

- Capture the whole `class_member` as `@member`.
- `remove` / `replace` use the member span (optionally `includeLeadingTrivia: true` for doc comments).

## Choosing anchors

| Goal | anchor |
|------|--------|
| Insert at start of capture span | `start` |
| Insert at end of capture span | `end` |

Only `insert` uses `anchor`. Omit for `replace` and `remove`.

## Idempotency

- **insert**: often non-idempotent unless query matches only when code is absent.
- **replace**: idempotent when `text` matches desired final state.
- **remove**: idempotent when target already gone (preview returns empty `files`).

After apply, re-run `preview_recipe` — expect `files: []` for idempotent recipes.

## Args

Declare all dynamic values as `args` and reference via `{{name}}` in path, query, and text.

Use `inputKind: file` for paths, `inputKind: symbol` for class/method names.

For template helpers and maps, see `codemod-yaml-dsl-v2` reference.md.

## Multi-file recipes

Combine steps in one recipe:

```yaml
steps:
  - create: { path: "...", template: "..." }
  - edit: { path: "...", ops: [...] }
  - delete: { path: "...", ifMissing: skip }
```

Or compose with `- recipe: other_recipe_id`.

## Testing checklist

1. `validate_recipes` — no schema errors
2. `preview_recipe` with realistic `args`
3. Review `files[].patches` / `replacementPreview`
4. `apply_recipe` with `previewToken`
5. Re-preview — confirm idempotency
6. Run `dart format` / analyzer if not using `postExecution: [dartFormat]`

## Common failures

| Symptom | Fix |
|---------|-----|
| `query matched no nodes` | Wrong class/method name or query structure |
| `query matched multiple nodes` | Tighten predicates with `#eq?` |
| Empty preview after apply | Expected if idempotent |
| `Stale previewToken` | Preview again after manual edits |

## Related skills

| Skill | Use for |
|-------|---------|
| `codemod-recipe-design-patterns` | create vs modify taxonomy |
| `codemod-yaml-dsl-v2` | YAML syntax, templates, maps |
| `codemod-mcp-playbook` | preview/apply workflow |
