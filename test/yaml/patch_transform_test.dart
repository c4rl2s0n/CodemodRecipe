import 'package:codemod_recipe/src/yaml/patch_transform.dart';
import 'package:codemod_recipe/codemod_recipe.dart';
import 'package:test/test.dart';

void main() {
  group('AstPathPatchIdempotency', () {
    const source = '''
class Settings {
  final int count = 0;
}
''';

    test('replace no-ops when whitespace-normalized text matches', () async {
      final transform = AstPathPatchTransform.replace(
        target: const AstPathPatchTarget(
          navigate: [
            NavigateStep(NavigateKind.classDecl, name: 'Settings'),
            NavigateStep(NavigateKind.field, name: 'count'),
          ],
        ),
        text: 'final int count = 0;',
      );

      final patches = await transform.apply(source, CodemodContext(const {}));
      expect(patches, isEmpty);
    });

    test('insert no-ops when field navigate target already exists', () async {
      final transform = AstPathPatchTransform.insert(
        path: const AstPath(
          navigate: [
            NavigateStep(NavigateKind.classDecl, name: 'Settings'),
            NavigateStep(NavigateKind.field, name: 'count'),
          ],
          anchor: Anchor(AnchorKind.docAfter),
        ),
        text: 'final int extra = 1;',
      );

      final patches = await transform.apply(source, CodemodContext(const {}));
      expect(patches, isEmpty);
    });

    test('remove deletes full field declaration span', () async {
      final transform = AstPathPatchTransform.remove(
        target: const AstPathPatchTarget(
          navigate: [
            NavigateStep(NavigateKind.classDecl, name: 'Settings'),
            NavigateStep(NavigateKind.field, name: 'count'),
          ],
        ),
      );

      final patches = await transform.apply(source, CodemodContext(const {}));
      expect(patches, hasLength(1));
      expect(patches.single.replacement, isEmpty);
      expect(patches.single.offset, greaterThan(0));
    });

    test('remove no-ops when field is already absent', () async {
      const emptySource = '''
class Settings {}
''';
      final transform = AstPathPatchTransform.remove(
        target: const AstPathPatchTarget(
          navigate: [
            NavigateStep(NavigateKind.classDecl, name: 'Settings'),
            NavigateStep(NavigateKind.field, name: 'count'),
          ],
        ),
      );

      final patches =
          await transform.apply(emptySource, CodemodContext(const {}));
      expect(patches, isEmpty);
    });
  });
}
