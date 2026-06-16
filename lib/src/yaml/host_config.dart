import 'dart:io';

import 'package:args/args.dart';

import '../context.dart';
import '../dart_codegen/field_spec.dart';
import '../recipe.dart';

/// Shared configuration for the generic host and CLI entrypoint.
class HostConfig {
  /// Creates host configuration.
  const HostConfig({
    required this.workspaceRoot,
    this.recipesDirectory = '.codemod/recipes',
    this.templatesRoot = '.codemod/templates',
    this.mapsDirectory = '.codemod/maps',
    this.preferences = const CodemodPreferences(),
    this.dartRecipes = const {},
  });

  /// Workspace root used to resolve relative paths.
  final String workspaceRoot;

  /// Directory containing YAML recipe files, relative to [workspaceRoot].
  final String recipesDirectory;

  /// Root for `templateFile:` and `runScript:` paths, relative to [workspaceRoot].
  final String templatesRoot;

  /// Directory containing reusable YAML maps, relative to [workspaceRoot].
  final String mapsDirectory;

  /// Global code generation preferences.
  final CodemodPreferences preferences;

  /// Optional Dart-registered recipes for advanced coexistence.
  final Map<String, CodemodRecipe> dartRecipes;

  /// Absolute path to the recipes directory.
  String get recipesDirectoryPath =>
      _resolveUnderWorkspace(recipesDirectory).path;

  /// Absolute path to the templates root.
  String get templatesRootPath => _resolveUnderWorkspace(templatesRoot).path;

  /// Absolute path to the maps directory.
  String get mapsDirectoryPath => _resolveUnderWorkspace(mapsDirectory).path;

  Directory _resolveUnderWorkspace(String relativePath) {
    final normalized = _normalizeRelative(relativePath);
    return Directory('${workspaceRoot.replaceAll('\\', '/')}/$normalized');
  }

  /// Parses [HostConfig] from parsed CLI [results].
  static HostConfig fromArgResults(ArgResults results) {
    final workspaceRoot = (results['workspace-root'] as String?) ?? '.';
    return HostConfig(
      workspaceRoot: Directory(workspaceRoot).absolute.path,
      recipesDirectory: results['recipes-dir'] as String? ?? '.codemod/recipes',
      templatesRoot:
          results['templates-root'] as String? ?? '.codemod/templates',
      mapsDirectory: results['maps-dir'] as String? ?? '.codemod/maps',
      preferences: CodemodPreferences(
        emptyConstructorStyle: _parseEmptyConstructorStyle(
          results['empty-constructor-style'] as String?,
        ),
      ),
    );
  }

  static ConstructorParamStyle _parseEmptyConstructorStyle(String? value) {
    switch (value) {
      case 'positional':
        return ConstructorParamStyle.positional;
      default:
        return ConstructorParamStyle.named;
    }
  }

  /// Builds a shared [ArgParser] for host/CLI flags.
  static ArgParser buildArgParser() {
    return ArgParser()
      ..addOption(
        'workspace-root',
        help: 'Workspace root for resolving relative paths',
        defaultsTo: '.',
      )
      ..addOption(
        'recipes-dir',
        help: 'Directory containing YAML recipes (workspace-relative)',
        defaultsTo: '.codemod/recipes',
      )
      ..addOption(
        'templates-root',
        help: 'Root for templateFile and runScript paths (workspace-relative)',
        defaultsTo: '.codemod/templates',
      )
      ..addOption(
        'maps-dir',
        help: 'Directory containing reusable YAML maps (workspace-relative)',
        defaultsTo: '.codemod/maps',
      )
      ..addOption(
        'empty-constructor-style',
        help: 'Style for empty constructor params (named|positional)',
        defaultsTo: 'named',
        allowed: ['named', 'positional'],
      )
      ..addFlag(
        'stdio-server',
        help: 'Run JSON command server on stdin/stdout',
        negatable: false,
      )
      ..addFlag(
        'validate',
        help: 'Validate YAML recipes and print diagnostics as JSON',
        negatable: false,
      );
  }

  static String _normalizeRelative(String path) {
    final segments = path
        .replaceAll('\\', '/')
        .split('/')
        .where((segment) => segment.isNotEmpty && segment != '.')
        .toList();

    if (segments.any((segment) => segment == '..')) {
      throw ArgumentError.value(path, 'path', 'Path must not contain ".."');
    }

    return segments.join('/');
  }
}
