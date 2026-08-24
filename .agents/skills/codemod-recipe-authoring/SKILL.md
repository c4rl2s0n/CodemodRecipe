---
name: codemod-recipe-authoring
description: Practical guide for authoring codemod-recipe Luau recipes — tree-sitter queries, captures, anchors, idempotency, and testing. Dart examples use language-pack grammar (class_definition).
---

# Recipe Authoring Guide

## Start from a working pattern

1. Copy an existing `.luau` recipe with a similar edit.
2. Change `query` captures and `args` incrementally.
3. Set `language` on edit ops when editing non-Dart files (see `codemod-languages`).
4. `validate_recipes` → `preview_recipe` → inspect patches + transcript → `apply_recipe`.

For recipe organization, see skill `codemod-recipe-design-patterns`.
For host API and module shape, see skill `codemod-luau-recipes`.
For query syntax, see skill `codemod-tree-sitter-queries`.

Human guide: [docs/writing-recipes.md](../../../docs/writing-recipes.md).

## Crafting tree-sitter queries (Dart — language-pack grammar)

Dart grammars from **tree-sitter-language-pack** use `class_definition` (not `class_declaration`).

1. Identify the AST node to target.
2. Name captures: `@className`, `@methodName`, `@body`, `@member`.
3. Filter with predicates: `(#eq? @className "Settings")`.
4. Pass `capture` on the op spec.

### Insert at end of method body

```lua
c:insert({
  path = c.args.file,
  query = [[
    (class_definition
      name: (identifier) @className
      body: (class_body
        (method_signature
          (function_signature
            name: (identifier) @methodName))
        (function_body
          (block) @body))
      (#eq? @className "]] .. c.args.className .. [[")
      (#eq? @methodName "]] .. c.args.methodName .. [["))
  ]],
  capture = "body",
  anchor = "end",
  text = "    print('codemod');\n",
})
```

### Remove or replace a field

Capture the `declaration` node as `@member`; use `c:remove` or `c:replace`.

## Other languages

Set `language = "rust"` (etc.) on the edit spec. Queries do not port across languages.

## Idempotency

Guard inserts with `c:queryExists` / `c:queryCount` or queries that match only when code is absent.

After apply, re-run `preview_recipe` — expect `files: []` for idempotent recipes.

## Args

Declare dynamics in `args`; reference as `c.args.name` or via `c:applyTemplate`.

## Multi-file recipes

Use sequential calls in `apply`:

```lua
apply = function(c)
  c:create({ path = "...", content = "..." })
  c:insert({ path = "...", query = "...", capture = "...", anchor = "end", text = "..." })
  c:delete({ path = "...", ifMissing = "skip" })
end
```

Or compose with `c:run("other.recipe.id", { ... })`.

## Testing checklist

1. `validate_recipes`
2. `preview_recipe` with realistic args
3. Review `files` and `transcript`
4. `apply_recipe` with `previewToken`
5. Re-preview for idempotency

## Related skills

| Skill | Use for |
|-------|---------|
| `codemod-luau-recipes` | host API, module shape |
| `codemod-tree-sitter-queries` | query syntax |
| `codemod-languages` | language ids |
| `codemod-recipe-design-patterns` | taxonomy |
| `codemod-mcp-playbook` | preview/apply |
| `recipe-generation` | generate from `@` refs |
