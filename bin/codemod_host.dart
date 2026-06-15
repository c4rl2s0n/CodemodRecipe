import 'dart:convert';
import 'dart:io';

import 'package:args/args.dart';
import 'package:codemod_recipe/codemod_recipe_vscode.dart';

/// Generic codemod host entrypoint for YAML recipes.
///
/// Stdio server (used by the VS Code extension):
///   dart run bin/codemod_host.dart --stdio-server
///
/// Validate recipes:
///   dart run bin/codemod_host.dart --validate
///
/// Run a recipe from CLI:
///   dart run bin/codemod_host.dart add_log_line --file lib/foo.dart --className Settings --methodName update
Future<void> main(List<String> arguments) async {
  final parser = HostConfig.buildArgParser()..addFlag('help', abbr: 'h', negatable: false);

  late ArgResults results;
  try {
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

  if (results['stdio-server'] == true || _looksLikeJsonCommand(results.rest)) {
    await CodemodHost.fromConfig(config).run(arguments);
    return;
  }

  final rest = results.rest;
  if (rest.isEmpty) {
    stderr.writeln('Error: recipe id is required');
    _printUsage(parser);
    exit(1);
  }

  final recipeId = rest.first;
  final loadResult = YamlRecipeRegistry.load(config);
  if (loadResult.diagnostics.any(
    (item) => item.severity == DiagnosticSeverity.error,
  )) {
    stderr.writeln('Recipe load failed:');
    for (final diagnostic in loadResult.diagnostics) {
      stderr.writeln('  [${diagnostic.code}] ${diagnostic.message}');
    }
    exit(1);
  }

  final recipe = loadResult.recipes[recipeId];
  if (recipe == null) {
    stderr.writeln('Unknown recipe: $recipeId');
    exit(1);
  }

  await CodemodRunner(recipe, preferences: config.preferences).run(
    rest.skip(1).toList(),
  );
}

bool _looksLikeJsonCommand(List<String> rest) {
  if (rest.isEmpty) return false;
  final first = rest.first.trim();
  return first.startsWith('{') && first.contains('"command"');
}

void _printUsage(ArgParser parser) {
  stderr.writeln('Usage: codemod_host [host options] <recipe-id> [recipe args]');
  stderr.writeln('');
  stderr.writeln(parser.usage);
}
