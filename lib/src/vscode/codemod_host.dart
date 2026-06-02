import 'dart:convert';
import 'dart:io';

import '../context.dart';
import '../operation.dart';
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
/// - `{"command": "describe", "recipe": "id"}`
/// - `{"command": "preview", "recipe": "id", "args": {..}}`
/// - `{"command": "diff", "recipe": "id", "path": "file", "args": {..}}`
/// - `{"command": "apply", "recipe": "id", "args": {..}, "selection": {..}}`
/// - `--stdio-server` argument: keeps process alive and reads one JSON command
///   per stdin line.
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
    if (args.contains('--stdio-server')) {
      await _runPersistent();
      return;
    }

    final raw = await _readStdin();
    final request = _decodeRequest(raw);
    await _handleRequest(request, fallbackCommand: raw);
  }

  Future<void> _runPersistent() async {
    await for (final line in stdin
        .transform(utf8.decoder)
        .transform(const LineSplitter())) {
      final request = _decodeRequest(line);
      await _handleRequest(request, fallbackCommand: line);
    }
  }

  Future<void> _handleRequest(
    Map<String, Object?>? request, {
    required String fallbackCommand,
  }) async {
    if (request == null) {
      _writeResponse({'ok': false, 'error': 'Invalid JSON request'});
      return;
    }

    final command = request['command']?.toString() ?? fallbackCommand;
    try {
      final runWatch = Stopwatch()..start();
      final response = await dispatch(request);
      runWatch.stop();
      _writeResponse(
        _withMetrics(response, {'command': command, 'runMs': runWatch.elapsedMilliseconds}),
      );
    } catch (error, stack) {
      _writeResponse({
        'ok': false,
        'error': error.toString(),
        'stack': stack.toString(),
      });
    }
  }

  Map<String, Object?>? _decodeRequest(String raw) {
    if (raw.trim().isEmpty) {
      return const {};
    }

    Map<String, Object?> request;
    try {
      request = jsonDecode(raw) as Map<String, Object?>;
    } catch (error) {
      stderr.writeln('Invalid JSON request: $error');
      return null;
    }
    return request;
  }

  /// Handles a single decoded [request] and returns a JSON-friendly response.
  Future<Map<String, Object?>> dispatch(Map<String, Object?> request) async {
    final command = request['command'] as String?;
    final commandWatch = Stopwatch()..start();
    Map<String, Object?> response;
    switch (command) {
      case 'list':
        response = {'ok': true, 'recipes': RecipeSchema.registryToJson(recipes)};
        break;
      case 'describe':
        final id = request['recipe'] as String?;
        if (id == null) {
          response = {'ok': false, 'error': 'Missing "recipe" id'};
          break;
        }
        final recipe = recipes[id];
        if (recipe == null) {
          response = {'ok': false, 'error': 'Unknown recipe: $id'};
          break;
        }
        response = {
          'ok': true,
          'recipe': RecipeSchema.recipeEntryToJson(id, recipe),
        };
        break;
      case 'preview':
        response = await _preview(request);
        break;
      case 'diff':
        response = await _diff(request);
        break;
      case 'apply':
        response = await _apply(request);
        break;
      default:
        response = {'ok': false, 'error': 'Unknown command: $command'};
        break;
    }
    commandWatch.stop();
    return _withMetrics(response, {
      'command': command,
      'dispatchMs': commandWatch.elapsedMilliseconds,
    });
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

    final collectWatch = Stopwatch()..start();
    final changes = await CodemodRunner(recipe).collectChanges(context);
    collectWatch.stop();
    final changedFiles = changes.where((c) => c.hasChanges).toList();
    final serializeWatch = Stopwatch()..start();
    final files = await DiffService.changesToJson(
      changedFiles,
      includeContents: false,
      includePatchReplacements: false,
    );
    serializeWatch.stop();

    return {
      'ok': true,
      'recipe': recipe.name,
      'files': files,
      '_timingsMs': {
        'collectChanges': collectWatch.elapsedMilliseconds,
        'serializeDiff': serializeWatch.elapsedMilliseconds,
      },
    };
  }

  Future<Map<String, Object?>> _diff(Map<String, Object?> request) async {
    final resolved = _resolveRecipe(request);
    if (resolved.error != null) {
      return {'ok': false, 'error': resolved.error};
    }
    final recipe = resolved.recipe!;
    final context = resolved.context!;
    final path = request['path'] as String?;
    if (path == null || path.isEmpty) {
      return {'ok': false, 'error': 'Missing "path"'};
    }

    final validationError = _validate(recipe, context);
    if (validationError != null) {
      return {'ok': false, 'error': validationError};
    }

    final collectWatch = Stopwatch()..start();
    final changes = await CodemodRunner(recipe).collectChanges(context);
    collectWatch.stop();
    final target = changes.firstWhere(
      (change) => change.path == path,
      orElse: () => _MissingFileChange(path),
    );
    if (target is _MissingFileChange) {
      return {'ok': false, 'error': 'No preview change found for $path'};
    }

    final serializeWatch = Stopwatch()..start();
    final file = await DiffService.changeToJson(target);
    serializeWatch.stop();

    return {
      'ok': true,
      'recipe': recipe.name,
      'file': file,
      '_timingsMs': {
        'collectChanges': collectWatch.elapsedMilliseconds,
        'serializeDiff': serializeWatch.elapsedMilliseconds,
      },
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
    final collectWatch = Stopwatch()..start();
    final changes = await CodemodRunner(recipe).collectChanges(context);
    collectWatch.stop();
    final changedFiles = changes.where((c) => c.hasChanges).toList();
    final selectWatch = Stopwatch()..start();
    final selected = PatchSelector.apply(changedFiles, selection);
    selectWatch.stop();

    final applyWatch = Stopwatch()..start();
    for (final change in selected) {
      await change.apply();
    }
    applyWatch.stop();

    final postExecutionWatch = Stopwatch()..start();
    if (selected.isNotEmpty) {
      final result = CodemodRunResult(changes: selected);
      for (final action in recipe.postExecution) {
        await action.run(context, result);
      }
    }
    postExecutionWatch.stop();

    return {
      'ok': true,
      'recipe': recipe.name,
      'applied': [for (final change in selected) change.path],
      '_timingsMs': {
        'collectChanges': collectWatch.elapsedMilliseconds,
        'selectPatches': selectWatch.elapsedMilliseconds,
        'applyChanges': applyWatch.elapsedMilliseconds,
        'postExecution': postExecutionWatch.elapsedMilliseconds,
      },
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

  Map<String, Object?> _withMetrics(
    Map<String, Object?> response,
    Map<String, Object?> metrics,
  ) {
    return {
      ...response,
      '_hostMetrics': metrics,
    };
  }
}

class _ResolvedRecipe {
  final CodemodRecipe? recipe;
  final CodemodContext? context;
  final String? error;

  const _ResolvedRecipe({this.recipe, this.context, this.error});
}

class _MissingFileChange implements FileChange {
  @override
  final String path;

  const _MissingFileChange(this.path);

  @override
  bool get hasChanges => false;

  @override
  bool get shouldFormat => false;

  @override
  Future<void> apply() async {}

  @override
  String preview() => '';
}
