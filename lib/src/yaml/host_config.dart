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
    this.codemodRoot = '.codemod',
    this.preferences = const CodemodPreferences(),
    this.dartRecipes = const {},
  });

  /// Workspace root used to resolve relative paths.
  final String workspaceRoot;

  /// Root directory for all codemod files, relative to [workspaceRoot].
  /// Contains recipes (.yaml), maps (.yaml), and templates (.template).
  final String codemodRoot;

  /// Global code generation preferences.
  final CodemodPreferences preferences;

  /// Optional Dart-registered recipes for advanced coexistence.
  final Map<String, CodemodRecipe> dartRecipes;

  /// Absolute path to the codemod root directory.
  String get codemodRootPath => _resolveUnderWorkspace(codemodRoot).path;

  Directory _resolveUnderWorkspace(String relativePath) {
    final normalized = _normalizeRelative(relativePath);
    return Directory('${workspaceRoot.replaceAll('\\', '/')}/$normalized');
  }

  /// Parses [HostConfig] from parsed CLI [results].
  static HostConfig fromArgResults(ArgResults results) {
    final workspaceRoot = (results['workspace-root'] as String?) ?? '.';
    return HostConfig(
      workspaceRoot: Directory(workspaceRoot).absolute.path,
      codemodRoot: results['codemod-root'] as String? ?? '.codemod',
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
        'codemod-root',
        help: 'Root directory for codemod files (recipes, maps, templates) (workspace-relative)',
        defaultsTo: '.codemod',
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
