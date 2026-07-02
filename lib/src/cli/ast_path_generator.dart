import 'dart:io';
import 'dart:convert';

import '../dart_codegen/ast_helpers/ast_focus.dart';
import '../ast_path/yaml_generator.dart';
import '../ast_path/model.dart';

// Aliases for global stdout/stderr to avoid naming conflicts with parameters
final dartStdout = stdout;
final dartStderr = stderr;

/// CLI command for generating AST paths from source files at specific offsets.
class AstPathGenerator {
  /// Generates an AST path for the node at the given offset in the file.
  ///
  /// Args:
  ///   filePath: Path to the Dart source file
  ///   offset: Byte offset in the file (0-indexed)
  ///   outputFormat: Format for output ('text', 'yaml', or 'json')
  ///   recipeId: ID to use for generated YAML recipe
  ///   stdout: Optional output stream for testing (defaults to global stdout)
  ///   stderr: Optional error stream for testing (defaults to global stderr)
  static Future<int> generateFromFile(
    String filePath,
    int offset, {
    String outputFormat = 'text',
    String recipeId = 'generated_recipe',
    IOSink? stdout,
    IOSink? stderr,
  }) async {
    try {
      // Read source file
      final source = await File(filePath).readAsString();

      // Parse and find node at offset
      final focus = AstFocus.parse(source, path: filePath);
      final path = focus.generatePathAtOffset(offset);

      if (path != null) {
        _outputResult(
          path,
          offset,
          outputFormat,
          recipeId,
          filePath,
          stdout: stdout ?? dartStdout,
          stderr: stderr ?? dartStderr,
        );
        return 0;
      } else {
        (stderr ?? dartStderr).writeln(
          '❌ No AST node found at offset $offset in $filePath',
        );
        return 1;
      }
    } on FileSystemException catch (e) {
      (stderr ?? dartStderr).writeln('❌ Error reading file: ${e.message}');
      return 1;
    } catch (e) {
      (stderr ?? dartStderr).writeln('❌ Error generating AST path: $e');
      return 1;
    }
  }

  static void _outputResult(
    AstPath path,
    int offset,
    String format,
    String recipeId,
    String filePath, {
    required IOSink stdout,
    required IOSink stderr,
  }) {
    final output = stdout;
    switch (format.toLowerCase()) {
      case 'yaml':
        final yaml = AstPathYamlGenerator.generateYaml(
          path,
          recipeId: recipeId,
        );
        output.writeln(yaml);
        break;
      case 'json':
        // Simple JSON representation
        final stepsJson = path.navigate
            .map(
              (step) => {
                'kind': step.kind?.name,
                'name': step.name,
                'match': step.match,
              },
            )
            .toList();

        final jsonOutput = jsonEncode({
          'file': filePath,
          'offset': offset,
          'path': {'navigate': stepsJson, 'anchor': path.anchor.toString()},
        });
        output.writeln(jsonOutput);
        break;
      case 'compact':
        // Compact localization string
        final compact = AstPathYamlGenerator.generateCompactLocalization(path);
        output.writeln(compact);
        break;
      case 'path-only':
        // Just the AST path in YAML format
        final pathYaml = AstPathYamlGenerator.generateAstPathYaml(path);
        output.writeln(pathYaml);
        break;
      case 'text':
      default:
        output.writeln('🎯 AST Path for offset $offset in $filePath:');
        output.writeln('');
        output.writeln('Navigate steps:');
        for (final step in path.navigate) {
          output.writeln(
            '  • ${step.kind?.name ?? 'inferred'}: ${step.name}${step.match != null ? ' (match: ${step.match})' : ''}',
          );
        }
        output.writeln('');
        output.writeln('Anchor: ${path.anchor}');
        output.writeln('');
        output.writeln('📋 Compact localization:');
        output.writeln(AstPathYamlGenerator.generateCompactLocalization(path));
        output.writeln('');
        output.writeln('📋 YAML DSL snippet:');
        output.writeln('at:');
        for (final step in path.navigate) {
          final kind = step.kind?.name ?? 'inferred';
          output.writeln(
            '  - $kind: "${step.name}"${step.match != null ? ' (match: "${step.match}")' : ''}',
          );
        }
        output.writeln('anchor: ${path.anchor}');
    }
  }

  /// Entry point for CLI usage.
  static Future<int> runCli(
    List<String> args, {
    IOSink? stdout,
    IOSink? stderr,
  }) async {
    final out = stdout ?? dartStdout;
    final err = stderr ?? dartStderr;

    for (final arg in args) {
      if (arg == '--help' || arg == '-h') {
        _printUsage(out);
        return 0;
      }
    }

    if (args.length < 2) {
      _printUsage(out);
      return 1;
    }

    final filePath = args[0];
    final offset = int.tryParse(args[1]);

    if (offset == null) {
      err.writeln('❌ Invalid offset: ${args[1]}');
      _printUsage(out);
      return 1;
    }

    // Parse optional arguments
    String format = 'text';
    String recipeId = 'generated_recipe';

    for (int i = 2; i < args.length; i++) {
      if (args[i] == '--format' && i + 1 < args.length) {
        format = args[++i];
      } else if (args[i] == '--recipe-id' && i + 1 < args.length) {
        recipeId = args[++i];
      }
    }

    return generateFromFile(
      filePath,
      offset,
      outputFormat: format,
      recipeId: recipeId,
      stdout: out,
      stderr: err,
    );
  }

  static Future<void> main(List<String> args) async {
    exit(await runCli(args));
  }

  static void _printUsage([IOSink? stdout]) {
    (stdout ?? dartStdout).writeln('''
🚀 AST Path Generator - Generate AST localization DSL code from source offsets

Usage: dart run ast_path_generator.dart <file> <offset> [options]

Arguments:
  <file>      Path to the Dart source file
  <offset>    Byte offset in the file (0-indexed)

Options:
  --format    Output format: text, yaml, json, compact, or path-only (default: text)
              text      - Detailed text output with all formats
              yaml      - Complete YAML recipe
              json      - JSON representation
              compact   - Compact localization string (e.g., "class:Name > method:methodName @ anchor")
              path-only - Just the AST path in YAML format
  --recipe-id  ID to use for generated YAML recipe (default: generated_recipe)
  --help, -h   Show this help message

Examples:
  # Generate text output (default)
  dart run ast_path_generator.dart lib/my_file.dart 123
  
  # Generate YAML recipe
  dart run ast_path_generator.dart lib/my_file.dart 123 --format yaml
  
  # Generate compact localization string
  dart run ast_path_generator.dart lib/my_file.dart 123 --format compact
  
  # Generate just the AST path
  dart run ast_path_generator.dart lib/my_file.dart 123 --format path-only
  
  # Generate JSON output
  dart run ast_path_generator.dart lib/my_file.dart 123 --format json --recipe-id my_recipe
''');
  }
}
