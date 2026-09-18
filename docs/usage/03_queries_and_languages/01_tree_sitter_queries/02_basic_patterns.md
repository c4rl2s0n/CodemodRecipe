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

Literal tokens use double quotes:

```scheme
(binary_expression operator: "!=" right: (null))
```

## Wildcards

| Pattern | Matches |
|---------|---------|
| `(_)` | any named node |
| `_` | any named or anonymous node |

```scheme
(call_expression function: (_) @callee)
```

## Special nodes

| Pattern | Meaning |
|---------|---------|
| `(ERROR)` | unparseable text |
| `(MISSING)` | recovered missing token |
| `(MISSING identifier)` | specific missing kind |
| `(expression)` | supertype — any expression subtype |
| `(expression/call_expression)` | call only when parent is supertype `expression` |

---
