import 'dart:io';

import 'package:codemod_recipe/codemod_recipe.dart';
import 'package:test/test.dart';

import 'package:codemod_recipe/src/yaml/patch_transform.dart';

void main() {
  late String settingsSource;

  setUp(() {
    settingsSource = File(
      'test/fixtures/ast_paths/settings.dart',
    ).readAsStringSync();
  });

  group('declarationSpan', () {
    test('includes leading doc comment and trailing semicolon', () {
      final interpreter = AstPathInterpreter();
      final countField = interpreter.navigateTo(settingsSource, [
        const NavigateStep(NavigateKind.classDecl, name: 'Settings'),
        const NavigateStep(NavigateKind.field, name: 'count'),
      ]);

      final span = declarationSpan(settingsSource, countField.node);
      final removed = applyPatches(settingsSource, [
        SourcePatch(span.offset, span.length, ''),
      ]);

      expect(removed, isNot(contains('final int count')));
      expect(removed, contains('/// Application settings'));
      expect(removed, contains('class Settings'));
      expect(removed, contains('void update()'));
    });

    test('removes import directive', () {
      final interpreter = AstPathInterpreter();
      final focus = interpreter.navigateTo(settingsSource, [
        const NavigateStep(NavigateKind.import, name: 'package:foo/bar.dart'),
      ]);

      final span = declarationSpan(settingsSource, focus.node);
      final removed = applyPatches(settingsSource, [
        SourcePatch(span.offset, span.length, ''),
      ]);

      expect(removed, isNot(contains("import 'package:foo/bar.dart'")));
      expect(removed, contains('class Settings'));
    });
  });

  group('AstPathPatchTransform', () {
    test('remove step without anchor deletes full field', () async {
      final transform = AstPathPatchTransform.remove(
        target: const AstPathPatchTarget(
          navigate: [
            NavigateStep(NavigateKind.classDecl, name: 'Settings'),
            NavigateStep(NavigateKind.field, name: 'count'),
          ],
        ),
      );

      final patches = await transform.apply(settingsSource, CodemodContext(const {}));
      final result = applyPatches(settingsSource, patches);

      expect(result, isNot(contains('final int count')));
      expect(result, contains('final String name'));
    });

    test('replace step without anchor replaces full field', () async {
      final transform = AstPathPatchTransform.replace(
        target: const AstPathPatchTarget(
          navigate: [
            NavigateStep(NavigateKind.classDecl, name: 'Settings'),
            NavigateStep(NavigateKind.field, name: 'count'),
          ],
        ),
        text: 'final int count = 99;',
      );

      final patches = await transform.apply(settingsSource, CodemodContext(const {}));
      final result = applyPatches(settingsSource, patches);

      expect(result, contains('final int count = 99;'));
      expect(result, isNot(contains('final int count = 0')));
    });

    test('remove is no-op when target is absent', () async {
      final transform = AstPathPatchTransform.remove(
        target: const AstPathPatchTarget(
          navigate: [
            NavigateStep(NavigateKind.classDecl, name: 'Settings'),
            NavigateStep(NavigateKind.field, name: 'missing'),
          ],
        ),
      );

      final patches = await transform.apply(settingsSource, CodemodContext(const {}));
      expect(patches, isEmpty);
    });

    test('rejects point anchor on remove', () async {
      final transform = AstPathPatchTransform.remove(
        target: const AstPathPatchTarget(
          navigate: [
            NavigateStep(NavigateKind.classDecl, name: 'Settings'),
            NavigateStep(NavigateKind.method, name: 'update'),
          ],
          anchor: Anchor(AnchorKind.stmtLast),
        ),
      );

      expect(
        () => transform.apply(settingsSource, CodemodContext(const {})),
        throwsA(isA<StateError>()),
      );
    });
  });
}
