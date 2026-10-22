# Predicates

Predicates filter matches. They are S-expressions starting with `#` and ending with `?`, placed inside the pattern:

```scheme
(class_definition
  name: (identifier) @className
  (#eq? @className "Settings"))
```

The Rust engine uses standard `tree_sitter::Query` — predicates supported by the tree-sitter CLI work in recipes.

## Equality

| Predicate | Meaning |
|-----------|---------|
| `#eq?` | capture text equals string or another capture |
| `#not-eq?` | inverse |
| `#any-eq?` | for quantified captures: any node matches |
| `#any-not-eq?` | for quantified captures: any node does not match |

```scheme
(#eq? @className "Settings")
(#eq? @a @b)
```

Build the predicate literal with Luau (`.. args.className`) or MiniJinja (`c:template`) so the query parser sees a concrete string.

## Regular expressions

| Predicate | Meaning |
|-----------|---------|
| `#match?` | capture text matches regex |
| `#not-match?` | inverse |
| `#any-match?` / `#any-not-match?` | quantified variants |

```scheme
(identifier) @name (#match? @name "^_[a-z]+$")
```

## Lists and properties

| Predicate | Meaning |
|-----------|---------|
| `#any-of?` | capture equals one of several strings |
| `#is?` | node has a semantic property (grammar-specific) |
| `#is-not?` | node lacks a property |

```scheme
(identifier) @id (#any-of? @id "foo" "bar" "baz")
```

## Directives (rare in recipes)

Directives end with `!` (`#set!`, `#strip!`) and attach metadata. codemod recipes today use **predicates only** for filtering — directives are not required for insert/replace/remove.

---
