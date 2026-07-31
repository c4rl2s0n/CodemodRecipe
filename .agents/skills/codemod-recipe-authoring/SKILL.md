---
name: codemod-recipe-authoring
description: Practical guide for authoring codemod-recipe YAML recipes — tree-sitter queries, captures, anchors, idempotency, and testing. Dart examples use language-pack grammar (class_definition).
---

# Recipe Authoring Guide

## Start from a working pattern

1. Copy an existing recipe with a similar edit (insert field, insert log line, remove member).
2. Change `query` captures and `args` incrementally.
3. Set `language:` when editing non-Dart files (see `codemod-languages`).
4. `validate_recipes` → `preview_recipe` → inspect patches → `apply_recipe`.

For recipe organization (create vs modify, scaffolds), see skill `codemod-recipe-design-patterns`.
For YAML syntax and templates, see skill `codemod-yaml-dsl`.
For query language syntax (captures, predicates, operators), see skill `codemod-tree-sitter-queries`.

## Crafting tree-sitter queries (Dart — language-pack grammar)

Dart grammars from **tree-sitter-language-pack** use `class_definition` (not `class_declaration`) and place members directly under `class_body` (no `class_member` wrapper).

1. Identify the AST node to target (method body, declaration, field).
2. Name captures explicitly: `@className`, `@methodName`, `@body`, `@member`.
3. Filter with predicates: `(#eq? @className "{{className}}")`.
4. Set `capture:` to the node whose span you will edit.

### Insert at end of method body

- Query captures `(block) @body` inside the target method’s `function_body`.
- `capture: body`, `anchor: end`
- `text` must include correct indentation and trailing newline.

```yaml
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
```

### Insert before a class member

- Capture `method_signature` (or other member) as `@member`.
- `capture: member`, `anchor: start`

### Remove or replace a field

- Capture the `declaration` node as `@member`.
- `remove` / `replace` use the member span (optionally `includeLeadingTrivia: true` for doc comments).

```yaml
query: |
  (class_definition
    name: (identifier) @className
    body: (class_body
      (declaration
        (initialized_identifier_list
          (initialized_identifier
            (identifier) @fieldName))) @member)
    (#eq? @className "Settings")
    (#eq? @fieldName "count"))
```

## Other languages

Set `edit.language` and author queries for that grammar’s node names. Queries do not port across languages. See skill `codemod-languages`.

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

Use `inputKind: file` for file paths, `inputKind: directory` for folders, and `inputKind: choice` with `options` for fixed value lists.

For template helpers and maps, see `codemod-yaml-dsl` reference.md.

## Multi-file recipes

Combine steps in one recipe:

```yaml
steps:
  - create: { path: "...", template: "..." }
  - edit: { path: "...", language: dart, ops: [...] }
  - delete: { path: "...", ifMissing: skip }
```

Or compose with `- recipe: other_recipe_id`, or bind child args via
`- recipe: { id: other_recipe_id, with: { ... } }`.

## Testing checklist

1. `validate_recipes` — no schema errors (including unknown `language`)
2. `preview_recipe` with realistic `args`
3. Review `files[].patches` / `replacementPreview`
4. `apply_recipe` with `previewToken`
5. Re-preview — confirm idempotency
6. Run `dart format` / analyzer if not using `postExecution: ["dart format ."]`

## Common failures

| Symptom | Fix |
|---------|-----|
| `Invalid node type "class_declaration"` | Use `class_definition` (pack Dart grammar) |
| `query matched no nodes` | Wrong class/method name, wrong language, or query structure |
| `query matched multiple nodes` | Tighten predicates with `#eq?` |
| `language not supported` | Set valid `language:` id (see `codemod-languages`) |
| `file type not supported` | Set `language:` or use a known extension |
| Empty preview after apply | Expected if idempotent |
| `Stale previewToken` | Preview again after manual edits |

## Related skills

| Skill | Use for |
|-------|---------|
| `codemod-tree-sitter-queries` | query syntax, captures, predicates, `.scm` files |
| `codemod-languages` | language ids, SQL dialects, extension inference |
| `codemod-recipe-design-patterns` | create vs modify taxonomy |
| `codemod-yaml-dsl` | YAML syntax, templates, maps |
| `codemod-mcp-playbook` | preview/apply workflow |
| `recipe-generation` | Generate recipes from `@` code refs |
