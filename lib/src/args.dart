// Copyright (c) 2026, the Dart project authors. Please see the AUTHORS file
// for details. All rights reserved. Use of this source code is governed by
// a BSD-style license that can be found in the LICENSE file.

import 'dart:io';

import 'package:args/args.dart';

import 'yaml/host_config.dart';

// Local constants for argument parsing
const String _kHelpFlag = 'help';
const String _kHelpAbbr = 'h';
const String _kApplyFlag = 'apply';
const String _kApplyAbbr = 'a';
const String _kMapRootFlag = 'map-root';
const String _kMapRootAbbr = 'm';
const String _kValidateFlag = 'validate';
const String _kStdioServerFlag = 'stdio-server';
const String _kCommandField = 'command';
const String _kDefaultMapRoot = '.codemod/maps';

/// Utility class for parsing and handling command-line arguments.
///
/// This class provides reusable argument parsing functionality for both
/// CLI and VS Code host entry points.
class HostArgsParser {
  /// Creates an argument parser with standard host options.
  static ArgParser buildArgParser() {
    return ArgParser()
      ..addFlag(
        _kValidateFlag,
        negatable: false,
        help: 'Validate YAML recipes and exit',
      )
      ..addFlag(
        _kStdioServerFlag,
        negatable: false,
        help: 'Run as a persistent stdio server for VS Code extension',
      )
      ..addOption(
        _kMapRootFlag,
        abbr: _kMapRootAbbr,
        defaultsTo: _kDefaultMapRoot,
        help: 'Directory containing map files (default: $_kDefaultMapRoot)',
      );
  }

  /// Parses command-line arguments and creates a HostConfig.
  ///
  /// Returns a tuple of (ArgResults, HostConfig) or null if parsing fails.
  static ({ArgResults results, HostConfig config})? parseAndCreateConfig(
    List<String> arguments, {
    String? workspaceRoot,
    String? codemodRoot,
  }) {
    final parser = buildArgParser();
    
    try {
      final results = parser.parse(arguments);
      final config = HostConfig(
        workspaceRoot: workspaceRoot ?? Directory.current.absolute.path,
        codemodRoot: codemodRoot ?? results['map-root'] as String? ?? '.codemod/maps',
      );
      return (results: results, config: config);
    } on FormatException catch (error) {
      stderr.writeln('Error: ${error.message}');
      return null;
    }
  }

  /// Separates host-level flags from recipe-specific arguments.
  ///
  /// Host flags are: --help, -h, --map-root, -m, --apply, -a, --validate, --stdio-server
  /// All other arguments are treated as recipe-specific and passed through.
  static SeparatedArgs separateHostAndRecipeArgs(List<String> arguments) {
    final hostArgs = <String>[];
    final recipeArgs = <String>[];

    var i = 0;
    while (i < arguments.length) {
      final arg = arguments[i];

      // Check if it's a host flag
      if (arg == '--$_kHelpFlag' || arg == '-$_kHelpAbbr') {
        hostArgs.add(arg);
        i++;
      } else if (arg == '--$_kMapRootFlag' || arg == '-$_kMapRootAbbr') {
        hostArgs.add(arg);
        if (i + 1 < arguments.length) {
          hostArgs.add(arguments[i + 1]);
          i += 2;
        } else {
          i++;
        }
      } else if (arg == '--$_kApplyFlag' || arg == '-$_kApplyAbbr') {
        hostArgs.add(arg);
        i++;
      } else if (arg == '--$_kValidateFlag') {
        hostArgs.add(arg);
        i++;
      } else if (arg == '--$_kStdioServerFlag') {
        hostArgs.add(arg);
        i++;
      } else {
        // This is a recipe argument (recipe path or recipe-specific flag)
        recipeArgs.add(arg);
        i++;
      }
    }

    return SeparatedArgs(hostArgs: hostArgs, recipeArgs: recipeArgs);
  }

  /// Checks if the given arguments look like a JSON command (for VS Code).
  static bool looksLikeJsonCommand(List<String> arguments) {
    if (arguments.isEmpty) return false;
    final first = arguments.first.trim();
    return first.startsWith('{') && first.contains('"$_kCommandField"');
  }

  /// Prints usage information for the CLI.
  static void printUsage(ArgParser parser, {String? programName}) {
    final name = programName ?? 'codemod_host';
    stderr.writeln('Usage: $name [host options]');
    stderr.writeln('');
    stderr.writeln('VS Code extension host options:');
    stderr.writeln(parser.usage);
    stderr.writeln('');
    stderr.writeln('For CLI usage, use: dart run bin/codemod.dart <recipe.yaml> [args]');
  }
}

/// Result of separating host and recipe arguments.
class SeparatedArgs {
  /// Host-level arguments (flags that apply to the host).
  final List<String> hostArgs;

  /// Recipe-specific arguments (recipe path and its flags).
  final List<String> recipeArgs;

  /// Creates a separated arguments result.
  const SeparatedArgs({required this.hostArgs, required this.recipeArgs});
}
