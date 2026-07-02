import 'dart:io';

import 'package:yaml/yaml.dart';

import '../core/recipe.dart';
import '../core/logging.dart';
import 'diagnostics.dart';
import 'host_config.dart';
import 'recipe_compiler.dart';

/// Result of loading YAML and Dart recipes into a registry.
class YamlRecipeLoadResult {
  const YamlRecipeLoadResult({
    required this.recipes,
    required this.diagnostics,
    this.definitionsById = const {},
    this.mapsById = const {},
  });

  final Map<String, CodemodRecipe> recipes;
  final List<RecipeDiagnostic> diagnostics;
  final Map<String, YamlRecipeDefinition> definitionsById;
  final Map<String, Map<String, String>> mapsById;

  List<Map<String, Object?>> diagnosticsJson() => [
    for (final item in diagnostics) item.toJson(),
  ];
}

/// Loads YAML recipes, maps, and templates from [config].
class YamlRecipeRegistry {
  /// Loads all codemod files from the configured codemod root directory.
  /// Recursively scans for:
  /// - .yaml/.yml files (detected as recipes or maps by content)
  /// - .template files (stubble templates)
  static YamlRecipeLoadResult load(HostConfig config) {
    final diagnostics = <RecipeDiagnostic>[];
    final recipeDefinitionsById = <String, YamlRecipeDefinition>{};
    final mapDefinitionsById = <String, Map<String, String>>{};
    final templatePaths = <String, String>{};
    final idSources = <String, List<DiagnosticSource>>{};

    final codemodDir = Directory(config.codemodRootPath);
    final mapIdSources = <String, List<DiagnosticSource>>{};

    if (codemodDir.existsSync()) {
      for (final entity in codemodDir.listSync(recursive: true)) {
        if (entity is! File) continue;

        final path = entity.path;
        final relativePath = _relativePath(config.workspaceRoot, path);

        if (_isYaml(path)) {
          _processYamlFile(
            path: path,
            relativePath: relativePath,
            workspaceRoot: config.workspaceRoot,
            recipeDefinitionsById: recipeDefinitionsById,
            mapDefinitionsById: mapDefinitionsById,
            idSources: idSources,
            mapIdSources: mapIdSources,
            diagnostics: diagnostics,
          );
        } else if (_isTemplate(path)) {
          final templateName = _deriveTemplateName(
            relativePath,
            config.codemodRoot,
          );
          templatePaths[templateName] = path;
        }
      }
    }

    for (final entry in config.dartRecipes.entries) {
      idSources
          .putIfAbsent(entry.key, () => [])
          .add(DiagnosticSource(file: '<dart:${entry.key}>'));
    }

    // Check for duplicate IDs within recipes only
    final rejectedIds = <String>{};
    for (final entry in idSources.entries) {
      if (entry.value.length < 2) continue;
      rejectedIds.add(entry.key);
      logger.yamlWarning("Duplicate recipe id '${entry.key}'");
      diagnostics.add(RecipeDiagnostics.duplicateRecipeId(entry.key, entry.value));
    }

    // Check for duplicate IDs within maps only
    for (final entry in mapIdSources.entries) {
      if (entry.value.length < 2) continue;
      rejectedIds.add(entry.key);
      logger.yamlWarning("Duplicate map id '${entry.key}'");
      diagnostics.add(RecipeDiagnostics.duplicateMapId(entry.key, entry.value));
    }

    final compiler = YamlRecipeCompiler(
      config: config,
      definitionsById: {
        for (final entry in recipeDefinitionsById.entries)
          if (!rejectedIds.contains(entry.key)) entry.key: entry.value,
      },
      dartRecipes: {
        for (final entry in config.dartRecipes.entries)
          if (!rejectedIds.contains(entry.key)) entry.key: entry.value,
      },
      mapsById: {
        for (final entry in mapDefinitionsById.entries)
          if (!rejectedIds.contains(entry.key)) entry.key: entry.value,
      },
    );

    final recipes = <String, CodemodRecipe>{};

    for (final entry in recipeDefinitionsById.entries) {
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

    return YamlRecipeLoadResult(
      recipes: recipes,
      diagnostics: diagnostics,
      definitionsById: {
        for (final entry in recipeDefinitionsById.entries)
          if (!rejectedIds.contains(entry.key)) entry.key: entry.value,
      },
      mapsById: {
        for (final entry in mapDefinitionsById.entries)
          if (!rejectedIds.contains(entry.key)) entry.key: entry.value,
      },
    );
  }

  /// Compiles an inline recipe document using registry context from [load].
  static CompileResult compileInline({
    required HostConfig config,
    required YamlMap document,
    required Map<String, YamlRecipeDefinition> definitionsById,
    required Map<String, CodemodRecipe> dartRecipes,
    required Map<String, Map<String, String>> mapsById,
  }) {
    final id = document['id']?.toString() ?? '__inline__';
    final definition = YamlRecipeDefinition(
      id: id,
      filePath: '<inline>',
      document: document,
    );
    final compiler = YamlRecipeCompiler(
      config: config,
      definitionsById: definitionsById,
      dartRecipes: dartRecipes,
      mapsById: mapsById,
    );
    return compiler.compile(definition);
  }

  static void _processYamlFile({
    required String path,
    required String relativePath,
    required String workspaceRoot,
    required Map<String, YamlRecipeDefinition> recipeDefinitionsById,
    required Map<String, Map<String, String>> mapDefinitionsById,
    required Map<String, List<DiagnosticSource>> idSources,
    required Map<String, List<DiagnosticSource>> mapIdSources,
    required List<RecipeDiagnostic> diagnostics,
  }) {
    try {
      final content = File(path).readAsStringSync();
      final doc = loadYaml(content) as YamlMap?;

      if (doc == null) {
        diagnostics.add(
          RecipeDiagnostics.yamlParseError(
            'Failed to parse YAML: root must be a map',
            relativePath,
          ),
        );
        return;
      }

      final id = doc['id']?.toString();
      if (id == null || id.isEmpty) {
        diagnostics.add(RecipeDiagnostics.missingId(relativePath));
        return;
      }

      // Detect type by content
      if (doc.containsKey('steps')) {
        // This is a recipe
        final definition = parseYamlRecipeFile(relativePath, content);
        recipeDefinitionsById[id] = definition;
        idSources
            .putIfAbsent(id, () => [])
            .add(DiagnosticSource(file: relativePath));
      } else if (doc.containsKey('entries') && doc['entries'] is YamlMap) {
        // This is a map
        final entries = <String, String>{};
        (doc['entries'] as YamlMap).forEach((key, value) {
          entries[key.toString()] = value?.toString() ?? '';
        });
        mapDefinitionsById[id] = entries;
        // Track map sources for duplicate detection
        mapIdSources
            .putIfAbsent(id, () => [])
            .add(DiagnosticSource(file: relativePath));
        // Maps are tracked separately, not in recipe idSources
      } else {
        // YAML has an id but no steps or entries - error
        diagnostics.add(RecipeDiagnostics.unknownYamlType(relativePath));
      }
    } catch (error) {
      logger.yamlError('Failed to process YAML file $relativePath', error);
      diagnostics.add(RecipeDiagnostics.yamlParseError('$error', relativePath));
    }
  }

  /// Derives a template name from its relative path.
  /// E.g., ".codemod/templates/foo/bar.template" with codemodRoot ".codemod" -> "templates/foo/bar"
  static String _deriveTemplateName(String relativePath, String codemodRoot) {
    // Remove codemodRoot prefix
    final normalizedRoot = codemodRoot.replaceAll('\\', '/');
    final normalizedPath = relativePath.replaceAll('\\', '/');

    if (normalizedPath.startsWith('$normalizedRoot/')) {
      final withoutRoot = normalizedPath.substring(normalizedRoot.length + 1);
      // Remove .template extension
      return withoutRoot.replaceFirst('.template', '');
    }
    // Fallback: just use filename without extension
    return normalizedPath.split('/').last.replaceFirst('.template', '');
  }

  static bool _isYaml(String path) {
    return path.endsWith('.yaml') || path.endsWith('.yml');
  }

  static bool _isTemplate(String path) {
    return path.endsWith('.template');
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
