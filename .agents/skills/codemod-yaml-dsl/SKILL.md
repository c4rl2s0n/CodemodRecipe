---
name: codemod-yaml-dsl
description: YAML DSL for codemod-recipe — recipe structure, step types, tree-sitter edit ops, maps, and variables. Read reference.md for templates, maps/variables, composition, and validation.
---

# YAML DSL

## When to use

- Writing or reviewing `.codemod/recipes/*.yaml`
- Crafting `inlineRecipe` payloads for MCP preview/apply
- Understanding `edit`, `create`, `delete`, and `recipe` step types

## When not to use

- Recipe organization (create vs modify) → `codemod-recipe-design-patterns`
- Tree-sitter query syntax → `codemod-tree-sitter-queries`
- Dart-specific query patterns → `codemod-recipe-authoring`
- Language IDs, SQL dialects, extension inference → `codemod-languages`
- Running MCP tools → `codemod-mcp-playbook`

## Top-level shape

```yaml
id: feature.area.recipe_id    # dotted ids define nested groups in the VS Code extension
args: []
maps: {}
steps: []
postExecution:
  - "dart format ."
```

## Step types

Each `steps[]` entry is a single-key object:

- `edit` — patch existing files (insert/replace/remove); optional `when` / `whenNot` guards and `let` step locals (see `reference.md`)
- `create` — new files from template
- `delete` — remove files
- `recipe` — compose another recipe by id (optional `with:` call-site arg bindings)

## Minimal insert example

```yaml
- edit:
    path: "{{file}}"
    language: dart   # optional for .dart; required for ambiguous paths
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

## Instructions

1. **Read and follow** [reference.md](reference.md) for templates, maps, variables, args, composition, and validation.
2. Call `validate_recipes` after editing recipe YAML.
3. When adjusting Rust parsing/validation/preview handling for YAML vocabulary, update the centralized owners first:
   - `rust/crates/yaml/src/keywords.rs` for DSL keys, query path conventions, and preview kinds
   - `rust/crates/host/src/protocol_keys.rs` for host transport keys like `inlineRecipe` and `previewToken`

## Related skills

| Skill | Use for |
|-------|---------|
| `codemod-recipe-design-patterns` | create vs modify taxonomy |
| `codemod-languages` | `language:` field and multi-language support |
| `codemod-tree-sitter-queries` | query syntax, captures, predicates |
| `codemod-recipe-authoring` | Dart query patterns and testing |
| `codemod-mcp-playbook` | preview/apply workflow |
