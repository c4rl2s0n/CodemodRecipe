import 'package:test/test.dart';

import '../../lib/src/ast_path/yaml_generator.dart';
import '../../lib/src/ast_path/model.dart';

void main() {
  group('AstPathYamlGenerator', () {
    
    final testPath = AstPath(
      navigate: [
        NavigateStep(NavigateKind.classDecl, name: 'MyClass'),
        NavigateStep(NavigateKind.method, name: 'myMethod'),
      ],
      anchor: const Anchor(AnchorKind.stmtLast),
    );
    
    group('generateYaml', () {
      test('generates valid YAML structure', () {
        final yaml = AstPathYamlGenerator.generateYaml(testPath);
        
        expect(yaml, contains('dslVersion: 1'));
        expect(yaml, contains('id: generated_recipe'));
        expect(yaml, contains('name: "Generated Recipe"'));
        expect(yaml, contains('description: "Recipe generated from AST path"'));
        expect(yaml, contains('inputKind: file'));
        expect(yaml, contains('edit:'));
        expect(yaml, contains('path: "{{file}}"'));
        expect(yaml, contains('insert:'));
        expect(yaml, contains('at:'));
        expect(yaml, contains('classDecl: "MyClass"'));
        expect(yaml, contains('method: "myMethod"'));
        expect(yaml, contains('anchor: stmtLast'));
        expect(yaml, contains('postExecution:'));
        expect(yaml, contains('dart format .'));
      });

      test('uses custom recipe ID and name', () {
        final yaml = AstPathYamlGenerator.generateYaml(
          testPath,
          recipeId: 'my_custom_recipe',
          recipeName: 'My Custom Recipe',
          description: 'Custom description',
        );
        
        expect(yaml, contains('id: my_custom_recipe'));
        expect(yaml, contains('name: "My Custom Recipe"'));
        expect(yaml, contains('description: "Custom description"'));
      });
    });

    group('generatePathSnippet', () {
      test('generates just the path portion', () {
        final snippet = AstPathYamlGenerator.generatePathSnippet(testPath);
        
        expect(snippet, contains('at:'));
        expect(snippet, contains('classDecl: "MyClass"'));
        expect(snippet, contains('method: "myMethod"'));
        expect(snippet, contains('anchor: stmtLast'));
        expect(snippet, isNot(contains('dslVersion:')));
        expect(snippet, isNot(contains('steps:')));
      });
    });

    group('generateMinimalYaml', () {
      test('generates minimal recipe structure', () {
        final yaml = AstPathYamlGenerator.generateMinimalYaml(
          testPath,
          'minimal_recipe'
        );
        
        expect(yaml, contains('dslVersion: 1'));
        expect(yaml, contains('id: minimal_recipe'));
        expect(yaml, contains('edit:'));
        expect(yaml, contains('insert:'));
        expect(yaml, contains('at:'));
        expect(yaml, contains('classDecl: "MyClass"'));
        expect(yaml, contains('method: "myMethod"'));
        expect(yaml, contains('anchor: stmtLast'));
        // Should not have full args section
        expect(yaml, isNot(contains('inputKind: file')));
      });
    });

    group('edge cases', () {
      test('handles inferred navigation steps', () {
        final pathWithInferred = AstPath(
          navigate: [
            NavigateStep(null, name: 'someNode'), // inferred
          ],
          anchor: const Anchor(AnchorKind.bodyEnd),
        );
        
        final yaml = AstPathYamlGenerator.generateYaml(pathWithInferred);
        expect(yaml, contains('inferred: "someNode"'));
      });

      test('handles steps with match filters', () {
        final pathWithMatch = AstPath(
          navigate: [
            NavigateStep(NavigateKind.classDecl, name: 'MyClass', match: 'specific'),
          ],
          anchor: const Anchor(AnchorKind.memberLast),
        );
        
        final yaml = AstPathYamlGenerator.generateYaml(pathWithMatch);
        expect(yaml, contains('classDecl: "MyClass"'));
        expect(yaml, contains('# match: "specific"'));
      });

      test('handles different anchor types', () {
        const anchors = [
          AnchorKind.bodyStart,
          AnchorKind.bodyEnd,
          AnchorKind.stmtLast,
          AnchorKind.memberLast,
          AnchorKind.paramLast,
          AnchorKind.argLast,
        ];
        
        for (final anchorKind in anchors) {
          final path = AstPath(
            navigate: [],
            anchor: Anchor(anchorKind),
          );
          
          final yaml = AstPathYamlGenerator.generateYaml(path);
          expect(yaml, contains('anchor: ${anchorKind.name}'));
        }
      });
    });
  });
}