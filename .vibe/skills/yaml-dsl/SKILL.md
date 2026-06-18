---
name: yaml-dsl
description: Complete reference for the YAML AST DSL (Domain Specific Language) used in codemod_recipe. This skill provides the syntax dictionary, grammar, examples, and validation rules for the YAML-based recipe definition language. Must be updated when the DSL in lib/src/yaml/dsl.dart changes.
user-invocable: false
allowed-tools: ["read", "grep"]
---

# YAML AST DSL Reference

> **IMPORTANT**: This skill documents the YAML AST DSL defined in `lib/src/yaml/dsl.dart`. Whenever the DSL code changes, YOU MUST UPDATE THIS SKILL to keep it in sync. This ensures all agents (including specialized subagents) have accurate information about the recipe syntax.

## Overview

The **YAML AST DSL** (Domain Specific Language) is a declarative language for defining code modifications (codemods) in the codemod_recipe framework. It allows developers to specify **what** code changes to make, while the framework handles **how** to make them.

**Key Features**:
- Declarative syntax using YAML
- AST-based code navigation
- Template support for dynamic values
- Composition and reuse
- Validation and error handling

**DSL Version**: 1 (current and only supported version)

---

## Table of Contents

1. [Recipe Structure](#recipe-structure)
2. [Top-Level Fields](#top-level-fields)
3. [Arguments (args)](#arguments-args)
4. [Steps](#steps)
5. [Edit Operations](#edit-operations)
6. [Create Operations](#create-operations)
7. [Recipe Composition](#recipe-composition)
8. [Maps](#maps)
9. [Post-Execution](#post-execution)
10. [AST Paths](#ast-paths)
11. [Anchors](#anchors)
12. [Templates](#templates)
13. [Validation Rules](#validation-rules)
14. [Complete Examples](#complete-examples)
15. [Error Messages](#error-messages)
16. [Maintenance](#maintenance)

---

## Recipe Structure

```yaml
dslVersion: 1                    # Required: DSL version
id: recipe_identifier            # Optional: Unique identifier
name: "Human Readable Name"      # Optional: Display name
description: "What this recipe does"  # Optional: Description

args:                          # Optional: Recipe arguments
  - name: arg1
    required: true
    inputKind: file

steps:                         # Required: List of steps
  - edit:
      path: "{{file}}"
      steps:
        - insert:
            at:
              - class: MyClass
            anchor: member:last
            text: "void newMethod() {}"

maps:                          # Optional: Reusable mappings
  constants:
    entries:
      MAX_SIZE: "100"

postExecution:                 # Optional: Post-execution hooks
  - run: dart format .
```

---

## Top-Level Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `dslVersion` | integer | ✅ Yes | DSL version number (currently 1) |
| `id` | string | ⚠️ Conditional | Unique identifier. Required if `name` not provided |
| `name` | string | ⚠️ Conditional | Human-readable name. Required if `id` not provided |
| `description` | string | ❌ No | Description of what the recipe does |
| `args` | list of maps | ❌ No | Recipe arguments/parameters |
| `steps` | list | ✅ Yes | List of transformation steps |
| `maps` | map | ❌ No | Reusable key-value mappings |
| `postExecution` | list | ❌ No | Commands to run after execution |

**Validation Rule**: At least one of `id` or `name` must be provided.

---

## Arguments (args)

Arguments define the inputs that a recipe accepts. They can be used in templates throughout the recipe.

### Argument Fields

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `name` | string | ✅ Yes | - | Argument name (used in templates as `{{name}}`) |
| `required` | boolean | ❌ No | `false` | Whether the argument is mandatory |
| `help` | string | ❌ No | - | Help text displayed to users |
| `abbr` | string | ❌ No | - | Short abbreviation (e.g., "f" for "file") |
| `contextKey` | string | ❌ No | - | Context key for auto-filling from editor |
| `inputKind` | string | ❌ No | `text` | Type of input expected |
| `options` | list of strings | ❌ No | - | Available options for dropdown selection |
| `allowCustomValue` | boolean | ❌ No | `false` | Allow values not in `options` list |
| `defaultsTo` | string | ❌ No | - | Default value if not provided |

### Input Kinds

| Kind | Description | Example |
|------|-------------|---------|
| `text` | Free-form text | `"MyClass"` |
| `file` | File path | `"./lib/my_file.dart"` |
| `directory` | Directory path | `"./lib/src"` |
| `symbol` | Class/method/function name | `"MyClass"` |

### Example Arguments

```yaml
args:
  # Simple required file argument
  - name: file
    required: true
    inputKind: file
    help: "The file to modify"

  # Optional text argument with default
  - name: className
    required: false
    inputKind: symbol
    defaultsTo: "MyClass"
    help: "The class name"

  # Dropdown selection
  - name: visibility
    inputKind: text
    options: [public, private, protected]
    defaultsTo: "public"
    help: "Visibility modifier"

  # With auto-fill from editor
  - name: currentFile
    inputKind: file
    contextKey: "currentFilePath"
    required: true
```

---

## Steps

Steps define the transformations to perform. Each step must have exactly one operation type.

### Step Types

| Type | Description | Use Case |
|------|-------------|----------|
| `recipe` | Reference another recipe | Composition, reuse |
| `edit` | Modify existing files | Code changes |
| `create` | Create new files | File generation |

### Validation Rule
Each step must have exactly one of: `recipe`, `edit`, or `create`. Having none or more than one is an error.

---

## Edit Operations

Edit operations modify existing files. They require a `path` and a list of sub-steps.

### Edit Step Structure

```yaml
- edit:
    path: "{{file}}"              # Required: File path (supports templates)
    steps:                       # Required: List of edit sub-steps
      - insert:
          at: [class: MyClass]     # Required: AST path
          anchor: member:last      # Required: Insertion anchor
          text: "void method() {}"  # Required: Text to insert
```

### Edit Sub-Step Types

| Type | Description | Fields |
|------|-------------|--------|
| `insert` | Insert text at a location | `at`, `anchor`, `text` |

**Note**: Currently, `insert` is the primary edit sub-step type. Future versions may add more (delete, replace, etc.).

### Insert Operation

**Fields**:
- `at` (required): AST path specifying WHERE to insert
- `anchor` (required): Anchors specifying RELATIVE TO WHAT
- `text` (required): The text/code to insert

**Validation Rules**:
- All three fields are required
- `at` must be a valid AST path
- `anchor` must be a valid anchor type
- `text` must be a non-empty string

---

## Create Operations

Create operations generate new files from templates.

### Create Step Structure

```yaml
- create:
    path: "lib/new_file.dart"    # Required: File path to create
    template: "file content"      # Optional: Inline template content
    templateFile: "template.dart" # Optional: Path to template file
    ifExists: skip                # Optional: Behavior if file exists
    format: true                 # Optional: Auto-format the file
```

### Create Fields

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `path` | string | ✅ Yes | - | File path to create (supports templates) |
| `template` | string | ⚠️ Conditional | - | Inline template content |
| `templateFile` | string | ⚠️ Conditional | - | Path to external template file |
| `ifExists` | string | ❌ No | `error` | What to do if file exists |
| `format` | boolean | ❌ No | `false` | Whether to auto-format the created file |

**Validation Rule**: Exactly one of `template` or `templateFile` must be provided.

### ifExists Options

| Value | Behavior |
|-------|----------|
| `skip` | Skip creation if file exists (no error) |
| `overwrite` | Overwrite existing file |
| `error` | Raise an error if file exists (default) |

---

## Recipe Composition

Recipe composition allows reusing existing recipes as steps.

### Recipe Reference Structure

```yaml
- recipe:
    recipe: "other_recipe_id"  # Required: ID of recipe to execute
```

### Example: Composed Recipe

```yaml
# main_recipe.yaml
dslVersion: 1
id: main_recipe
steps:
  - recipe:
      recipe: "setup_recipe"
  - edit:
      path: "{{file}}"
      steps:
        - insert:
            at: [class: MyClass]
            anchor: member:last
            text: "// Main logic"
  - recipe:
      recipe: "cleanup_recipe"
```

---

## Maps

Maps define reusable key-value pairs that can be referenced throughout the recipe.

### Map Structure

```yaml
maps:
  myMap:
    entries:
      KEY1: "value1"
      KEY2: "value2"
  anotherMap:
    entries:
      SETTING: "enabled"
```

### Map Validation

- Map keys are strings
- Map values are strings
- Maps are always valid (no validation errors)
- Maps are referenced by name in templates

---

## Post-Execution

Post-execution hooks run after the main recipe steps complete. They can run commands, scripts, or built-in actions.

### Post-Execution Structure

```yaml
postExecution:
  - run: "dart format ."
  - runScript: "scripts/cleanup.dart"
  - command: "format"
```

### Post-Execution Types

| Type | Field | Description |
|------|-------|-------------|
| Shell command | `run` | Run a shell command |
| Dart script | `runScript` | Execute a Dart script file |
| Built-in command | `command` | Run a built-in command |

**Validation Rule**: Each post-execution entry must have exactly one of: `run`, `runScript`, or `command`.

---

## AST Paths

**AST Paths** (Abstract Syntax Tree Paths) are used to navigate and identify specific code elements. They are the primary way to specify **WHERE** in the code to make changes.

### AST Path Format

AST paths are specified as a **list** of path components:

```yaml
at:
  - class: MyClass
  - method: myMethod
  - arg: first
```

Or as a **map** (alternative syntax):

```yaml
at:
  class: MyClass
  method: myMethod
  arg: first
```

**Note**: Both formats are equivalent and can be used interchangeably.

### AST Path Components

| Component Type | Description | Example |
|---------------|-------------|---------|
| `class` | Class name | `class: MyClass` |
| `method` | Method/function name | `method: myMethod` |
| `function` | Function name (alias for method) | `function: myFunction` |
| `arg` | Argument/parameter position | `arg: first`, `arg: 0`, `arg: last` |
| `param` | Parameter name | `param: myParam` |
| `field` | Field/property name | `field: myField` |
| `call` | Function/method call | `call: WidgetName` |
| `type` | Type annotation | `type: int` |
| Any custom | Can match any AST node type | `myNode: value` |

### AST Path Examples

```yaml
# Navigate to a specific method in a class
at:
  - class: User
  - method: login

# Navigate to first argument of a function
at:
  - function: processData
  - arg: first

# Navigate to a specific call
at:
  - call: MaterialApp

# Navigate to a field in a class
at:
  - class: Settings
  - field: timeout

# Complex path
at:
  - class: MyClass
  - method: myMethod
  - arg: 0
  - type: String
```

---

## Anchors

**Anchors** define the **insertion point** relative to the AST path location. They specify WHERE to insert new code relative to the target element.

### Anchor Types

#### Documentation Anchors
| Anchor | Description | Inserts |
|--------|-------------|---------|
| `doc:before` | Before the documentation comment | Above the doc comment |
| `doc:after` | After the documentation comment | Below the doc comment |

#### Member Anchors
| Anchor | Description | Inserts |
|--------|-------------|---------|
| `member:first` | Before the first member | At the top of the class |
| `member:last` | After the last member | At the bottom of the class |
| `member:before:FieldName` | Before a specific member | Above the specified field/method |
| `member:after:FieldName` | After a specific member | Below the specified field/method |

#### Argument/Parameter Anchors
| Anchor | Description | Inserts |
|--------|-------------|---------|
| `arg:first` | Before the first argument | At the start of args list |
| `arg:last` | After the last argument | At the end of args list |
| `arg:N` | At position N (0-indexed) | Between existing args |
| `arg:before:paramName` | Before a specific parameter | Before named param |
| `arg:after:paramName` | After a specific parameter | After named param |

#### Function Call Anchors
| Anchor | Description | Inserts |
|--------|-------------|---------|
| `call:FunctionName` | At a call to FunctionName | Inside the call's arguments |
| `call:before` | Before the function call | Before the entire call |
| `call:after` | After the function call | After the entire call |

#### Parameter Anchors
| Anchor | Description | Inserts |
|--------|-------------|---------|
| `param:first` | Before the first parameter | Start of param list |
| `param:last` | After the last parameter | End of param list |
| `param:N` | At position N | Between params |

#### General Position Anchors
| Anchor | Description | Inserts |
|--------|-------------|---------|
| `before` | Before the target element | Prior to the element |
| `after` | After the target element | Following the element |
| `replace` | Replace the target element | Replaces the element |

### Anchor Examples

```yaml
# Insert at the end of a class
- insert:
    at: [class: MyClass]
    anchor: member:last
    text: "void newMethod() {}"

# Insert before first argument
- insert:
    at: [function: myFunction]
    anchor: arg:first
    text: "int newParam"

# Insert at the end of function arguments
- insert:
    at: [call: WidgetName]
    anchor: arg:last
    text: ", key: ValueKey('myKey')"

# Insert before a specific member
- insert:
    at: [class: MyClass]
    anchor: member:before:existingMethod
    text: "// New method"
```

---

## Templates

Templates allow dynamic content generation using variables and expressions.

### Template Syntax

#### Variable Substitution

```
{{variableName}}
```

Variables are replaced with their values from arguments or maps.

#### With Default Values

```
{{variableName | defaultValue}}
```

If `variableName` is not defined, `defaultValue` is used.

#### Variable Modifiers

| Modifier | Example | Result |
|----------|---------|--------|
| `upper` | `{{name | upper}}` | Converts to uppercase |
| `lower` | `{{name | lower}}` | Converts to lowercase |
| `capitalize` | `{{name | capitalize}}` | Capitalizes first letter |
| `camel` | `{{name | camel}}` | Converts to camelCase |
| `snake` | `{{name | snake}}` | Converts to snake_case |

**Note**: Check the actual implementation in `template.dart` for supported modifiers. The above are common patterns.

### Template Examples

```yaml
# Using argument in path
- edit:
    path: "{{file}}"
    steps:
      - insert:
          at: [class: "{{className}}"]
          anchor: member:last
          text: |
            void {{methodName}}() {
              // Implementation
            }

# Using map values
- create:
    path: "lib/{{maps.constants.entries.NAME}}.dart"
    template: |
      class {{Name | camel}} {
        static const int MAX = {{maps.constants.entries.MAX_SIZE}};
      }

# With default values
- edit:
    path: "{{file | lib/my_file.dart}}"
    steps:
      - insert:
          at: [class: "{{className | MyClass}}"]
          anchor: member:last
          text: "void method() {}"
```

---

## Validation Rules

The DSL performs validation at multiple levels:

### Recipe-Level Validation

1. **Required Fields**: `dslVersion` and `steps` are required
2. **Identifier**: At least one of `id` or `name` must be present
3. **DSL Version**: Currently only version 1 is supported

### Arguments Validation

1. **Name Required**: Each argument must have a `name`
2. **Name Not Empty**: Argument name cannot be empty string

### Steps Validation

1. **At Least One Operation**: Each step must have exactly one of: `recipe`, `edit`, or `create`
2. **Edit Steps**: Edit steps require `path` and `steps`
3. **Create Steps**: Create steps require `path` and exactly one of `template` or `templateFile`
4. **Recipe Steps**: Recipe steps require `recipe` field

### Edit Sub-Steps Validation

1. **Insert Validation**:
   - `at` is required (AST path)
   - `anchor` is required (insertion point)
   - `text` is required and non-empty (content to insert)

### Create Validation

1. **Path Required**: `path` is required
2. **Template Source**: Exactly one of `template` or `templateFile` must be provided
3. **Inline Template**: If using `template`, it must be non-empty
4. **File Template**: If using `templateFile`, the file must exist (validated at runtime)

### Post-Execution Validation

1. **At Least One Operation**: Each post-execution entry must have exactly one of: `run`, `runScript`, or `command`

---

## Complete Examples

### Example 1: Simple Method Addition

```yaml
dslVersion: 1
id: add_method
name: Add Method
description: Adds a new method to a class

args:
  - name: file
    required: true
    inputKind: file
    help: The file containing the class
  - name: className
    required: true
    inputKind: symbol
    help: The class name
  - name: methodName
    required: true
    inputKind: text
    help: The method name
  - name: methodBody
    required: false
    inputKind: text
    defaultsTo: "// TODO: implement"
    help: The method body

steps:
  - edit:
      path: "{{file}}"
      steps:
        - insert:
            at:
              - class: "{{className}}"
            anchor: member:last
            text: |
              void {{methodName}}() {
                {{methodBody}}
              }

postExecution:
  - run: dart format "{{file}}"
```

### Example 2: Field Addition with Template

```yaml
dslVersion: 1
id: add_field
name: Add Field

args:
  - name: file
    required: true
    inputKind: file
  - name: className
    required: true
    inputKind: symbol
  - name: fieldName
    required: true
    inputKind: symbol
  - name: fieldType
    required: true
    inputKind: symbol
    defaultsTo: "int"

steps:
  - edit:
      path: "{{file}}"
      steps:
        - insert:
            at: [class: "{{className}}"]
            anchor: member:last
            text: "final {{fieldType}} {{fieldName}};"
```

### Example 3: Composed Recipe

```yaml
dslVersion: 1
id: full_setup
name: Full Setup

steps:
  # Run setup recipe first
  - recipe:
      recipe: "initial_setup"
  
  # Add logging
  - edit:
      path: "{{file}}"
      steps:
        - insert:
            at: [class: App]
            anchor: member:first
            text: |
              static final Logger _logger = Logger('App');
  
  # Run cleanup
  - recipe:
      recipe: "cleanup_recipe"

postExecution:
  - run: dart analyze
  - run: dart format .
```

### Example 4: File Creation with Template

```yaml
dslVersion: 1
id: create_service
name: Create Service Class

args:
  - name: name
    required: true
    inputKind: symbol
    help: Service name (e.g., UserService)
  - name: directory
    required: false
    inputKind: directory
    defaultsTo: "lib/services"

steps:
  - create:
      path: "{{directory}}/{{name}}.dart"
      template: |
        /// Service for {{name | lower}} operations.
        class {{name}} {
          {{name}}();
        }
      format: true
```

### Example 5: Modify Function Call

```yaml
dslVersion: 1
id: add_key_to_widget
name: Add Key to Widget

args:
  - name: file
    required: true
    inputKind: file
  - name: widgetName
    required: true
    inputKind: symbol
    defaultsTo: "MaterialApp"

steps:
  - edit:
      path: "{{file}}"
      steps:
        - insert:
            at: [call: "{{widgetName}}"]
            anchor: arg:last
            text: ", key: ValueKey('{{widgetName | lower}}_key')"
```

---

## Error Messages

When validation fails, the following error messages are generated:

### Recipe-Level Errors
| Error | Cause |
|-------|-------|
| `Recipe must have an id or name` | Neither `id` nor `name` provided |

### Arguments Errors
| Error | Cause |
|-------|-------|
| `Argument name is required` | Argument missing `name` field |

### Steps Errors
| Error | Cause |
|-------|-------|
| `Step must have one of: recipe, edit, or create` | Step has none of these fields |
| `Step can only have one operation type` | Step has more than one operation field |

### Edit Errors
| Error | Cause |
|-------|-------|
| `Edit operation requires a path` | Edit step missing `path` field |
| `Edit operation requires steps` | Edit step missing `steps` list |

### Insert Errors
| Error | Cause |
|-------|-------|
| `Insert operation requires "at" field` | Insert missing `at` (AST path) |
| `Insert operation requires "anchor" field` | Insert missing `anchor` |
| `Insert operation requires "text" field` | Insert missing `text` |

### Create Errors
| Error | Cause |
|-------|-------|
| `Create operation requires "path" field` | Create missing `path` |
| `Create operation requires "template" or "templateFile"` | Create missing both template fields |

### Post-Execution Errors
| Error | Cause |
|-------|-------|
| `Post-execution action must have a command, run, or runScript` | Post-execution entry has none |
| `Post-execution action can only have one operation type` | Post-execution entry has more than one |

---

## Maintenance

> **CRITICAL**: This skill must be kept in sync with the DSL implementation in `lib/src/yaml/dsl.dart`.

### When to Update This Skill

Update this skill **immediately** when:

1. **New DSL features are added** to `dsl.dart`
   - New node types
   - New fields on existing nodes
   - New validation rules
   - New anchor types
   - New template modifiers

2. **DSL behavior changes**
   - Field requirements change
   - Validation logic changes
   - Default values change
   - Error messages change

3. **New examples would be helpful**
   - New use cases discovered
   - Common patterns identified
   - Best practices established

### How to Update

1. **Review `lib/src/yaml/dsl.dart`**: Check for any changes to the DSL classes
2. **Update corresponding sections**: Modify this skill to match
3. **Add new examples**: Show how to use new features
4. **Update validation rules**: Reflect any changes to validation logic
5. **Test the skill**: Verify the documentation matches actual behavior

### Quick Sync Checklist

- [ ] Compare all class definitions in `dsl.dart` with skill sections
- [ ] Verify all fields match (name, type, required, default)
- [ ] Update validation rules to match `validate()` methods
- [ ] Add examples for new features
- [ ] Update error messages list
- [ ] Check for deprecated features

### Relationship with Other Files

This skill is related to:
- **`lib/src/yaml/dsl.dart`**: The primary source of truth for DSL structure
- **`lib/src/yaml/recipe_compiler.dart`**: Compiles recipes to executable form
- **`lib/src/yaml/schema_validator.dart`**: Validates recipe schemas
- **`.vibe.md`**: Project overview and quick reference
- **`test/fixtures/yaml_recipes/`**: Example recipes for testing

---

## Quick Reference

### DSL Version
```yaml
dslVersion: 1
```

### Minimal Valid Recipe
```yaml
dslVersion: 1
id: my_recipe
steps: []
```

### Common Anchor Patterns
```yaml
member:last      # End of class
arg:last        # End of argument list
member:before:X  # Before member X
arg:after:Y     # After argument Y
```

### Template Patterns
```yaml
{{variable}}          # Simple substitution
{{var | default}}    # With default
{{var | upper}}      # With modifier
```

---

**Last Updated**: This skill must be reviewed whenever `lib/src/yaml/dsl.dart` changes.
