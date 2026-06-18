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
  // Initialize logging
  logger.runnerInfo('Starting codemod execution with arguments: ${arguments.join(' ')}');
  // Separate host flags from recipe arguments
  final separation = HostArgsParser.separateHostAndRecipeArgs(arguments);
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
    logger.runnerError('Argument parsing failed: ${error.message}');
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
    logger.runnerError('No recipe file path provided');
    stderr.writeln('Error: Recipe file path is required');
    _printUsage();
    exit(1);
  }

  final recipePath = recipeArgs.first;
  logger.runnerInfo('Loading recipe from: $recipePath');

  try {
    // Load the recipe
    final recipe = await _loadRecipe(recipePath, results['map-root'] as String);
    if (recipe == null) {
      logger.runnerError('Failed to load recipe from $recipePath');
      exit(1); // Error already printed
    }
    
    logger.runnerInfo('Successfully loaded recipe: ${recipe.name}');

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
    logger.runnerInfo('Executing recipe with arguments: ${runnerArgs.join(' ')}');
    final config = HostConfig(
      workspaceRoot: FileUtils.getCurrentDirectory(),
      codemodRoot: results['map-root'] as String,
    );

    final runner = CodemodRunner(
      recipe,
      preferences: config.preferences,
    );

    await runner.run(runnerArgs);
    logger.runnerInfo('Recipe execution completed successfully');

  } on FormatException catch (e) {
    logger.runnerError('YAML parsing failed: ${e.message}', e);
    stderr.writeln('Invalid YAML: ${e.message}');
    exit(1);
  } on FileSystemException catch (e) {
    logger.runnerError('File system error: ${e.message}', e);
    stderr.writeln('File error: ${e.message}');
    exit(1);
  } catch (e) {
    logger.runnerError('Unexpected error during recipe execution', e);
    stderr.writeln('Error: $e');
    exit(1);
  }
}

// The SeparatedArgs class is now provided by HostArgsParser in lib/src/args.dart

/// Loads a single YAML recipe from a file path.
Future<CodemodRecipe?> _loadRecipe(String path, String mapRoot) async {
  logger.fileInfo('Checking recipe file existence: $path');
  final file = File(path);
  if (!await file.exists()) {
    logger.fileError('Recipe file not found: $path');
    stderr.writeln('Recipe file not found: $path');
    return null;
  }

  try {
    logger.fileInfo('Reading recipe file content');
    final content = await file.readAsString();
    logger.yamlInfo('Parsing YAML content');
    final yaml = loadYaml(content);
    
    if (yaml is! YamlMap) {
      logger.yamlError('Invalid YAML structure: root must be a map in $path');
      stderr.writeln('Invalid YAML: root must be a map in $path');
      return null;
    }

    final filePath = file.absolute.path;
    final workspaceRoot = FileUtils.getCurrentDirectory();
    final relativePath = FileUtils.relativePath(workspaceRoot, filePath);
    logger.yamlInfo('Parsing recipe definition from: $relativePath');

    // Parse the recipe definition
    final definition = parseYamlRecipeFile(relativePath, content);

    // Load maps from the map root directory
    logger.fileInfo('Loading maps from: $mapRoot');
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

    logger.yamlInfo('Compiling recipe definition');
    final compiled = compiler.compile(definition);

    if (compiled.recipe == null) {
      logger.yamlError('Recipe compilation failed for $path');
      stderr.writeln('Failed to compile recipe $path:');
      for (final diagnostic in compiled.diagnostics) {
        logger.yamlError('Diagnostic [${diagnostic.code}]: ${diagnostic.message}');
        stderr.writeln('  [${diagnostic.code}] ${diagnostic.message}');
      }
      return null;
    }
    
    logger.yamlInfo('Successfully compiled recipe: ${compiled.recipe!.name}');
    return compiled.recipe!;
  } catch (e) {
    logger.fileError('Error loading recipe $path', e);
    stderr.writeln('Error loading recipe $path: $e');
    return null;
  }
}

/// Loads map files from the specified directory.
Future<Map<String, Map<String, String>>> _loadMaps(String mapRoot) async {
  logger.fileInfo('Loading maps from directory: $mapRoot');
  final mapsById = <String, Map<String, String>>{};
  
  final mapDir = Directory(mapRoot);
  if (!await mapDir.exists()) {
    logger.fileInfo('Map directory does not exist, returning empty map collection');
    return mapsById;
  }

  try {
    await for (final entity in mapDir.list(recursive: false)) {
      if (entity is! File) continue;
      
      final filePath = entity.path;
      if (!FileUtils.hasExtension(filePath, ['.yaml', '.yml'])) continue;
      
      logger.fileInfo('Processing map file: $filePath');
      final content = await File(filePath).readAsString();
      final yaml = loadYaml(content);
      
      if (yaml is! YamlMap) {
        logger.fileWarning('Invalid map file structure (not a YAML map): $filePath');
        continue;
      }

      final id = yaml['id']?.toString();
      if (id == null || id.isEmpty) {
        logger.fileWarning('Map file missing ID: $filePath');
        continue;
      }

      // Check if this is a map (has entries)
      if (yaml.containsKey('entries') && yaml['entries'] is YamlMap) {
        final entries = <String, String>{};
        (yaml['entries'] as YamlMap).forEach((key, value) {
          entries[key.toString()] = value?.toString() ?? '';
        });
        mapsById[id] = entries;
        logger.fileInfo('Loaded map with ID: $id (${entries.length} entries)');
      }
    }
  } catch (e) {
    logger.fileError('Error loading maps from $mapRoot', e);
    // Ignore errors loading maps - they may not exist
  }

  logger.fileInfo('Finished loading ${mapsById.length} maps');
  return mapsById;
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
