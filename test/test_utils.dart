import 'dart:io';

import 'package:codemod_recipe/codemod_recipe.dart';
import 'package:test/test.dart';

/// Test utilities for standardizing common test patterns.
class TestUtils {
  /// Creates a temporary workspace directory for testing.
  ///
  /// Automatically adds a teardown to clean up the directory.
  ///
  /// ## Example
  /// ```dart
  /// test('test with workspace', () async {
  ///   final workspace = await TestUtils.createTempWorkspace();
  ///   // Use workspace.path for test files
  /// });
  /// ```
  static Future<Directory> createTempWorkspace(String prefix) async {
    final workspace = await Directory.systemTemp.createTemp(prefix);
    addTearDown(() => workspace.deleteSync(recursive: true));
    return workspace;
  }

  /// Copies a file from source to destination.
  ///
  /// Creates parent directories if they don't exist.
  static Future<void> copyFile(String sourcePath, String destPath) async {
    final file = File(sourcePath);
    final destFile = File(destPath);
    
    if (!await destFile.parent.exists()) {
      await destFile.parent.create(recursive: true);
    }
    
    await file.copy(destFile.path);
  }

  /// Creates a test file with given content.
  static Future<File> createTestFile(String path, String content) async {
    final file = File(path);
    if (!await file.parent.exists()) {
      await file.parent.create(recursive: true);
    }
    await file.writeAsString(content);
    return file;
  }

  /// Asserts that a recipe loads without errors.
  static void expectRecipeLoadsSuccessfully(LoadRecipeResult result, String recipeId) {
    expect(
      result.diagnostics.where((d) => d.severity == DiagnosticSeverity.error),
      isEmpty,
      reason: 'Recipe $recipeId should load without errors',
    );
    expect(result.recipes[recipeId], isNotNull, reason: 'Recipe $recipeId should be found');
  }

  /// Asserts that a recipe has compilation errors.
  static void expectRecipeHasErrors(LoadRecipeResult result) {
    expect(
      result.diagnostics.where((d) => d.severity == DiagnosticSeverity.error),
      isNotEmpty,
      reason: 'Recipe should have compilation errors',
    );
  }

  /// Creates a standard HostConfig for testing.
  static HostConfig createTestHostConfig(String workspacePath, {String codemodRoot = '.codemod'}) {
    return HostConfig(
      workspaceRoot: workspacePath,
      codemodRoot: codemodRoot,
    );
  }

  /// Creates a CodemodContext with test values.
  static CodemodContext createTestContext({Map<String, Object?>? values}) {
    return CodemodContext(values ?? {});
  }

  /// Asserts that two strings are equal, ignoring whitespace differences.
  static void expectStringsEqualIgnoringWhitespace(String actual, String expected, {String? reason}) {
    final normalizedActual = actual.replaceAll(RegExp(r'\s+'), ' ').trim();
    final normalizedExpected = expected.replaceAll(RegExp(r'\s+'), ' ').trim();
    
    expect(normalizedActual, normalizedExpected, reason: reason);
  }

  /// Asserts that a function throws a StateError with the expected message.
  static void expectStateError(void Function() fn, String expectedMessage) {
    expect(
      () => fn(),
      throwsA(
        isA<StateError>().having((e) => e.message, 'error message', contains(expectedMessage)),
      ),
    );
  }

  /// Asserts that a function throws an ArgumentError with the expected message.
  static void expectArgumentError(void Function() fn, String expectedMessage) {
    expect(
      () => fn(),
      throwsA(
        isA<ArgumentError>().having((e) => e.message, 'error message', contains(expectedMessage)),
      ),
    );
  }

  /// Creates a test YAML recipe file in the workspace.
  static Future<File> createTestYamlRecipe(
    Directory workspace, {
    String recipeId = 'test_recipe',
    String recipeName = 'Test Recipe',
    String recipeContent = '',
  }) async {
    final recipeDir = Directory('${workspace.path}/.codemod/recipes');
    if (!await recipeDir.exists()) {
      await recipeDir.create(recursive: true);
    }
    
    final defaultContent = '''
id: $recipeId
name: $recipeName
description: A test recipe
args:
  - name: file
    required: true
operations:
  - insert:
      at: "file"
      text: "// Test insertion"
''';
    
    final content = recipeContent.isNotEmpty ? recipeContent : defaultContent;
    final recipeFile = File('${recipeDir.path}/$recipeId.yaml');
    await recipeFile.writeAsString(content);
    
    return recipeFile;
  }

  /// Loads a recipe from the given workspace.
  static Future<LoadRecipeResult> loadRecipeFromWorkspace(Directory workspace, {String codemodRoot = '.codemod'}) async {
    final config = createTestHostConfig(workspace.path, codemodRoot: codemodRoot);
    return YamlRecipeRegistry.load(config);
  }

  /// Runs a recipe and returns the result.
  static Future<String> runRecipeOnContent(CodemodRecipe recipe, String content, Map<String, String> args) async {
    final context = createTestContext(values: args);
    final changes = await recipe.collectChanges(context);
    
    if (changes.isEmpty) {
      return content;
    }
    
    // Apply the first change (simplified for testing)
    final change = changes.first;
    return applyPatches(content, change.patches);
  }
}