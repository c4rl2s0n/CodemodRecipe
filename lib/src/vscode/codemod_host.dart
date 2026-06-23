import 'dart:convert';
import 'dart:io';

import '../core/context.dart';
import '../core/operation.dart';
import '../core/post_execution.dart';
import '../core/recipe.dart';
import '../core/runner.dart';
import '../core/constants.dart';
import '../yaml/diagnostics.dart';
import '../yaml/host_config.dart';
import '../yaml/recipe_registry.dart';
import '../dart_codegen/ast_helpers/ast_focus.dart';
import 'diff_service.dart';
import 'patch_selector.dart';
import 'recipe_schema.dart';

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
/// - `{"command": "reload"}`
/// - `{"command": "validate"}`
/// - `--stdio-server` argument: keeps process alive and reads one JSON command
///   per stdin line.
class CodemodHost {
  /// Recipes available to the extension, keyed by a stable id.
  Map<String, CodemodRecipe> _recipes;

  /// Read-only view of loaded recipes.
  Map<String, CodemodRecipe> get recipes => Map.unmodifiable(_recipes);

  /// Project-wide code generation preferences for all recipes in this host.
  final CodemodPreferences preferences;

  /// Optional YAML configuration used to load [recipes].
  final HostConfig? hostConfig;

  List<RecipeDiagnostic> _diagnostics = [];

  /// Load-time diagnostics from the most recent registry load.
  List<RecipeDiagnostic> get diagnostics => List.unmodifiable(_diagnostics);

  final Map<String, _CachedPreview> _previewCache = {};

  /// Creates a host exposing [recipes] to the VS Code extension.
  CodemodHost(
    Map<String, CodemodRecipe> recipes, {
    this.preferences = const CodemodPreferences(),
    this.hostConfig,
  }) : _recipes = Map.of(recipes);

  /// Creates a host that loads YAML recipes from [config].
  factory CodemodHost.fromConfig(HostConfig config) {
    final host = CodemodHost(
      const {},
      preferences: config.preferences,
      hostConfig: config,
    );
    host._reloadFromConfig();
    return host;
  }

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
  factory CodemodHost.fromList(
    Iterable<CodemodRecipe> recipes, {
    CodemodPreferences preferences = const CodemodPreferences(),
  }) {
    return CodemodHost({
      for (final recipe in recipes) recipe.name: recipe,
    }, preferences: preferences);
  }

  void _reloadFromConfig() {
    final config = hostConfig;
    if (config == null) {
      throw StateError('hostConfig is required to reload YAML recipes');
    }

    final result = YamlRecipeRegistry.load(config);
    _recipes = result.recipes;
    _diagnostics = result.diagnostics;
    _previewCache.clear();
  }

  List<Map<String, Object?>> _diagnosticsJson() {
    return [for (final item in _diagnostics) item.toJson()];
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
    await for (final line
        in stdin.transform(utf8.decoder).transform(const LineSplitter())) {
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
        _withMetrics(response, {
          'command': command,
          'runMs': runWatch.elapsedMilliseconds,
        }),
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
        response = {
          'ok': true,
          'recipes': RecipeSchema.registryToJson(_recipes),
          'diagnostics': _diagnosticsJson(),
        };
        break;
      case 'reload':
        if (hostConfig == null) {
          response = {
            'ok': false,
            'error': 'Reload requires a YAML-enabled host (fromConfig)',
          };
          break;
        }
        _reloadFromConfig();
        response = {
          'ok': true,
          'recipes': RecipeSchema.registryToJson(_recipes),
          'diagnostics': _diagnosticsJson(),
        };
        break;
      case 'validate':
        if (hostConfig == null) {
          response = {
            'ok': false,
            'error': 'Validate requires a YAML-enabled host (fromConfig)',
          };
          break;
        }
        _reloadFromConfig();
        response = {
          'ok': _diagnostics.every(
            (item) => item.severity != DiagnosticSeverity.error,
          ),
          'diagnostics': _diagnosticsJson(),
        };
        break;
      case 'describe':
        final id = request['recipe'] as String?;
        if (id == null) {
          response = {'ok': false, 'error': 'Missing "recipe" id'};
          break;
        }
        final recipe = _recipes[id];
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
      case 'generateAstPath':
        response = await _generateAstPath(request);
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
    final snippetLines = _parseSnippetLines(request['snippetLines']);

    final validationError = _validate(recipe, context);
    if (validationError != null) {
      return {'ok': false, 'error': validationError};
    }

    final collectWatch = Stopwatch()..start();
    final collected = await _collectChangesWithCache(
      request,
      recipe,
      context,
      allowReuse: true,
      updateCache: true,
    );
    collectWatch.stop();
    final changedFiles = collected.changes.where((c) => c.hasChanges).toList();
    final serializeWatch = Stopwatch()..start();
    final files = await DiffService.changesToJson(
      changedFiles,
      includeContents: false,
      includePatchReplacements: false,
      snippetLines: snippetLines,
    );
    serializeWatch.stop();

    return {
      'ok': true,
      'recipe': recipe.name,
      'files': files,
      '_timingsMs': {
        'collectChanges': collectWatch.elapsedMilliseconds,
        'serializeDiff': serializeWatch.elapsedMilliseconds,
        'reusedPreviewCache': collected.reusedCache ? 1 : 0,
      },
    };
  }

  int _parseSnippetLines(Object? raw) {
    if (raw is num) {
      final value = raw.toInt();
      if (value < 1) return 1;
      if (value > 20) return 20;
      return value;
    }
    return 5;
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
    final collected = await _collectChangesWithCache(
      request,
      recipe,
      context,
      allowReuse: true,
      updateCache: true,
    );
    collectWatch.stop();
    final changes = collected.changes;
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
        'reusedPreviewCache': collected.reusedCache ? 1 : 0,
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
    final collected = await _collectChangesWithCache(
      request,
      recipe,
      context,
      allowReuse: true,
      updateCache: true,
    );
    collectWatch.stop();
    final changedFiles = collected.changes.where((c) => c.hasChanges).toList();
    final selectWatch = Stopwatch()..start();
    final selected = PatchSelector.apply(changedFiles, selection);
    selectWatch.stop();

    final applyWatch = Stopwatch()..start();
    for (final change in selected) {
      await change.apply();
    }
    applyWatch.stop();
    _previewCache.remove(_cacheKey(request));

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
        'reusedPreviewCache': collected.reusedCache ? 1 : 0,
      },
    };
  }

  Future<Map<String, Object?>> _generateAstPath(
    Map<String, Object?> request,
  ) async {
    final path = request['path'] as String?;
    final offset = request['offset'] as int?;

    if (path == null || offset == null) {
      return {'ok': false, 'error': 'Missing path or offset'};
    }

    try {
      final source = await File(path).readAsString();
      final focus = AstFocus.parse(source, path: path);
      final astPath = focus.generatePathAtOffset(offset);

      if (astPath != null) {
        return {
          'ok': true,
          'path': {
            'navigate': astPath.navigate
                .map(
                  (step) => {
                    'kind': step.kind?.name,
                    'name': step.name,
                    'match': step.match,
                  },
                )
                .toList(),
            'anchor': astPath.anchor.toString(),
            'offset': offset,
          },
        };
      } else {
        return {'ok': false, 'error': 'No AST node found at offset $offset'};
      }
    } catch (error) {
      return {'ok': false, 'error': error.toString()};
    }
  }

  _ResolvedRecipe _resolveRecipe(Map<String, Object?> request) {
    final id = request['recipe'] as String?;
    if (id == null) {
      return const _ResolvedRecipe(error: 'Missing "recipe" id');
    }
    final recipe = _recipes[id];
    if (recipe == null) {
      return _ResolvedRecipe(error: 'Unknown recipe: $id');
    }

    final context = CodemodContext(const {}, preferences);
    final rawArgs = request['args'];
    final requestValues = <String, String>{};
    if (rawArgs is Map) {
      rawArgs.forEach((key, value) {
        if (value != null) {
          requestValues[key.toString()] = value.toString();
        }
      });
    }

    for (final arg in recipe.args) {
      if (arg.hidden) {
        arg.contributeToContext(context);
        continue;
      }
      final error = arg.contributeToContext(
        context,
        rawValue: requestValues[arg.name],
        hiddenWins: true,
      );
      if (error != null && error.startsWith('--')) {
        return _ResolvedRecipe(
          error: 'Missing required arguments: ${arg.name}',
        );
      }
      if (error != null) {
        return _ResolvedRecipe(error: error);
      }
    }

    return _ResolvedRecipe(recipe: recipe, context: context);
  }

  String? _validate(CodemodRecipe recipe, CodemodContext context) {
    final missing = <String>[];
    for (final arg in recipe.args) {
      if (arg.required && !context.has(arg.name)) {
        missing.add(arg.name);
      }
    }
    if (missing.isNotEmpty) {
      return 'Missing required arguments: ${missing.join(', ')}';
    }

    for (final arg in recipe.args) {
      final message = arg.validateInContext(context);
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

  Future<_CollectedChanges> _collectChangesWithCache(
    Map<String, Object?> request,
    CodemodRecipe recipe,
    CodemodContext context, {
    required bool allowReuse,
    required bool updateCache,
  }) async {
    final key = _cacheKey(request);
    final cached = _previewCache[key];
    if (allowReuse && cached != null && await _isCacheValid(cached)) {
      return _CollectedChanges(changes: cached.changes, reusedCache: true);
    }

    final changes = await CodemodRunner(
      recipe,
      preferences: preferences,
    ).collectChanges(context);
    if (updateCache) {
      _previewCache[key] = _CachedPreview(
        changes: changes,
        snapshots: await _captureSnapshots(changes),
      );
    }
    return _CollectedChanges(changes: changes, reusedCache: false);
  }

  String _cacheKey(Map<String, Object?> request) {
    final recipeId = request['recipe']?.toString() ?? '';
    final rawArgs = request['args'];
    if (rawArgs is! Map) {
      return recipeId;
    }
    final normalized = <String, String>{
      for (final entry in rawArgs.entries)
        entry.key.toString(): entry.value?.toString() ?? '',
    };
    final sortedKeys = normalized.keys.toList()..sort();
    final sortedMap = <String, String>{
      for (final key in sortedKeys) key: normalized[key]!,
    };
    return '$recipeId:${jsonEncode(sortedMap)}';
  }

  Future<Map<String, _FileSnapshot>> _captureSnapshots(
    List<FileChange> changes,
  ) async {
    final snapshots = <String, _FileSnapshot>{};
    for (final change in changes) {
      snapshots[change.path] = await _snapshotForPath(change.path);
    }
    return snapshots;
  }

  Future<bool> _isCacheValid(_CachedPreview cached) async {
    for (final entry in cached.snapshots.entries) {
      final current = await _snapshotForPath(entry.key);
      if (!current.matches(entry.value)) {
        return false;
      }
    }
    return true;
  }

  Future<_FileSnapshot> _snapshotForPath(String path) async {
    final file = File(path);
    if (!await file.exists()) {
      return const _FileSnapshot(exists: false, modifiedMs: -1, size: -1);
    }
    final stat = await file.stat();
    return _FileSnapshot(
      exists: true,
      modifiedMs: stat.modified.millisecondsSinceEpoch,
      size: stat.size,
    );
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
    return {...response, '_hostMetrics': metrics};
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

class _CollectedChanges {
  final List<FileChange> changes;
  final bool reusedCache;

  const _CollectedChanges({required this.changes, required this.reusedCache});
}

class _CachedPreview {
  final List<FileChange> changes;
  final Map<String, _FileSnapshot> snapshots;

  const _CachedPreview({required this.changes, required this.snapshots});
}

class _FileSnapshot {
  final bool exists;
  final int modifiedMs;
  final int size;

  const _FileSnapshot({
    required this.exists,
    required this.modifiedMs,
    required this.size,
  });

  bool matches(_FileSnapshot other) {
    return exists == other.exists &&
        modifiedMs == other.modifiedMs &&
        size == other.size;
  }
}
