import 'dart:io';

import 'package:args/args.dart';

import '../context.dart';
import '../recipe.dart';

/// Shared configuration for the generic host and CLI entrypoint.
class HostConfig {
  /// Creates host configuration.
  const HostConfig({
    required this.workspaceRoot,
    this.recipesDirectory = '.codemod/recipes',
    this.templatesRoot = '.codemod/templates',
    this.preferences = const CodemodPreferences(),
    this.dartRecipes = const {},
  });

  /// Workspace root used to resolve relative paths.
  final String workspaceRoot;

  /// Directory containing YAML recipe files, relative to [workspaceRoot].
  final String recipesDirectory;

  /// Root for `templateFile:` and `runScript:` paths, relative to [workspaceRoot].
  final String templatesRoot;

  /// Global code generation preferences.
  final CodemodPreferences preferences;

  /// Optional Dart-registered recipes for advanced coexistence.
  final Map<String, CodemodRecipe> dartRecipes;

  /// Absolute path to the recipes directory.
  String get recipesDirectoryPath =>
      _resolveUnderWorkspace(recipesDirectory).path;

  /// Absolute path to the templates root.
  String get templatesRootPath => _resolveUnderWorkspace(templatesRoot).path;

  Directory _resolveUnderWorkspace(String relativePath) {
    final normalized = _normalizeRelative(relativePath);
    return Directory('${workspaceRoot.replaceAll('\\', '/')}/$normalized');
  }

  /// Parses [HostConfig] from parsed CLI [results].
  static HostConfig fromArgResults(ArgResults results) {
    final workspaceRoot = (results['workspace-root'] as String?) ?? '.';
    return HostConfig(
      workspaceRoot: Directory(workspaceRoot).absolute.path,
      recipesDirectory:
          results['recipes-dir'] as String? ?? '.codemod/recipes',
      templatesRoot:
          results['templates-root'] as String? ?? '.codemod/templates',
    );
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
