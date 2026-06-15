import 'dart:io';

import '../recipe.dart';
import 'diagnostics.dart';
import 'host_config.dart';
import 'recipe_compiler.dart';

/// Result of loading YAML and Dart recipes into a registry.
class YamlRecipeLoadResult {
  /// Creates a load result.
  const YamlRecipeLoadResult({
    required this.recipes,
    required this.diagnostics,
  });

  /// Successfully loaded recipes keyed by id.
  final Map<String, CodemodRecipe> recipes;

  /// Load-time diagnostics including ID collisions.
  final List<RecipeDiagnostic> diagnostics;

  List<Map<String, Object?>> diagnosticsJson() =>
      [for (final item in diagnostics) item.toJson()];
}

/// Loads YAML recipes from [config] and merges optional Dart recipes.
class YamlRecipeRegistry {
  /// Loads recipes from the configured recipes directory.
  static YamlRecipeLoadResult load(HostConfig config) {
    final diagnostics = <RecipeDiagnostic>[];
    final definitionsById = <String, YamlRecipeDefinition>{};
    final idSources = <String, List<DiagnosticSource>>{};

    final recipesDir = config.recipesDirectoryPath;
    final directory = Directory(recipesDir);
    if (directory.existsSync()) {
      for (final file in directory
          .listSync(recursive: true)
          .whereType<File>()
          .where((file) => _isYaml(file.path))) {
        final relativePath = _relativePath(config.workspaceRoot, file.path);
        try {
          final definition = parseYamlRecipeFile(
            relativePath,
            file.readAsStringSync(),
          );
          definitionsById[definition.id] = definition;
          idSources
              .putIfAbsent(definition.id, () => [])
              .add(DiagnosticSource(file: relativePath));
        } catch (error) {
          diagnostics.add(
            RecipeDiagnostic(
              severity: DiagnosticSeverity.error,
              code: 'E_YAML_PARSE',
              message: '$error',
              sources: [DiagnosticSource(file: relativePath)],
            ),
          );
        }
      }
    }

    for (final entry in config.dartRecipes.entries) {
      idSources
          .putIfAbsent(entry.key, () => [])
          .add(DiagnosticSource(file: '<dart:${entry.key}>'));
    }

    final rejectedIds = <String>{};
    for (final entry in idSources.entries) {
      if (entry.value.length < 2) continue;
      rejectedIds.add(entry.key);
      diagnostics.add(
        RecipeDiagnostic(
          severity: DiagnosticSeverity.error,
          code: 'E_DUPLICATE_RECIPE_ID',
          message: "Duplicate recipe id '${entry.key}'",
          sources: entry.value,
        ),
      );
    }

    final compiler = YamlRecipeCompiler(
      config: config,
      definitionsById: {
        for (final entry in definitionsById.entries)
          if (!rejectedIds.contains(entry.key)) entry.key: entry.value,
      },
      dartRecipes: {
        for (final entry in config.dartRecipes.entries)
          if (!rejectedIds.contains(entry.key)) entry.key: entry.value,
      },
    );

    final recipes = <String, CodemodRecipe>{};

    for (final entry in definitionsById.entries) {
      if (rejectedIds.contains(entry.key)) continue;
      final compiled = compiler.compile(entry.value);
      diagnostics.addAll(compiled.diagnostics);
      if (compiled.recipe != null) {
        recipes[entry.key] = compiled.recipe!;
      }
    }

    for (final entry in config.dartRecipes.entries) {
      if (rejectedIds.contains(entry.key)) continue;
      recipes.putIfAbsent(entry.key, () => entry.value);
    }

    return YamlRecipeLoadResult(recipes: recipes, diagnostics: diagnostics);
  }

  static bool _isYaml(String path) {
    return path.endsWith('.yaml') || path.endsWith('.yml');
  }

  static String _relativePath(String workspaceRoot, String absolutePath) {
    final root = Directory(workspaceRoot).absolute.path.replaceAll('\\', '/');
    final file = File(absolutePath).absolute.path.replaceAll('\\', '/');
    if (file.startsWith('$root/')) {
      return file.substring(root.length + 1);
    }
    return file;
  }
}
