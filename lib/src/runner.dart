// ignore_for_file: avoid_print

import 'dart:io';

import 'package:args/args.dart';

import 'context.dart';
import 'operation.dart';
import 'patch_helpers.dart';
import 'post_execution.dart';
import 'recipe.dart';

/// Shared CLI runner for recipe-based codemods.
class CodemodRunner {
  /// Recipe executed by this runner.
  final CodemodRecipe recipe;

  /// Default preferences injected into contexts built by this runner.
  final CodemodPreferences preferences;

  /// Creates a runner for [recipe].
  const CodemodRunner(
    this.recipe, {
    this.preferences = const CodemodPreferences(),
  });

  /// Parses CLI [arguments], previews or applies edits, and exits on failure.
  Future<void> run(List<String> arguments) async {
    final parser = _buildParser();

    late ArgResults args;
    try {
      args = parser.parse(arguments);
    } on FormatException catch (error) {
      stderr.writeln('Error: ${error.message}');
      _printUsage(parser);
      exit(1);
    }

    if (args['help'] as bool) {
      _printUsage(parser);
      exit(0);
    }

    final context = _buildContext(args, parser);
    final apply = args['apply'] as bool;

    try {
      final changes = await collectChanges(context);
      final changedFiles = changes
          .where((change) => change.hasChanges)
          .toList();

      if (changedFiles.isEmpty) {
        print('No changes needed.');
        exit(0);
      }

      await _execute(changedFiles, context: context, apply: apply);
    } catch (error) {
      stderr.writeln('Error: $error');
      exit(1);
    }
  }

  ArgParser _buildParser() {
    final parser = ArgParser();
    for (final arg in recipe.args) {
      parser.addOption(
        arg.name,
        abbr: arg.abbr,
        help: arg.help,
        defaultsTo: arg.defaultsTo,
      );
    }
    parser
      ..addFlag('apply', abbr: 'a', help: 'Apply changes (default is dry-run)')
      ..addFlag('help', abbr: 'h', help: 'Show usage', negatable: false);
    return parser;
  }

  CodemodContext _buildContext(ArgResults args, ArgParser parser) {
    final context = CodemodContext(const {}, preferences);
    final missing = <String>[];

    for (final arg in recipe.args) {
      final value = args[arg.name] as String?;
      if (value == null || value.isEmpty) {
        if (arg.required) {
          missing.add('--${arg.name}');
        }
        continue;
      }
      context.set(arg.name, value);
    }

    if (missing.isNotEmpty) {
      stderr.writeln('Error: ${missing.join(', ')} required');
      _printUsage(parser);
      exit(1);
    }

    for (final arg in recipe.args) {
      final message = arg.validate?.call(context.get(arg.name), context);
      if (message != null) {
        stderr.writeln('Error: $message');
        exit(1);
      }
    }

    return context;
  }

  /// Collects and merges all planned [FileChange]s for [context].
  ///
  /// Runs every operation in [recipe], then merges patch changes targeting the
  /// same file. Exposed so non-CLI front-ends (such as the VS Code extension
  /// host) can reuse the exact collection and merge behavior of the runner.
  Future<List<FileChange>> collectChanges(CodemodContext context) async {
    final changes = <FileChange>[];

    for (final operation in recipe.operations) {
      changes.addAll(await operation.collect(context));
    }

    return _mergePatchChanges(changes);
  }

  Future<void> _execute(
    List<FileChange> changes, {
    required CodemodContext context,
    required bool apply,
  }) async {
    for (final change in changes) {
      print('\n=== ${change.path} ===\n');

      if (apply) {
        await change.apply();
        print('Applied.');
      } else {
        print('DRY RUN - Changes that would be applied:\n');
        print(change.preview());
        print('Run with --apply to apply these changes.');
      }
    }

    if (apply) {
      final result = CodemodRunResult(changes: changes);
      for (final action in recipe.postExecution) {
        await action.run(context, result);
      }
    }

    print(apply ? '\nDone!' : '\nDry run complete.');
  }

  List<FileChange> _mergePatchChanges(List<FileChange> changes) {
    final merged = <FileChange>[];
    final patchByPath = <String, PatchFileChange>{};

    for (final change in changes) {
      if (change is! PatchFileChange) {
        if (patchByPath.containsKey(change.path)) {
          throw StateError(
            'Cannot combine patch and full-file changes for ${change.path}',
          );
        }
        if (merged.any((existing) => existing.path == change.path)) {
          throw StateError('Multiple full-file changes for ${change.path}');
        }
        merged.add(change);
        continue;
      }

      if (merged.any(
        (existing) =>
            existing.path == change.path && existing is! PatchFileChange,
      )) {
        throw StateError(
          'Cannot combine patch and full-file changes for ${change.path}',
        );
      }

      final existing = patchByPath[change.path];
      if (existing == null) {
        patchByPath[change.path] = change;
        merged.add(change);
      } else {
        final patches = [...existing.patches, ...change.patches];
        validateNonOverlappingPatches(patches);
        final mergedChange = PatchFileChange(
          path: existing.path,
          source: existing.source,
          patches: patches,
        );
        patchByPath[change.path] = mergedChange;
        final index = merged.indexOf(existing);
        merged[index] = mergedChange;
      }
    }

    return merged;
  }

  void _printUsage(ArgParser parser) {
    if (recipe.description.isNotEmpty) {
      print('${recipe.description}\n');
    }
    print(
      'Usage: dart run tool/codemods/${recipe.name}/codemod.dart [options]\n',
    );
    print(parser.usage);
  }
}
