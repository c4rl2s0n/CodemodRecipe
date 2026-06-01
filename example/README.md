# codemod_recipe Examples

This directory contains runnable examples demonstrating various features of the
`codemod_recipe` package.

## Examples

### add_method_example

Demonstrates adding a method to an existing Dart class using
`EditDartFileOperation` and `AddMethodTransform`.

```bash
cd add_method_example
dart pub get
dart run bin/add_method.dart --file=lib/counter.dart --class=Counter --method=decrement
dart run bin/add_method.dart --file=lib/counter.dart --class=Counter --method=decrement --apply
```

### scaffold_feature_example

Demonstrates scaffolding new feature files from templates using
`CreateFileOperation`.

```bash
cd scaffold_feature_example
dart pub get
dart run bin/scaffold_feature.dart --feature=user_profile
dart run bin/scaffold_feature.dart --feature=user_profile --apply
```

### composed_recipe_example

Demonstrates composing multiple recipes with shared arguments using
`CodemodRecipe.compose`.

```bash
cd composed_recipe_example
dart pub get
dart run bin/composed_codemod.dart --file=lib/models.dart --model=User --property=email
dart run bin/composed_codemod.dart --file=lib/models.dart --model=User --property=email --apply
```

## Running Examples

Each example follows the same pattern:

1. **Dry run** (default): Shows what changes would be made without modifying files
2. **Apply** (`--apply`): Actually applies the changes to files
3. **Help** (`--help`): Shows available options

All examples include `DartFormatPostExecution` to automatically format Dart
files after changes are applied.
