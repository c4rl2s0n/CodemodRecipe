# Basic patterns

## Node types

Match a named node and optionally its children:

```scheme
(function_item)
(function_item (identifier) @name)
```

Omit children to match any node of that type with any children:

```scheme
(class_definition (identifier))   ; class with at least one identifier child somewhere
```

## Fields

Grammar fields make patterns precise. Prefix a child with `field_name:`:

```scheme
(class_definition
  name: (identifier) @className
  body: (class_body) @body)
```

## Negated fields

Require that a field is **absent**:

```scheme
(class_definition
  name: (identifier) @name
  !type_parameters)
```

## Anonymous nodes

Literal tokens (keywords, operators, punctuation) use **double quotes**:

```scheme
(binary_expression operator: "!=" right: (null))
"required"
"final"
```

These do **not** appear in `dump_ast` when `namedOnly: true` (the default in some UIs). Pass **`namedOnly: false`** (MCP `dump_ast`) to see keywords and punctuation before writing patterns that match them.

Anonymous siblings (e.g. a keyword next to a parameter node) are matched in the query with `"…"`, not by peeking source text around a span.

## Wildcards

| Pattern | Matches | Capturable? |
|---------|---------|-------------|
| `(_)` | any **named** node | Yes — `(_) @x` |
| `_` | any named **or anonymous** node | No — do not write `_ @x` |

```scheme
(call_expression function: (_) @callee)
```

If you need the text of an anonymous token, match it with a string literal (`"required"`) or capture a nearby named node and use `hit.text` / `c:spanText`.

## Special nodes

| Pattern | Meaning |
|---------|---------|
| `(ERROR)` | unparseable text |
| `(MISSING)` | recovered missing token |
| `(MISSING identifier)` | specific missing kind |
| `(expression)` | supertype — any expression subtype |
| `(expression/call_expression)` | call only when parent is supertype `expression` |

---
