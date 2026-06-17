import 'dart:convert';
import 'dart:io';

import 'package:args/args.dart';
import 'package:codemod_recipe/codemod_recipe_vscode.dart';

/// VS Code extension host entrypoint for YAML recipes.
///
/// Stdio server (used by the VS Code extension):
///   dart run bin/codemod_host.dart --stdio-server
///
/// Validate recipes:
///   dart run bin/codemod_host.dart --validate
///
/// For CLI usage, use: dart run bin/codemod.dart <recipe.yaml> [args]
Future<void> main(List<String> arguments) async {
  final parser = HostConfig.buildArgParser()
    ..addFlag('help', abbr: 'h', negatable: false);

  late ArgResults results;

  try {
    // Parse all arguments as host arguments
    results = parser.parse(arguments);
  } on FormatException catch (error) {
    stderr.writeln('Error: ${error.message}');
    _printUsage(parser);
    exit(1);
  }

  if (results['help'] == true) {
    _printUsage(parser);
    exit(0);
  }

  final config = HostConfig.fromArgResults(results);

  if (results['validate'] == true) {
    final loadResult = YamlRecipeRegistry.load(config);
    stdout.writeln(
      jsonEncode({
        'ok': loadResult.diagnostics.every(
          (item) => item.severity != DiagnosticSeverity.error,
        ),
        'diagnostics': loadResult.diagnosticsJson(),
        'recipes': RecipeSchema.registryToJson(loadResult.recipes),
      }),
    );
    exit(
      loadResult.diagnostics.any(
            (item) => item.severity == DiagnosticSeverity.error,
          )
          ? 1
          : 0,
    );
  }

  if (results['stdio-server'] == true || _looksLikeJsonCommand(arguments)) {
    await CodemodHost.fromConfig(config).run(arguments);
    return;
  }

  // No CLI mode - direct users to codemod.dart
  stderr.writeln('For CLI usage, use: dart run bin/codemod.dart <recipe.yaml> [args]');
  stderr.writeln('For VS Code extension, use: dart run bin/codemod_host.dart --stdio-server');
  exit(1);
}

bool _looksLikeJsonCommand(List<String> rest) {
  if (rest.isEmpty) return false;
  final first = rest.first.trim();
  return first.startsWith('{') && first.contains('"command"');
}

void _printUsage(ArgParser parser) {
  stderr.writeln(
    'Usage: codemod_host [host options]',
  );
  stderr.writeln('');
  stderr.writeln('VS Code extension host options:');
  stderr.writeln(parser.usage);
  stderr.writeln('');
  stderr.writeln(
    'For CLI usage, use: dart run bin/codemod.dart <recipe.yaml> [args]',
  );
}
