import 'dart:io';

import '../dart_codegen/ast_helpers/ast_focus.dart';
import '../ast_path/node_finder.dart';
import '../ast_path/yaml_generator.dart';
import '../ast_path/model.dart';

/// CLI command for generating AST paths from source files at specific offsets.
class AstPathGenerator {
  /// Generates an AST path for the node at the given offset in the file.
  ///
  /// Args:
  ///   filePath: Path to the Dart source file
  ///   offset: Byte offset in the file (0-indexed)
  ///   outputFormat: Format for output ('text', 'yaml', or 'json')
  ///   recipeId: ID to use for generated YAML recipe
  static Future<void> generateFromFile(
    String filePath, 
    int offset, {
    String outputFormat = 'text',
    String recipeId = 'generated_recipe',
  }) async {
    try {
      // Read source file
      final source = await File(filePath).readAsString();
      
      // Parse and find node at offset
      final focus = AstFocus.parse(source, path: filePath);
      final path = focus.generatePathAtOffset(offset);
      
      if (path != null) {
        _outputResult(path, offset, outputFormat, recipeId, filePath);
      } else {
        stderr.writeln('❌ No AST node found at offset $offset in $filePath');
        exit(1);
      }
    } on FileSystemException catch (e) {
      stderr.writeln('❌ Error reading file: ${e.message}');
      exit(1);
    } catch (e) {
      stderr.writeln('❌ Error generating AST path: $e');
      exit(1);
    }
  }
  
  static void _outputResult(
    AstPath path, 
    int offset, 
    String format, 
    String recipeId, 
    String filePath,
  ) {
    switch (format.toLowerCase()) {
      case 'yaml':
        final yaml = AstPathYamlGenerator.generateYaml(path, recipeId: recipeId);
        print(yaml);
        break;
      case 'json':
        // Simple JSON representation
        final stepsJson = path.navigate.map((step) => {
          'kind': step.kind?.name,
          'name': step.name,
          'match': step.match,
        }).toList();
        
        print({
          'file': filePath,
          'offset': offset,
          'path': {
            'navigate': stepsJson,
            'anchor': path.anchor.toString(),
          }
        });
        break;
      case 'compact':
        // Compact localization string
        final compact = AstPathYamlGenerator.generateCompactLocalization(path);
        print(compact);
        break;
      case 'path-only':
        // Just the AST path in YAML format
        final pathYaml = AstPathYamlGenerator.generateAstPathYaml(path);
        print(pathYaml);
        break;
      case 'text':
      default:
        print('🎯 AST Path for offset $offset in $filePath:');
        print('');
        print('Navigate steps:');
        for (final step in path.navigate) {
          print('  • ${step.kind?.name ?? 'inferred'}: ${step.name}${step.match != null ? ' (match: ${step.match})' : ''}');
        }
        print('');
        print('Anchor: ${path.anchor}');
        print('');
        print('📋 Compact localization:');
        print(AstPathYamlGenerator.generateCompactLocalization(path));
        print('');
        print('📋 YAML DSL snippet:');
        print('at:');
        for (final step in path.navigate) {
          final kind = step.kind?.name ?? 'inferred';
          print('  - $kind: "${step.name}"${step.match != null ? ' (match: "${step.match}")' : ''}');
        }
        print('anchor: ${path.anchor}');
    }
  }
  
  /// Entry point for CLI usage.
  static Future<void> main(List<String> args) async {
    if (args.length < 2) {
      _printUsage();
      exit(1);
    }
    
    final filePath = args[0];
    final offset = int.tryParse(args[1]);
    
    if (offset == null) {
      stderr.writeln('❌ Invalid offset: ${args[1]}');
      _printUsage();
      exit(1);
    }
    
    // Parse optional arguments
    String format = 'text';
    String recipeId = 'generated_recipe';
    
    for (int i = 2; i < args.length; i++) {
      if (args[i] == '--format' && i + 1 < args.length) {
        format = args[++i];
      } else if (args[i] == '--recipe-id' && i + 1 < args.length) {
        recipeId = args[++i];
      } else if (args[i] == '--help' || args[i] == '-h') {
        _printUsage();
        exit(0);
      }
    }
    
    await generateFromFile(filePath, offset, 
      outputFormat: format, 
      recipeId: recipeId
    );
  }
  
  static void _printUsage() {
    print('''
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