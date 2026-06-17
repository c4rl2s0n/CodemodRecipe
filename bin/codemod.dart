#!/usr/bin/env dart

import 'dart:io';

import 'package:args/args.dart';
import 'package:yaml/yaml.dart';

import 'package:codemod_recipe/codemod_recipe.dart';
import 'package:codemod_recipe/codemod_recipe_vscode.dart';

/// CLI entry point for direct recipe execution.
///
/// Usage:
///   dart run bin/codemod.dart <recipe-file.yaml> [options]
///
/// Example:
///   dart run bin/codemod.dart add_log_line.yaml --file lib/main.dart --className MyClass --methodName myMethod --apply
Future<void> main(List<String> arguments) async {
  // Separate host flags from recipe arguments
  final separation = _separateHostAndRecipeArgs(arguments);
  final hostArgs = separation.hostArgs;
  final recipeArgs = separation.recipeArgs;

  // Parse only host arguments
  final parser = ArgParser()
    ..addOption('map-root', abbr: 'm', help: 'Directory containing map files (default: .codemod/maps)', defaultsTo: '.codemod/maps')
    ..addFlag('apply', abbr: 'a', negatable: false, help: 'Apply changes (default is dry-run)')
    ..addFlag('help', abbr: 'h', negatable: false, help: 'Show usage information');

  late ArgResults results;

  try {
    results = parser.parse(hostArgs);
  } on FormatException catch (error) {
    stderr.writeln('Error: ${error.message}');
    _printUsage();
    exit(1);
  }

  // If --help is a host flag and no recipe is specified, show host-level help
  if (results['help'] == true && recipeArgs.isEmpty) {
    _printUsage();
    exit(0);
  }

  // Get the recipe file path (should be the first recipe argument)
  if (recipeArgs.isEmpty) {
    stderr.writeln('Error: Recipe file path is required');
    _printUsage();
    exit(1);
  }

  final recipePath = recipeArgs.first;

  try {
    // Load the recipe
    final recipe = await _loadRecipe(recipePath, results['map-root'] as String);
    if (recipe == null) {
      exit(1); // Error already printed
    }

    // Build the arguments list for the runner
    // Start with recipe-specific args (excluding the recipe path)
    final runnerArgs = recipeArgs.skip(1).toList();

    // Add host flags that should be passed to the runner
    // (--help will be passed through to show recipe-specific help)
    if (results['help'] == true) {
      runnerArgs.insert(0, '--help');
    }
    if (results['apply'] == true) {
      runnerArgs.add('--apply');
    }

    // Run the recipe
    final config = HostConfig(
      workspaceRoot: Directory.current.absolute.path,
      codemodRoot: results['map-root'] as String,
    );

    final runner = CodemodRunner(
      recipe,
      preferences: config.preferences,
    );

    await runner.run(runnerArgs);

  } on FormatException catch (e) {
    stderr.writeln('Invalid YAML: ${e.message}');
    exit(1);
  } on FileSystemException catch (e) {
    stderr.writeln('File error: ${e.message}');
    exit(1);
  } catch (e) {
    stderr.writeln('Error: $e');
    exit(1);
  }
}

/// Separates host-level flags from recipe-specific arguments.
/// Host flags are: --help, -h, --map-root, -m, --apply, -a
/// All other arguments are treated as recipe-specific and passed through.
_SeparatedArgs _separateHostAndRecipeArgs(List<String> arguments) {
  final hostArgs = <String>[];
  final recipeArgs = <String>[];

  var i = 0;
  while (i < arguments.length) {
    final arg = arguments[i];

    // Check if it's a host flag
    if (arg == '--help' || arg == '-h') {
      hostArgs.add(arg);
      i++;
    } else if (arg == '--map-root' || arg == '-m') {
      hostArgs.add(arg);
      if (i + 1 < arguments.length) {
        hostArgs.add(arguments[i + 1]);
        i += 2;
      } else {
        i++;
      }
    } else if (arg == '--apply' || arg == '-a') {
      hostArgs.add(arg);
      i++;
    } else {
      // This is a recipe argument (recipe path or recipe-specific flag)
      recipeArgs.add(arg);
      i++;
    }
  }

  return _SeparatedArgs(hostArgs, recipeArgs);
}

class _SeparatedArgs {
  final List<String> hostArgs;
  final List<String> recipeArgs;

  _SeparatedArgs(this.hostArgs, this.recipeArgs);
}

/// Loads a single YAML recipe from a file path.
Future<CodemodRecipe?> _loadRecipe(String path, String mapRoot) async {
  final file = File(path);
  if (!await file.exists()) {
    stderr.writeln('Recipe file not found: $path');
    return null;
  }

  try {
    final content = await file.readAsString();
    final yaml = loadYaml(content);
    
    if (yaml is! YamlMap) {
      stderr.writeln('Invalid YAML: root must be a map in $path');
      return null;
    }

    final filePath = file.absolute.path;
    final workspaceRoot = Directory.current.absolute.path;
    final relativePath = _relativePath(workspaceRoot, filePath);

    // Parse the recipe definition
    final definition = parseYamlRecipeFile(relativePath, content);

    // Load maps from the map root directory
    final mapsById = await _loadMaps(mapRoot);

    // Create a minimal compiler context
    final compiler = YamlRecipeCompiler(
      config: HostConfig(
        workspaceRoot: workspaceRoot,
        codemodRoot: mapRoot,
      ),
      definitionsById: {definition.id: definition},
      dartRecipes: const {},
      mapsById: mapsById,
    );

    final compiled = compiler.compile(definition);

    if (compiled.recipe == null) {
      stderr.writeln('Failed to compile recipe $path:');
      for (final diagnostic in compiled.diagnostics) {
        stderr.writeln('  [${diagnostic.code}] ${diagnostic.message}');
      }
      return null;
    }

    return compiled.recipe!;
  } catch (e) {
    stderr.writeln('Error loading recipe $path: $e');
    return null;
  }
}

/// Loads map files from the specified directory.
Future<Map<String, Map<String, String>>> _loadMaps(String mapRoot) async {
  final mapsById = <String, Map<String, String>>{};
  
  final mapDir = Directory(mapRoot);
  if (!await mapDir.exists()) {
    return mapsById;
  }

  try {
    await for (final entity in mapDir.list(recursive: false)) {
      if (entity is! File) continue;
      
      final path = entity.path;
      if (!path.endsWith('.yaml') && !path.endsWith('.yml')) continue;

      final content = await File(path).readAsString();
      final yaml = loadYaml(content);
      
      if (yaml is! YamlMap) continue;

      final id = yaml['id']?.toString();
      if (id == null || id.isEmpty) continue;

      // Check if this is a map (has entries)
      if (yaml.containsKey('entries') && yaml['entries'] is YamlMap) {
        final entries = <String, String>{};
        (yaml['entries'] as YamlMap).forEach((key, value) {
          entries[key.toString()] = value?.toString() ?? '';
        });
        mapsById[id] = entries;
      }
    }
  } catch (e) {
    // Ignore errors loading maps - they may not exist
  }

  return mapsById;
}

/// Computes the relative path from workspace root to file.
String _relativePath(String workspaceRoot, String absolutePath) {
  final root = Directory(workspaceRoot).absolute.path.replaceAll('\\', '/');
  final file = File(absolutePath).absolute.path.replaceAll('\\', '/');
  if (file.startsWith('$root/')) {
    return file.substring(root.length + 1);
  }
  return file;
}

/// Prints usage information for the CLI.
void _printUsage() {
  print('''
Usage: dart run bin/codemod.dart <recipe-file.yaml> [options]

Arguments:
  <recipe-file.yaml>    Path to the YAML recipe file

Host Options:
  -m, --map-root       Directory containing map files (default: .codemod/maps)
  -a, --apply          Apply changes (default: dry-run)
  -h, --help           Show this help message

Recipe Options:
  Recipe-specific arguments (like --file, --className, --methodName, etc.) are passed through to the recipe.
  Run with a recipe file and --help to see its specific arguments.

Examples:
  dart run bin/codemod.dart add_log_line.yaml --file lib/main.dart --className MyClass --methodName myMethod
  dart run bin/codemod.dart add_log_line.yaml --file lib/main.dart --className MyClass --methodName myMethod --apply
  dart run bin/codemod.dart add_counter_field.yaml --file lib/main.dart --className MyClass --apply
  dart run bin/codemod.dart add_constructor_param.yaml --file lib/main.dart --className MyClass --param myParam
  dart run bin/codemod.dart add_log_line.yaml --help
''');
}
