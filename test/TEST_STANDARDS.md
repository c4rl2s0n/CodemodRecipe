# Test Standards and Best Practices

This document outlines the standardized approach for writing tests in the codemod_recipe package.

## Table of Contents

- [General Principles](#general-principles)
- [Test Structure](#test-structure)
- [Common Patterns](#common-patterns)
- [Test Utilities](#test-utilities)
- [Naming Conventions](#naming-conventions)
- [Best Practices](#best-practices)

## General Principles

1. **Isolation**: Each test should be independent and not rely on the state of other tests.
2. **Determinism**: Tests should produce the same results regardless of when or how often they run.
3. **Clarity**: Test names and structure should clearly communicate what is being tested.
4. **Maintainability**: Tests should be easy to understand, modify, and extend.

## Test Structure

### Basic Structure

```dart
import 'package:codemod_recipe/codemod_recipe.dart';
import 'package:test/test.dart';
import 'test_utils.dart';

void main() {
  group('Component Name', () {
    // Setup that runs before each test in the group
    setUp(() {
      // Initialize common test fixtures
    });

    test('should do something specific', () async {
      // Test implementation
    });

    test('should handle edge case', () {
      // Test implementation
    });

    // Teardown that runs after each test in the group
    tearDown(() {
      // Clean up resources
    });
  });
}
```

### Recommended Organization

```
test/
├── component_name/
│   ├── component_name_test.dart      # Main component tests
│   ├── subcomponent_test.dart        # Subcomponent tests
│   └── fixtures/                     # Test data files
├── integration/                     # Integration tests
├── test_utils.dart                   # Shared test utilities
└── TEST_STANDARDS.md                 # This document
```

## Common Patterns

### Recipe Loading Tests

```dart
test('loads valid recipe without errors', () async {
  final workspace = await TestUtils.createTempWorkspace('recipe_load_');
  await TestUtils.copyFile(
    'test/fixtures/yaml_recipes/valid_recipe.yaml',
    '${workspace.path}/.codemod/recipes/valid_recipe.yaml',
  );

  final result = await TestUtils.loadRecipeFromWorkspace(workspace);
  
  TestUtils.expectRecipeLoadsSuccessfully(result, 'valid_recipe');
  expect(result.recipes['valid_recipe']!.operations.length, 2);
});
```

### Recipe Execution Tests

```dart
test('executes recipe and applies changes correctly', () async {
  final workspace = await TestUtils.createTempWorkspace('recipe_exec_');
  
  // Create test recipe
  await TestUtils.createTestYamlRecipe(
    workspace,
    recipeId: 'test_insert',
    recipeContent: '''
id: test_insert
name: Test Insert
args:
  - name: file
    required: true
operations:
  - insert:
      at: "file"
      text: "// Inserted by test"
''',
  );

  // Load and execute recipe
  final result = await TestUtils.loadRecipeFromWorkspace(workspace);
  final recipe = result.recipes['test_insert']!;
  
  // Test execution
  final content = 'class Test {}';
  final modified = await TestUtils.runRecipeOnContent(
    recipe,
    content,
    {'file': 'test.dart'},
  );
  
  expect(modified, contains('// Inserted by test'));
});
```

### Error Handling Tests

```dart
test('throws StateError for missing required argument', () {
  final context = TestUtils.createTestContext();
  
  TestUtils.expectStateError(
    () => context.require<String>('missing_arg'),
    'Required variable "missing_arg" is not set',
  );
});
```

### String Comparison Tests

```dart
test('generates correct output format', () {
  final actual = generateSomeOutput();
  final expected = 'expected output';
  
  // For exact matching
  expect(actual, expected);
  
  // For whitespace-insensitive matching
  TestUtils.expectStringsEqualIgnoringWhitespace(actual, expected);
});
```

## Test Utilities

The `test_utils.dart` file provides standardized utilities for common test scenarios:

### Workspace Management
- `createTempWorkspace(prefix)` - Creates temporary workspace with auto-cleanup
- `copyFile(source, dest)` - Copies test files
- `createTestFile(path, content)` - Creates test files with content

### Recipe Testing
- `expectRecipeLoadsSuccessfully(result, recipeId)` - Asserts recipe loads without errors
- `expectRecipeHasErrors(result)` - Asserts recipe has compilation errors
- `createTestYamlRecipe(workspace, ...)` - Creates test YAML recipes
- `loadRecipeFromWorkspace(workspace)` - Loads recipes from workspace
- `runRecipeOnContent(recipe, content, args)` - Runs recipe on test content

### Assertion Helpers
- `expectStringsEqualIgnoringWhitespace(actual, expected)` - Whitespace-insensitive string comparison
- `expectStateError(fn, message)` - Asserts StateError with specific message
- `expectArgumentError(fn, message)` - Asserts ArgumentError with specific message

### Configuration
- `createTestHostConfig(workspacePath)` - Creates standard HostConfig
- `createTestContext(values)` - Creates CodemodContext with test values

## Naming Conventions

### Test Files
- Use lowercase with underscores: `component_name_test.dart`
- Place in appropriate subdirectory: `test/component_name/`

### Test Groups
- Use the component/class name: `group('CodemodContext', () { ... })`
- For complex components, use sub-groups: `group('CodemodContext.get', () { ... })`

### Test Names
- Use `should` or `when` style: `test('should return null for missing values', () { ... })`
- Be specific about expected behavior: `test('throws StateError when required arg is missing', () { ... })`
- For parameterized tests: `test('should handle $inputType input', () { ... })`

### Variables
- `actual` - The actual result being tested
- `expected` - The expected result
- `result` - Return value from functions under test
- `error` - Caught exceptions
- Use descriptive names for test-specific variables

## Best Practices

### Test Organization
1. **Group related tests**: Use `group()` to organize tests by component or feature
2. **Order matters**: Arrange tests from simple to complex within a group
3. **One assertion per test**: When possible, test one specific behavior per test
4. **Avoid test interdependence**: Each test should set up its own state

### Test Data
1. **Use realistic data**: Test with data that resembles real-world usage
2. **Test edge cases**: Include tests for boundary conditions and error cases
3. **Avoid magic values**: Use named constants or variables for test values
4. **Keep fixtures organized**: Store test data files in `test/fixtures/`

### Performance
1. **Avoid heavy operations**: Don't do file I/O or network calls unless necessary
2. **Use setUp/tearDown**: For expensive operations that can be shared
3. **Mock external dependencies**: When testing components in isolation

### Documentation
1. **Add comments for complex tests**: Explain why a test exists, not what it does
2. **Update documentation**: Keep this standards document current
3. **Use clear variable names**: Make tests self-documenting

### Maintenance
1. **Run all tests before committing**: Ensure no regressions
2. **Add tests for new features**: Maintain good coverage
3. **Refactor tests**: Improve test code just like production code
4. **Review test changes**: Treat test code with the same care as production code

## Example Test File

```dart
import 'package:codemod_recipe/codemod_recipe.dart';
import 'package:test/test.dart';
import 'test_utils.dart';

void main() {
  group('StringUtils', () {
    group('toSnakeCase', () {
      test('converts PascalCase to snake_case', () {
        expect(StringUtils.toSnakeCase('UserProfile'), 'user_profile');
        expect(StringUtils.toSnakeCase('HTMLParser'), 'html_parser');
      });

      test('converts camelCase to snake_case', () {
        expect(StringUtils.toSnakeCase('userProfile'), 'user_profile');
        expect(StringUtils.toSnakeCase('htmlParser'), 'html_parser');
      });

      test('handles existing snake_case', () {
        expect(StringUtils.toSnakeCase('already_snake'), 'already_snake');
      });
    });

    group('toCamelCase', () {
      test('converts PascalCase to camelCase', () {
        expect(StringUtils.toCamelCase('UserProfile'), 'userProfile');
      });

      test('handles single word', () {
        expect(StringUtils.toCamelCase('User'), 'user');
      });
    });
  });

  group('CodemodContext', () {
    late CodemodContext context;

    setUp(() {
      context = TestUtils.createTestContext(values: {
        'name': 'TestValue',
        'count': 42,
      });
    });

    group('get', () {
      test('returns typed values', () {
        expect(context.get<String>('name'), 'TestValue');
        expect(context.get<int>('count'), 42);
      });

      test('returns null for missing values', () {
        expect(context.get<String>('missing'), isNull);
      });
    });

    group('require', () {
      test('returns values for existing keys', () {
        expect(context.require<String>('name'), 'TestValue');
      });

      test('throws for missing required values', () {
        TestUtils.expectStateError(
          () => context.require<String>('missing'),
          'Required variable "missing" is not set',
        );
      });
    });
  });
}
```

## Running Tests

```bash
# Run all tests
dart test

# Run tests in a specific file
dart test test/component_name_test.dart

# Run tests with specific tags
dart test --tags integration

# Run tests with coverage
dart test --coverage=coverage
```

## Test Coverage Goals

- **Unit tests**: 80%+ coverage for core components
- **Integration tests**: Key workflows and common use cases
- **Regression tests**: For all reported bugs and issues
- **Edge case tests**: Boundary conditions and error handling

## Continuous Improvement

Regularly review and improve the test suite:
- Add missing test cases
- Refactor complex or brittle tests
- Update standards as patterns emerge
- Remove redundant or obsolete tests
- Ensure tests run quickly and reliably