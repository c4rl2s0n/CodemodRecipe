# AST Localization Syntax Reference

This document describes all supported AST (Abstract Syntax Tree) localization syntax for the Codemod Recipe DSL. The syntax allows you to precisely target locations within Dart source code for code modifications.

## Quick Reference

### Navigation Steps (v2)
`class:Name` | `method:Name` | `ctor:Name` | `field:Name` | `function:Name` | `var:Name` | `variable:Name` | `import:uri` | `call:TypeName` | `initializer` | `redirection` | `.` / `root`

### Anchors (v2)
`body:start` | `body:end` | `stmt:last` | `stmt:$` | `member:last` | `param:last` | `arg:last` | `meta:before` | `doc:before` | `doc:after` | `initializer:replace` | `initializer:last` | `param:name:NAME` | `arg:name:NAME` | `param:N` | `arg:N` | `initializer:name:NAME` | `redirection:arg:last` | `redirection:arg:name:NAME`

---

## Table of Contents

- [Overview](#overview)
- [Path Structure](#path-structure)
- [Navigation Steps](#navigation-steps)
  - [Typed Navigation](#typed-navigation)
  - [Type-Inferred Navigation](#type-inferred-navigation)
  - [Match Filters](#match-filters)
- [Anchors](#anchors)
  - [Position Anchors](#position-anchors)
  - [Named Anchors](#named-anchors)
  - [Indexed Anchors](#indexed-anchors)
  - [Metadata Anchors](#metadata-anchors)
  - [Initializer Anchors](#initializer-anchors)
  - [Redirection Anchors](#redirection-anchors)
- [String Format](#string-format)
- [Structured Format](#structured-format)
- [Examples](#examples)

---

## Overview

AST localization uses a path-based syntax to navigate from the compilation unit root to a specific node, then specify an anchor point for insertion or modification. The system supports two equivalent formats:

1. **String format**: `class:Settings > method:update @ stmt:last`
2. **Structured (YAML) format**:
   ```yaml
   at:
     - class: Settings
     - method: update
   anchor: stmt:last
   ```

---

## Path Structure

A complete AST path consists of:

```
[navigation steps] @ [anchor]
```

- **Navigation steps**: Zero or more steps separated by `>` that navigate to a target node
- **Anchor**: A single anchor token that specifies the insertion point relative to the target node

---

## Navigation Steps

### Typed Navigation

Typed navigation explicitly specifies the kind of AST node to find. Use the format `kind:name` where `kind` is one of the following:

| Kind | Shorthand | Description | Example |
|------|-----------|-------------|---------|
| `class` | - | Class declaration by name | `class:Settings` |
| `classDecl` | - | Alias for `class` | `classDecl:Settings` |
| `method` | - | Method declaration by name (within current class) | `method:update` |
| `ctor` | - | Constructor declaration | `ctor:` (unnamed) or `ctor:MyClass` |
| `field` | - | Field declaration by variable name | `field:count` |
| `function` | - | Top-level function declaration | `function:main` |
| `var` | `variable` | Top-level variable declaration | `var:config` or `variable:config` |
| `import` | - | Import directive by URI | `import:package:flutter/material.dart` |
| `call` | - | Constructor/instance creation by type name | `call:MaterialApp` |
| `initializer` | - | Constructor's initializer list | `initializer` |
| `redirection` | - | Constructor's redirection target | `redirection` |
| `root` | `.` | Compilation unit root | `root` or `.` |

**Notes:**
- For `ctor:`, if no name is provided, it matches the unnamed constructor
- For `import:`, the name must be the full URI string
- For `call:`, the name must be the type name being instantiated

### Type-Inferred Navigation

When the node kind is omitted, the system uses type inference to find the best matching node by name. The search priority is:

1. Class declarations (at compilation unit level)
2. Constructors (within current class)
3. Methods (within current class)
4. Fields (within current class)
5. Top-level functions
6. Top-level variables

**Example:**
```yaml
at:
  - Settings  # Will find class:Settings
  - update    # Will find method:update in Settings class
anchor: stmt:last
```

This is equivalent to:
```yaml
at:
  - class: Settings
  - method: update
anchor: stmt:last
```

### Match Filters

When multiple nodes match the same name, use the `match` property to disambiguate by source code content:

```yaml
at:
  - class: DerivedSettings
    match: extends BaseSettings
anchor: member:last
```

The `match` value is a substring that must appear in the node's source code snippet.

---

## Anchors

Anchors specify the exact insertion point relative to the focused node. All anchors are prefixed with their category (e.g., `stmt:`, `param:`, `arg:`).

### Position Anchors

Position anchors target specific positions within a node:

| Anchor | Description | Valid For | Example |
|--------|-------------|-----------|---------|
| `body:start` | Beginning of body (after `{`) | Class, Function | `body:start` |
| `body:end` | End of body (before `}`) | Class, Function | `body:end` |
| `stmt:last` | After the last statement in a block | Method, Function | `stmt:last` |
| `stmt:$` | Alias for `stmt:last` | Method, Function | `stmt:$` |
| `member:last` | After the last member in a class | Class | `member:last` |
| `param:last` | After the last parameter | Constructor, Function | `param:last` |
| `arg:last` | After the last argument in a call | Constructor call | `arg:last` |

### Named Anchors

Named anchors target specific named elements:

| Anchor Format | Description | Valid For | Example |
|---------------|-------------|-----------|---------|
| `param:name:NAME` | At the end of a named parameter | Constructor, Function | `param:name:key` |
| `arg:name:NAME` | At the end of a named argument | Constructor call | `arg:name:home` |
| `initializer:name:NAME` | At a specific field initializer | Constructor | `initializer:name:count` |
| `redirection:arg:name:NAME` | At a named argument in redirection | Constructor | `redirection:arg:name:key` |

### Indexed Anchors

Indexed anchors target elements by their zero-based position:

| Anchor Format | Description | Valid For | Example |
|---------------|-------------|-----------|---------|
| `param:N` | At the end of the Nth parameter (0-based) | Constructor, Function | `param:0` |
| `arg:N` | At the end of the Nth argument (0-based) | Constructor call | `arg:1` |

### Metadata Anchors

Metadata anchors target documentation and annotations:

| Anchor | Description | Valid For | Example |
|--------|-------------|-----------|---------|
| `meta:before` | Before the first metadata/annotation | Class, Method, Constructor, Field, Function | `meta:before` |
| `doc:before` | Before the documentation comment | Class, Method, Constructor, Field, Function | `doc:before` |
| `doc:after` | After the documentation comment (before declaration) | Class, Method, Constructor, Field, Function | `doc:after` |

### Initializer Anchors

Initializer anchors target constructor initializer lists:

| Anchor | Description | Valid For | Example |
|--------|-------------|-----------|---------|
| `initializer:replace` | Replaces the entire initializer expression of a field | Field | `initializer:replace` |
| `initializer:last` | After the last initializer in a constructor's initializer list | Constructor | `initializer:last` |
| `initializer:name:NAME` | At a specific field's initializer in the list | Constructor | `initializer:name:count` |

**Note:** `initializer:replace` has a non-zero length and replaces existing content, while other anchors are pure insertion points (length = 0).

### Redirection Anchors

Redirection anchors target constructor redirection:

| Anchor | Description | Valid For | Example |
|--------|-------------|-----------|---------|
| `redirection:arg:last` | After the last argument in a redirection | Constructor | `redirection:arg:last` |
| `redirection:arg:name:NAME` | At a named argument in a redirection | Constructor | `redirection:arg:name:key` |

---

## String Format

The string format uses a compact syntax for inline path specifications:

```
[step1] > [step2] > ... > [stepN] @ [anchor]
```

**Examples:**
```dart
// Navigate to method in class
'class:Settings > method:update @ stmt:last'

// Navigate to field in class
'class:Widget > field:count @ initializer:replace'

// Type-inferred navigation
'Settings > update @ stmt:last'

// With unnamed constructor
'class:Widget > ctor: @ param:last'

// Using stmt:$ alias
'class:Settings > method:update @ stmt:$'
```

**Rules:**
- Steps are separated by `>` (with optional whitespace)
- Navigation and anchor are separated by `@` (with optional whitespace)
- Colons in step values must not contain colons (use structured format for complex values)

---

## Structured Format

The structured format uses YAML maps and lists for better readability and to support additional properties like `match`:

### Simple Navigation

```yaml
at:
  - class: Settings
  - method: update
anchor: stmt:last
```

### With Match Filter

```yaml
at:
  - class: DerivedSettings
    match: extends BaseSettings
anchor: member:last
```

### Mixed Typed and Inferred

```yaml
at:
  - Settings
  - method: update
anchor: stmt:last
```

### Named Anchors

```yaml
at:
  - class: Widget
  - ctor: null
anchor: param:name:key
```

### Using `navigate` instead of `at`

The `navigate` key is an alias for `at`:

```yaml
navigate:
  - class: Settings
  - method: update
anchor: stmt:last
```

---

## Examples

### Common Use Cases

#### 1. Add a method to a class

```yaml
steps:
  - edit:
      path: "{{file}}"
      steps:
        - insert:
            at:
              - class: "{{className}}"
            anchor: member:last
            text: |
              void newMethod() {}
```

#### 2. Add a statement to a method

```yaml
steps:
  - edit:
      path: "{{file}}"
      steps:
        - insert:
            at:
              - class: "{{className}}"
              - method: "{{methodName}}"
            anchor: stmt:last
            text: |
              print('added by codemod');
```

#### 3. Add a parameter to a constructor

```yaml
steps:
  - edit:
      path: "{{file}}"
      steps:
        - insert:
            at:
              - class: "{{className}}"
              - ctor: null
            anchor: param:last
            text: ", String newParam"
```

#### 4. Add an argument to a constructor call

```yaml
steps:
  - edit:
      path: "{{file}}"
      steps:
        - insert:
            at:
              - call: MaterialApp
            anchor: arg:last
            text: ", title: 'My App'"
```

#### 5. Add documentation to a class

```yaml
steps:
  - edit:
      path: "{{file}}"
      steps:
        - insert:
            at:
              - class: "{{className}}"
            anchor: doc:before
            text: |
              /// This class was generated by codemod.
```

#### 6. Replace a field initializer

```yaml
steps:
  - edit:
      path: "{{file}}"
      steps:
        - insert:
            at:
              - class: "{{className}}"
              - field: count
            anchor: initializer:replace
            text: "42"
```

#### 7. Add annotation to a class

```yaml
steps:
  - edit:
      path: "{{file}}"
      steps:
        - insert:
            at:
              - class: "{{className}}"
            anchor: meta:before
            text: "@deprecated"
```

#### 8. Using match filter for disambiguation

```yaml
steps:
  - edit:
      path: "{{file}}"
      steps:
        - insert:
            at:
              - class: BaseWidget
                match: extends StatelessWidget
            anchor: member:last
            text: |
              Widget build(BuildContext context) {}
```

#### 9. Insert at end of a function

```yaml
steps:
  - edit:
      path: "{{file}}"
      steps:
        - insert:
            at: function:main @ stmt:last
            text: |
              final int value;
```

**Note:** Use `function:` for top-level functions and `method:` for class methods. The `stmt:last` anchor now works for both functions and methods.

---

## Error Codes

When path resolution fails, the following error codes may be returned:

| Code | Description |
|------|-------------|
| `E_NODE_NOT_FOUND` | The specified node (class, method, field, etc.) was not found |
| `E_ANCHOR_INVALID` | The specified anchor is not valid for the focused node type |
| `E_NAVIGATION_INVALID` | The navigation step is invalid for the current context |

---

## Best Practices

1. **Use typed navigation** when the node type is known for clarity and to avoid ambiguity
2. **Use type-inferred navigation** for brevity when the context makes the type obvious
3. **Add match filters** when working with multiple classes/methods with the same name
4. **Prefer structured format** for complex paths or when using match filters
5. **Use string format** for simple, inline path specifications
6. **Test paths** against sample code before using them in production recipes

---

## Version History

- **v1**: Initial AST path syntax with basic navigation and anchors
- **v2**: Added named anchors (`param:name:X`, `arg:name:X`), indexed anchors, and improved error handling
