# Operators

## Quantifiers

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

## Grouping

Parentheses group sibling sequences:

```scheme
(
  (comment)
  (function_definition)
)
```

Quantifiers apply to groups: `((comment) (function_definition))+`

## Alternation

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

## Query anchor `.` (not codemod anchor)

The `.` operator constrains **position in the syntax tree** — first/last child or immediate siblings. This is **different** from codemod `anchor = "start"|"end"` on `c:insert`.

| Position | Example | Effect |
|----------|---------|--------|
| First child | `(array . (identifier) @first)` | `@first` only if first named child |
| Last child | `(block (_) @last .)` | `@last` only if last named child |
| Siblings | `(dotted_name (identifier) @a . (identifier) @b)` | consecutive siblings only |

**codemod `anchor = "start"|"end"`** chooses whether inserted `text` goes at the **byte start or end** of the captured node span. Only `c:insert` uses it.

---
