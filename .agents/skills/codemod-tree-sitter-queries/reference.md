# Tree-sitter query language reference

Tree-sitter queries are **S-expression patterns** matched against a parsed syntax tree. codemod-recipe runs them per `edit` step using the grammar for `edit.language` (or inferred from the file extension).

This guide condenses the [official tree-sitter query documentation](https://tree-sitter.github.io/tree-sitter/using-parsers/queries/1-syntax.html) for recipe authoring. For grammar-specific node names, see skill `codemod-languages`.

Human-oriented copy also lives in [docs/tree-sitter-queries.md](../../../../docs/tree-sitter-queries.md) and [docs/writing-recipes.md](../../../../docs/writing-recipes.md).

---

## Overview

A query is one or more patterns. Each pattern is `(node_type child1 child2 …)` where children are nested patterns, captures, predicates, or operators.

```text
source file  →  tree-sitter parse  →  syntax tree  →  query match  →  capture span  →  codemod patch
```

Official chapters:

| Topic | URL |
|-------|-----|
| Basic syntax | https://tree-sitter.github.io/tree-sitter/using-parsers/queries/1-syntax.html |
| Operators | https://tree-sitter.github.io/tree-sitter/using-parsers/queries/2-operators.html |
| Predicates | https://tree-sitter.github.io/tree-sitter/using-parsers/queries/3-predicates-and-directives.html |

---

## Basic patterns

### Node types

Match a named node and optionally its children:

```scheme
(function_item)
(function_item (identifier) @name)
```

Omit children to match any node of that type with any children:

```scheme
(class_definition (identifier))   ; class with at least one identifier child somewhere
```

### Fields

Grammar fields make patterns precise. Prefix a child with `field_name:`:

```scheme
(class_definition
  name: (identifier) @className
  body: (class_body) @body)
```

### Negated fields

Require that a field is **absent**:

```scheme
(class_definition
  name: (identifier) @name
  !type_parameters)
```

### Anonymous nodes

Literal tokens use double quotes:

```scheme
(binary_expression operator: "!=" right: (null))
```

### Wildcards

| Pattern | Matches |
|---------|---------|
| `(_)` | any named node |
| `_` | any named or anonymous node |

```scheme
(call_expression function: (_) @callee)
```

### Special nodes

| Pattern | Meaning |
|---------|---------|
| `(ERROR)` | unparseable text |
| `(MISSING)` | recovered missing token |
| `(MISSING identifier)` | specific missing kind |
| `(expression)` | supertype — any expression subtype |
| `(expression/call_expression)` | call only when parent is supertype `expression` |

---

## Captures

Attach `@name` after a matched node:

```scheme
(identifier) @className
(block) @body
```

A single pattern may define **multiple captures**. Use extra captures for filtering (`#eq?`) and pick the edit target with the op’s `capture:` field:

```yaml
query: |
  (class_definition
    name: (identifier) @className
    body: (class_body
      (declaration) @member)
    (#eq? @className "Settings"))
capture: member    # edits @member, not @className
```

### codemod capture semantics

For each `insert` / `replace` / `remove` op:

1. The engine runs the query against the file.
2. It collects spans for the capture named in `capture:` (without the `@`).
3. **0 matches** → op skipped or preview shows empty `files`.
4. **1 match** → span is edited.
5. **2+ matches** → error (`query matched multiple nodes`) unless the engine takes the first (prefer tightening the query).

Always name captures explicitly (`@className`, `@body`, `@member`) — never rely on implicit ordering.

---

## Operators

### Quantifiers

Postfix on a pattern (like regex):

| Op | Meaning |
|----|---------|
| `+` | one or more repetitions |
| `*` | zero or more |
| `?` | optional |

```scheme
(comment)+
(class_definition (decorator)* @decorators name: (identifier) @name)
(call_expression function: (identifier) @fn arguments: (arguments (string)? @arg))
```

### Grouping

Parentheses group sibling sequences:

```scheme
(
  (comment)
  (function_definition)
)
```

Quantifiers apply to groups: `((comment) (function_definition))+`

### Alternation

Square brackets — match any alternative:

```scheme
(call_expression
  function: [
    (identifier) @function
    (member_expression property: (property_identifier) @method)
  ])
```

Anonymous token alternation:

```scheme
["if" "else" "return" "while"] @keyword
```

### Query anchor `.` (not codemod anchor)

The `.` operator constrains **position in the syntax tree** — first/last child or immediate siblings. This is **different** from codemod `anchor: start|end` on `insert` ops.

| Position | Example | Effect |
|----------|---------|--------|
| First child | `(array . (identifier) @first)` | `@first` only if first named child |
| Last child | `(block (_) @last .)` | `@last` only if last named child |
| Siblings | `(dotted_name (identifier) @a . (identifier) @b)` | consecutive siblings only |

**codemod `anchor: start|end`** chooses whether inserted `text` goes at the **byte start or end** of the captured node span. Only `insert` uses it.

---

## Predicates

Predicates filter matches. They are S-expressions starting with `#` and ending with `?`, placed inside the pattern:

```scheme
(class_definition
  name: (identifier) @className
  (#eq? @className "Settings"))
```

The Rust engine uses standard `tree_sitter::Query` — predicates supported by the tree-sitter CLI work in recipes.

### Equality

| Predicate | Meaning |
|-----------|---------|
| `#eq?` | capture text equals string or another capture |
| `#not-eq?` | inverse |
| `#any-eq?` | for quantified captures: any node matches |
| `#any-not-eq?` | for quantified captures: any node does not match |

```scheme
(#eq? @className "{{className}}")
(#eq? @a @b)
```

After template rendering, `{{className}}` becomes a literal string inside the query.

### Regular expressions

| Predicate | Meaning |
|-----------|---------|
| `#match?` | capture text matches regex |
| `#not-match?` | inverse |
| `#any-match?` / `#any-not-match?` | quantified variants |

```scheme
(identifier) @name (#match? @name "^_[a-z]+$")
```

### Lists and properties

| Predicate | Meaning |
|-----------|---------|
| `#any-of?` | capture equals one of several strings |
| `#is?` | node has a semantic property (grammar-specific) |
| `#is-not?` | node lacks a property |

```scheme
(identifier) @id (#any-of? @id "foo" "bar" "baz")
```

### Directives (rare in recipes)

Directives end with `!` (`#set!`, `#strip!`) and attach metadata. codemod recipes today use **predicates only** for filtering — directives are not required for insert/replace/remove.

---

## Codemod integration

### `query` field

Inline (multiline YAML recommended):

```yaml
query: |
  (class_definition
    name: (identifier) @className
    body: (class_body (block) @body)
    (#eq? @className "{{className}}"))
```

Or path to a `.scm` file (using the shared resolver in
`rust/crates/core/src/resource_path.rs`):

```yaml
query: settings_update_body.scm
```

Search order (engine): recipe directory → `queries/` under recipe →
`.codemod/` → `.codemod/queries/`. YAML query libraries remain id-based under
`.codemod/queries/*.yaml`; they are not loaded directly via `query: file.yaml`.

### Template substitution

Recipe args (`{{ name }}`, `{{ field | camel_case }}`, maps) are expanded into `query`, `capture`, and `text` **before** the query is parsed. Use predicates with rendered literals:

```yaml
args:
  - name: className
    required: true
query: |
  (class_definition
    name: (identifier) @className
    (#eq? @className "{{className}}"))
```

Do not try to parameterize predicate names — only values.

### `capture` + `anchor` on ops

| Op | `capture` | `anchor` |
|----|-----------|----------|
| `insert` | required — node to insert beside/inside | `start` or `end` of capture span |
| `replace` | required — node span replaced by `text` | n/a |
| `remove` | required — node span deleted | n/a |

Optional `includeLeadingTrivia: true` on `replace`/`remove` expands the span over leading `///` or `//` comments (language-dependent; see `codemod-languages`).

### Parse errors

If the source file has syntax errors (`has_error()` on the tree root), the engine rejects the edit before matching. Fix or exclude broken files.

### Syntax errors in the query itself

Invalid node types for the grammar fail at preview time:

```text
Invalid node type "class_declaration"
```

Use node names from the grammar for `edit.language` (pack Dart uses `class_definition` — see `codemod-languages`).

---

## Authoring workflow

1. **Pick language** — set `edit.language` or rely on extension (`codemod-languages`).
2. **Inspect AST** — tree-sitter playground or `tree-sitter parse` / grammar docs for node kinds.
3. **Write pattern** — start broad, add `field:` and `@captures`.
4. **Filter** — add `#eq?` / `#match?` until one match per file.
5. **Choose edit capture** — set `capture:` to the node you will insert beside, replace, or remove.
6. **Validate** — `validate_recipes` (schema + language id).
7. **Preview** — `preview_recipe`; inspect `files[].patches`.
8. **Idempotency** — re-preview after apply; expect empty `files` when done (`codemod-recipe-authoring`).

---

## Examples

### Insert at end of method body (Dart)

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
capture: body
anchor: end
text: "    print('codemod');\n"
```

### Remove a field by name (Dart)

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
capture: member
```

### Rust — any function item

```yaml
language: rust
query: "(function_item) @fn"
capture: fn
anchor: end
text: "\n// codemod\n"
```

### External `.scm` file

`settings_update_body.scm`:

```scheme
(class_definition
  name: (identifier) @className
  body: (class_body
    (method_signature
      (function_signature
        name: (identifier) @methodName))
    (function_body
      (block) @body))
  (#eq? @className "Settings")
  (#eq? @methodName "update"))
```

Recipe:

```yaml
- insert:
    query: settings_update_body.scm
    capture: body
    anchor: end
    text: "    print('codemod');\n"
```

---

## Troubleshooting

| Symptom | Likely cause | Fix |
|---------|--------------|-----|
| `Invalid node type "…"` | Wrong node name for grammar | Use **Query Tools** / `dump_ast`; check `codemod-languages` |
| `query matched no nodes` | Typo, wrong language, or predicates too strict | Run `debug_query` / Query Tools; loosen predicates; verify `language:` |
| `query matched multiple nodes` | Pattern too broad | Add `#eq?`, `field:`, or structural constraints |
| Confused insert position | Mixed up anchors | tree-sitter `.` = tree position; codemod `anchor` = byte offset in capture |
| `Missing capture` | `capture:` name not in query | Align `capture:` with an `@name` in the query (recipe requires explicit `capture:`) |
| `query file not found` | Bad `.scm` path | Place file next to recipe or under `.codemod/queries/` |
| Predicate ignored | Wrong capture or quantifier | Ensure predicate is inside the pattern that owns the capture |
| Preview looks like full-file wipe | Patch is whole-file replace | Use Query Tools to see capture spans; not a capture bug by itself |
| AST tree stale while editing | Query AST refreshes on save / view open, not every keystroke | Save to refresh tree; Run/Generate still use live buffer (host caches parse by hash) |

---

## Related skills

| Skill | Use for |
|-------|---------|
| `codemod-languages` | language ids, per-grammar node names, SQL dialects |
| `codemod-recipe-authoring` | Dart patterns, idempotency, testing checklist |
| `codemod-yaml-dsl` | `edit.ops` schema, templates, maps |
| `codemod-mcp-playbook` | MCP preview/apply workflow |
