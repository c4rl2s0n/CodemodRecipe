import 'model.dart';

/// Generates YAML DSL code from AST paths for use in codemod recipes.
class AstPathYamlGenerator {
  /// Generates a complete YAML recipe with the given AST path.
  ///
  /// The generated recipe includes proper structure with dslVersion, id, name,
  /// and a basic edit step using the provided path.
  static String generateYaml(
    AstPath path, {
    String recipeId = 'generated_recipe',
    String recipeName = 'Generated Recipe',
    String description = 'Recipe generated from AST path',
  }) {
    final buffer = StringBuffer();
    
    buffer.writeln('dslVersion: 1');
    buffer.writeln('id: $recipeId');
    buffer.writeln('name: "$recipeName"');
    buffer.writeln('description: "$description"');
    buffer.writeln('');
    buffer.writeln('args:');
    buffer.writeln('  - name: file');
    buffer.writeln('    required: true');
    buffer.writeln('    inputKind: file');
    buffer.writeln('    help: "The file to modify"');
    buffer.writeln('');
    buffer.writeln('steps:');
    buffer.writeln('  - edit:');
    buffer.writeln('      path: "{{file}}"');
    buffer.writeln('      steps:');
    buffer.writeln('        - insert:');
    buffer.writeln('            at:');
    
    // Generate navigation steps
    for (final step in path.navigate) {
      _writeNavigationStep(buffer, step);
    }
    
    buffer.writeln('            anchor: ${path.anchor}');
    buffer.writeln('            text: "// TODO: Add your code here"');
    buffer.writeln('');
    buffer.writeln('postExecution:');
    buffer.writeln('  - run: dart format .');
    
    return buffer.toString();
  }
  
  /// Writes a single navigation step in YAML format.
  static void _writeNavigationStep(StringBuffer buffer, NavigateStep step) {
    final indent = '              '; // 14 spaces for proper YAML indentation
    
    if (step.kind != null) {
      // Explicit navigation kind
      buffer.write('$indent- ${step.kind!.name}: "${step.name}"');
    } else {
      // Type-inferred navigation
      buffer.write('$indent- inferred: "${step.name}"');
    }
    
    // Add match filter if present
    if (step.match != null) {
      buffer.write(' # match: "${step.match}"');
    }
    
    buffer.writeln();
  }
  
  /// Generates just the AST path portion for embedding in existing recipes.
  static String generatePathSnippet(AstPath path) {
    final buffer = StringBuffer();
    
    buffer.writeln('at:');
    for (final step in path.navigate) {
      if (step.kind != null) {
        buffer.writeln('  - ${step.kind!.name}: "${step.name}"${step.match != null ? ' # match: "${step.match}"' : ''}');
      } else {
        buffer.writeln('  - inferred: "${step.name}"${step.match != null ? ' # match: "${step.match}"' : ''}');
      }
    }
    buffer.writeln('anchor: ${path.anchor}');
    
    return buffer.toString();
  }
  
  /// Generates a minimal recipe with just the path information.
  static String generateMinimalYaml(AstPath path, String recipeId) {
    return '''dslVersion: 1
id: $recipeId
steps:
  - edit:
      path: "{{file}}"
      steps:
        - insert:
            at:
${_generateStepsYaml(path.navigate)}
            anchor: ${path.anchor}
            text: "// Insert code here"
''';
  }
  
  /// Generates a compact localization string for easy embedding.
  /// 
  /// Format: "class:Name > method:methodName @ anchorType"
  static String generateCompactLocalization(AstPath path) {
    final steps = path.navigate.map((step) {
      if (step.kind != null) {
        return '${step.kind!.name}:${step.name}';
      } else {
        return 'inferred:${step.name}';
      }
    }).join(' > ');
    
    return '$steps @ ${path.anchor}';
  }
  
  /// Generates just the AST path portion in YAML format.
  static String generateAstPathYaml(AstPath path) {
    return '''at:
${_generateStepsYaml(path.navigate)}
anchor: ${path.anchor}''';
  }
  
  static String _generateStepsYaml(List<NavigateStep> steps) {
    final buffer = StringBuffer();
    for (final step in steps) {
      if (step.kind != null) {
        buffer.writeln('              - ${step.kind!.name}: "${step.name}"');
      } else {
        buffer.writeln('              - inferred: "${step.name}"');
      }
    }
    return buffer.toString();
  }
}