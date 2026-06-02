import 'dart:convert';
import 'dart:io';

import '../context.dart';
import '../post_execution.dart';
import '../recipe.dart';
import '../runner.dart';
import 'diff_service.dart';
import 'patch_selector.dart';
import 'recipe_schema.dart';

/// Marks the start of the JSON response block on stdout.
///
/// Post-execution actions (for example `dart format`) may print to stdout, so
/// the response is wrapped in unambiguous markers the extension can scan for.
const String kResultBegin = '__CODEMOD_RESULT_BEGIN__';

/// Marks the end of the JSON response block on stdout.
const String kResultEnd = '__CODEMOD_RESULT_END__';

/// Headless entry point that bridges recipes to the VS Code extension.
///
/// Users register their recipes by recipe id and create a tiny entry point:
///
/// ```dart
/// import 'package:codemod_recipe/codemod_recipe_vscode.dart';
/// import 'recipes.dart';
///
/// Future<void> main(List<String> args) {
///   return CodemodHost({
///     'add_method': addMethodRecipe,
///     'scaffold_feature': scaffoldFeatureRecipe,
///   }).run(args);
/// }
/// ```
///
/// The host reads a single JSON request object from stdin and writes a single
/// JSON response, wrapped in [kResultBegin]/[kResultEnd] markers, to stdout.
///
/// Supported commands:
/// - `{"command": "list"}`
/// - `{"command": "preview", "recipe": "id", "args": {..}}`
/// - `{"command": "apply", "recipe": "id", "args": {..}, "selection": {..}}`
class CodemodHost {
  /// Recipes available to the extension, keyed by a stable id.
  final Map<String, CodemodRecipe> recipes;

  /// Creates a host exposing [recipes] to the VS Code extension.
  const CodemodHost(this.recipes);

  /// Creates a host from a list of recipes keyed by each recipe's [name].
  ///
  /// This supports a simple registry pattern:
  ///
  /// ```dart
  /// final allRecipes = [featureRecipe, widgetRecipe];
  /// Future<void> main(List<String> args) {
  ///   return CodemodHost.fromList(allRecipes).run(args);
  /// }
  /// ```
  ///
  /// If two recipes share a name, the later recipe replaces the earlier one.
  factory CodemodHost.fromList(Iterable<CodemodRecipe> recipes) {
    return CodemodHost({for (final recipe in recipes) recipe.name: recipe});
  }

  /// Reads a JSON request from stdin, dispatches it, and writes the response.
  Future<void> run(List<String> args) async {
    final raw = await _readStdin();
    Map<String, Object?> request;
    try {
      request = raw.trim().isEmpty
          ? const {}
          : jsonDecode(raw) as Map<String, Object?>;
    } catch (error) {
      _writeResponse({'ok': false, 'error': 'Invalid JSON request: $error'});
      return;
    }

    try {
      final response = await dispatch(request);
      _writeResponse(response);
    } catch (error, stack) {
      _writeResponse({
        'ok': false,
        'error': error.toString(),
        'stack': stack.toString(),
      });
    }
  }

  /// Handles a single decoded [request] and returns a JSON-friendly response.
  Future<Map<String, Object?>> dispatch(Map<String, Object?> request) async {
    final command = request['command'] as String?;
    switch (command) {
      case 'list':
        return {'ok': true, 'recipes': RecipeSchema.registryToJson(recipes)};
      case 'preview':
        return _preview(request);
      case 'apply':
        return _apply(request);
      default:
        return {'ok': false, 'error': 'Unknown command: $command'};
    }
  }

  Future<Map<String, Object?>> _preview(Map<String, Object?> request) async {
    final resolved = _resolveRecipe(request);
    if (resolved.error != null) {
      return {'ok': false, 'error': resolved.error};
    }
    final recipe = resolved.recipe!;
    final context = resolved.context!;

    final validationError = _validate(recipe, context);
    if (validationError != null) {
      return {'ok': false, 'error': validationError};
    }

    final changes = await CodemodRunner(recipe).collectChanges(context);
    final changedFiles = changes.where((c) => c.hasChanges).toList();

    return {
      'ok': true,
      'recipe': recipe.name,
      'files': await DiffService.changesToJson(changedFiles),
    };
  }

  Future<Map<String, Object?>> _apply(Map<String, Object?> request) async {
    final resolved = _resolveRecipe(request);
    if (resolved.error != null) {
      return {'ok': false, 'error': resolved.error};
    }
    final recipe = resolved.recipe!;
    final context = resolved.context!;

    final validationError = _validate(recipe, context);
    if (validationError != null) {
      return {'ok': false, 'error': validationError};
    }

    final selection = _parseSelection(request['selection']);
    final changes = await CodemodRunner(recipe).collectChanges(context);
    final changedFiles = changes.where((c) => c.hasChanges).toList();
    final selected = PatchSelector.apply(changedFiles, selection);

    for (final change in selected) {
      await change.apply();
    }

    if (selected.isNotEmpty) {
      final result = CodemodRunResult(changes: selected);
      for (final action in recipe.postExecution) {
        await action.run(context, result);
      }
    }

    return {
      'ok': true,
      'recipe': recipe.name,
      'applied': [for (final change in selected) change.path],
    };
  }

  _ResolvedRecipe _resolveRecipe(Map<String, Object?> request) {
    final id = request['recipe'] as String?;
    if (id == null) {
      return const _ResolvedRecipe(error: 'Missing "recipe" id');
    }
    final recipe = recipes[id];
    if (recipe == null) {
      return _ResolvedRecipe(error: 'Unknown recipe: $id');
    }

    final context = CodemodContext();
    final rawArgs = request['args'];
    if (rawArgs is Map) {
      rawArgs.forEach((key, value) {
        if (value != null) context.set(key.toString(), value.toString());
      });
    }

    return _ResolvedRecipe(recipe: recipe, context: context);
  }

  String? _validate(CodemodRecipe recipe, CodemodContext context) {
    final missing = <String>[];
    for (final arg in recipe.args) {
      final value = context.get(arg.name);
      if ((value == null || value.isEmpty) && arg.required) {
        missing.add(arg.name);
      }
    }
    if (missing.isNotEmpty) {
      return 'Missing required arguments: ${missing.join(', ')}';
    }

    for (final arg in recipe.args) {
      final message = arg.validate?.call(context.get(arg.name), context);
      if (message != null) return message;
    }
    return null;
  }

  Map<String, FileSelection> _parseSelection(Object? raw) {
    if (raw is! Map) return const {};
    final files = raw['files'];
    if (files is! Map) return const {};
    final result = <String, FileSelection>{};
    files.forEach((key, value) {
      if (value is Map) {
        result[key.toString()] = FileSelection.fromJson(
          value.cast<String, Object?>(),
        );
      }
    });
    return result;
  }

  Future<String> _readStdin() async {
    final buffer = StringBuffer();
    await for (final chunk in stdin.transform(utf8.decoder)) {
      buffer.write(chunk);
    }
    return buffer.toString();
  }

  void _writeResponse(Map<String, Object?> response) {
    stdout.writeln(kResultBegin);
    stdout.writeln(jsonEncode(response));
    stdout.writeln(kResultEnd);
  }
}

class _ResolvedRecipe {
  final CodemodRecipe? recipe;
  final CodemodContext? context;
  final String? error;

  const _ResolvedRecipe({this.recipe, this.context, this.error});
}
