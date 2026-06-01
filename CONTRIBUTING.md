# Contributing to codemod_recipe

Thank you for your interest in contributing to `codemod_recipe`! This document
provides guidelines and instructions for setting up your development
environment, running tests, and submitting contributions.

## Development Setup

### Prerequisites

- [Dart SDK](https://dart.dev/get-dart) version ^3.10.0 or higher
- [Git](https://git-scm.com/) for version control

### Clone the Repository

```bash
git clone https://github.com/yourusername/codemod_recipe.git
cd codemod_recipe
```

### Install Dependencies

```bash
dart pub get
```

## Development Workflow

### Running Tests

Run all tests:

```bash
dart test
```

Run tests with coverage (requires `coverage` package):

```bash
dart pub global activate coverage
dart test --coverage=coverage
dart pub global run coverage:format_coverage --lcov --in=coverage --out=coverage/lcov.info
```

### Code Analysis

Run static analysis:

```bash
dart analyze
```

### Formatting

Format all Dart files:

```bash
dart format .
```

Check formatting without making changes:

```bash
dart format --output=none --set-exit-if-changed .
```

### Generating Documentation

Generate API documentation:

```bash
dart doc
```

The generated documentation will be in `doc/api/`.

## Code Style Guidelines

This project follows the [Dart style guide](https://dart.dev/effective-dart/style)
and uses `package:lints/recommended.yaml` for static analysis.

Key points:

- Use `lowerCamelCase` for variables, constants, and function names
- Use `UpperCamelCase` for class and enum names
- Use `lowercase_with_underscores` for file names
- Use `PascalCase` for acronyms longer than two letters (e.g., `HttpResponse`)
- Write docstrings for all public APIs using `///`
- Prefer `const` constructors when possible

## Testing Guidelines

### Unit Tests

Place unit tests in the `test/` directory with the naming convention
`*_test.dart`.

### Test Structure

Use descriptive group and test names:

```dart
group('CodemodTemplate', () {
  test('renders simple placeholders', () {
    // test code
  });

  test('throws when variable is missing', () {
    // test code
  });
});
```

### What to Test

- **Template rendering**: Variable substitution, casing filters, missing variables
- **Recipe composition**: Arg merging, operation concatenation, post-execution ordering
- **Patch helpers**: Overlap detection, ordering, application
- **Operations**: File changes, error handling, edge cases
- **Transforms**: Idempotency (running twice produces no changes on second run)
- **CodeEditor**: AST navigation, insertion offsets, method/field detection

### Idempotency Testing

All transforms should be idempotent. Test this pattern:

```dart
test('is idempotent', () async {
  final patches = await transform.apply(source, context);
  final result = applyPatches(source, patches);
  
  // Second run should produce no changes
  final secondPatches = await transform.apply(result, context);
  expect(secondPatches, isEmpty);
});
```

## Pull Request Process

1. **Create a branch**: `git checkout -b feature/your-feature-name`

2. **Make your changes**: Follow the code style guidelines and add tests for
   new functionality.

3. **Run all checks locally**:
   ```bash
   dart format --output=none --set-exit-if-changed .
   dart analyze --fatal-infos
   dart test
   ```

4. **Update documentation**: If your changes affect the public API, update
   docstrings and the README if necessary.

5. **Commit your changes**: Use clear, descriptive commit messages following
   [Conventional Commits](https://www.conventionalcommits.org/) style when
   possible:
   ```
   feat: add AddGetterTransform for generating getters
   fix: handle empty class bodies in CodeEditor
   docs: improve CodemodContext docstrings
   test: add edge case tests for template rendering
   ```

6. **Push to your fork**: `git push origin feature/your-feature-name`

7. **Open a Pull Request**: Include a clear description of the changes,
   motivation, and any breaking changes.

## Reporting Issues

When reporting issues, please include:

- Dart SDK version (`dart --version`)
- Package version from `pubspec.yaml`
- Minimal reproduction steps
- Expected vs actual behavior
- Any relevant error messages or stack traces

## Architecture Overview

Understanding the architecture helps contribute effectively:

```
┌─────────────────────────────────────────────────────────────────┐
│                        CodemodRunner                            │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │
│  │ CLI Parsing │→│ Context     │→│ Collect Changes         │  │
│  │             │  │ Building    │  │ (Operations → Changes)  │  │
│  └─────────────┘  └─────────────┘  └─────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────────────────────────┐
│                        CodemodRecipe                            │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  Arguments (CodemodArg)                                  │    │
│  │  Operations (EditDartFileOperation, CreateFileOperation)│   │
│  │  PostExecution (DartFormatPostExecution, etc.)           │    │
│  └─────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────────────────────────┐
│                      CodemodOperation                             │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  EditDartFileOperation: Transforms → Patches            │    │
│  │  CreateFileOperation: Templates → File content           │    │
│  └─────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────┘
```

### Key Design Principles

1. **Idempotency**: Transforms must produce no changes when run again on their
   own output.

2. **Composability**: Recipes can be composed with `CodemodRecipe.compose()`,
   merging arguments and concatenating operations.

3. **Separation of concerns**: Generic primitives in this package,
   project-specific conventions in extensions outside this package.

4. **Fail fast**: Missing variables, overlapping patches, and invalid operations
   throw errors rather than silently producing incorrect output.

## Questions?

If you have questions not covered here, please open an issue or discussion on
GitHub.

Thank you for contributing to codemod_recipe!
