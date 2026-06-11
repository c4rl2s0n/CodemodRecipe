# codemod_recipe

[![Pub Version](https://img.shields.io/pub/v/codemod_recipe)](https://pub.dev/packages/codemod_recipe)
[![Dart CI](https://github.com/c4rl2s0n/CodemodRecipe/actions/workflows/dart.yml/badge.svg)](https://github.com/c4rl2s0n/CodemodRecipe/actions)
[![License: BSD-3-Clause](https://img.shields.io/badge/License-BSD%203--Clause-blue.svg)](LICENSE)

Reusable primitives for deterministic, recipe-based Dart codemods and file
scaffolds.

This package intentionally contains no project-specific transforms, paths, or
naming conventions. Projects build their own integration layer on top of the
generic recipe, runner, template, operation, patch, and Dart analyzer editor
APIs.

## Installation

Add `codemod_recipe` to your `pubspec.yaml`:

```yaml
dependencies:
  codemod_recipe: ^0.1.0
```

Or install via command line:

```bash
dart pub add codemod_recipe
```

## Core Concepts

```text
CLI args -> CodemodContext -> CodemodRecipe -> CodemodOperation -> FileChange -> preview/apply -> PostExecution
```

- `CodemodRecipe` declares arguments, ordered operations, and post actions.
- `CodemodRunner` handles CLI parsing, dry-run/apply, and errors.
- `CodemodContext` stores raw argument values and generic case helpers.
- `CodemodTemplate` renders inline or file-backed templates.
- `CodemodOperation` plans file edits or file creation.
- `CodeTransform` converts source text into deterministic `SourcePatch`es.
- `CodeEditor` provides AST-guided Dart insertion helpers.
- `PostExecution` runs reusable commands after successful apply.

## Example

```dart
import 'package:codemod_recipe/codemod_recipe.dart';

Future<void> main(List<String> args) {
  return CodemodRunner(addMethodRecipe).run(args);
}

final addMethodRecipe = CodemodRecipe(
  name: 'add_method',
  args: [
    CodemodArg<String>.required('file'),
    CodemodArg<String>.required('class'),
    CodemodArg<String>.required('method'),
  ],
  operations: [
    EditDartFileOperation(
      path: (context) => context.require('file'),
      transforms: (context) => [
        AddMethodTransform(
          className: (context) => context.require('class'),
          methodName: (context) => context.camel('method'),
          body: const CodemodTemplate.inline('''
  void {{method:camel}}() {}
'''),
        ),
      ],
    ),
  ],
  postExecution: const [DartFormatPostExecution()],
);
```

## Templates

Templates use Mustache-style placeholders with package-defined casing filters:

```mustache
lib/features/{{feature:snake}}/{{feature:snake}}_view.dart
class {{feature:pascal}}View {}
final {{feature:camel}}Controller = ...
```

Supported casing filters are `snake`, `camel`, and `pascal`. Missing variables
and unsupported placeholders fail the run instead of rendering silently.

## File Creation

Use `CreateFileOperation` to scaffold files:

```dart
CreateFileOperation(
  path: (context) => context.render(
    'lib/features/{{feature:snake}}/{{feature:snake}}_view.dart',
  ),
  template: const CodemodTemplate.inline('''
class {{feature:pascal}}View {}
'''),
)
```

The default behavior fails when a file already exists. Use
`FileExistsStrategy.skip` or `FileExistsStrategy.overwrite` only when a recipe
explicitly wants that behavior.

## Post Execution

Post actions run only after `--apply` succeeds. Built-ins include:

- `DartFormatPostExecution`
- `ProcessPostExecution`
- `BuildRunnerPostExecution`

## Project Integration

Keep project conventions outside this package:

```dart
extension ProjectCodemodContext on CodemodContext {
  String get featureName => require('feature');
  String get featureFile => 'lib/features/${snake('feature')}.dart';
}
```

Then project-specific recipes can import this package plus their local extension
file.

## API Reference

See the [API documentation](https://pub.dev/documentation/codemod_recipe/latest/) for detailed information about all public classes and functions.

Key components:

- **CodemodArg\<T\>**: Typed recipe arguments (`String`, `bool`, `int`, `double`, `Enum`). Use `CodemodArg<String>.required(...)`, `CodemodArg<bool>.optional(defaultsTo: false)`, and `CodemodArg<String>.fixed(...)` for workspace-pinned hidden args. `inputKind` is inferred for non-`String` types (`bool` → checkbox in the VS Code UI).
- **CodemodRecipe**: Define recipes with arguments, operations, and post-execution actions. Use `CodemodRecipe.compose(steps: ...)` to mix recipes, operations, and post-execution actions in one ordered list.
- **CodemodStep**: Marker interface implemented by recipes, operations, and post-execution actions for composition.
- **CodemodRunner**: Execute recipes with CLI parsing and dry-run/apply support.
- **CodemodContext**: Access arguments and convert between naming conventions.
- **CodemodTemplate**: Render file and code templates with variable substitution.
- **CodeEditor**: AST-guided code modifications using the Dart analyzer.
- **Transforms**: Pre-built transforms for common operations like adding imports, methods, fields, and annotations.

## Examples

The `example/` directory contains runnable examples demonstrating:

- `add_method_example`: Adding methods to existing classes
- `scaffold_feature_example`: Creating new feature files from templates
- `composed_recipe_example`: Combining multiple recipes with shared arguments

Run an example:

```bash
cd example/add_method_example
dart pub get
dart run bin/add_method.dart --help
```

## VS Code Extension

A GUI front-end lives in [`vscode_extension/`](vscode_extension/README.md). It
lets you browse recipes, fill placeholder values in a form, preview changes as a
native diff, and selectively apply individual patches — no command line needed.

The extension talks to a small Dart host that registers your recipes:

```dart
import 'package:codemod_recipe/codemod_recipe_vscode.dart';

Future<void> main(List<String> args) {
  return CodemodHost.fromList([addMethodRecipe]).run(args);
}
```

Recipes may also add UI metadata such as `inputKind`, `options`, and
`contextKey`. The extension uses that metadata for file pickers, editable
dropdown suggestions, and cursor-context shortcuts. File scaffold previews are
derived from `CreateFileOperation.templatePath`, so previews reuse the same
template that will be applied.

See [`example/vscode_host_example`](example/vscode_host_example) for a runnable
host, and the [extension README](vscode_extension/README.md) for setup.

## Testing

Run package tests:

```bash
dart test
```

Tests should cover generic template rendering, operation behavior, patch
behavior, recipe composition, and AST-guided editor primitives.
Project-specific transforms should be tested in the consuming project.

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for
guidelines on development setup, running tests, and submitting pull requests.

## License

This project is licensed under the BSD-3-Clause License - see the
[LICENSE](LICENSE) file for details.
