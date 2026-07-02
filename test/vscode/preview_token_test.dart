import 'package:codemod_recipe/codemod_recipe_vscode.dart';
import 'package:test/test.dart';

void main() {
  group('computePreviewToken', () {
    test('is stable for key order differences', () {
      final a = computePreviewToken({
        'args': {'b': '2', 'a': '1'},
        'recipe': 'add_method',
      });
      final b = computePreviewToken({
        'recipe': 'add_method',
        'args': {'a': '1', 'b': '2'},
      });
      expect(a, b);
    });

    test('changes when snapshot content changes', () {
      final before = computePreviewToken({
        'recipe': 'x',
        'snapshots': {
          'lib/a.dart': {'exists': true, 'modifiedMs': 1, 'size': 10},
        },
      });
      final after = computePreviewToken({
        'recipe': 'x',
        'snapshots': {
          'lib/a.dart': {'exists': true, 'modifiedMs': 2, 'size': 10},
        },
      });
      expect(before, isNot(after));
    });
  });
}
