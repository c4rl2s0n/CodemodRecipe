import 'dart:io';

import 'package:codemod_recipe/codemod_recipe.dart';
import 'package:test/test.dart';

void main() {
  late String settingsSource;

  setUp(() {
    settingsSource = File(
      'test/fixtures/ast_paths/settings.dart',
    ).readAsStringSync();
  });

  group('AstPathParser', () {
    test('parses structured path', () {
      final path = parseStructuredPath({
        'at': [
          {'class': 'Settings'},
          {'method': 'update'},
        ],
        'anchor': 'stmt:last',
      });

      expect(
        path,
        const AstPath(
          navigate: [
            NavigateStep(NavigateKind.classDecl, name: 'Settings'),
            NavigateStep(NavigateKind.method, name: 'update'),
          ],
          anchor: Anchor(AnchorKind.stmtLast),
        ),
      );
    });

    test('parses path string with typed steps', () {
      final path = parsePathString('class:Settings > method:update @ stmt:last');

      expect(path.navigate.length, 2);
      expect(path.anchor.kind, AnchorKind.stmtLast);
    });

    test('parses stmt dollar alias', () {
      final path = parsePathString(r'class:Settings > method:update @ stmt:$');
      expect(path.anchor.kind, AnchorKind.stmtLast);
    });

    test('parses ctor without name', () {
      final path = parseStructuredPath({
        'at': [
          {'class': 'Widget'},
          'ctor:',
        ],
        'anchor': 'param:last',
      });

      expect(path.navigate.last.kind, NavigateKind.constructor);
      expect(path.navigate.last.name, isNull);
    });

    test('rejects unknown anchor', () {
      expect(
        () => parseAnchor('stmt:first'),
        throwsA(isA<AstPathParseException>()),
      );
    });
  });

  group('AstPathInterpreter', () {
    final interpreter = AstPathInterpreter();

    test('resolves stmt:last in method body', () {
      final path = parsePathString('class:Settings > method:update @ stmt:last');
      final offset = interpreter.resolveOffset(settingsSource, path);

      expect(settingsSource.substring(offset - 1, offset), ';');
      final patched = applyPatches(settingsSource, [
        SourcePatch(offset, 0, '\n    // inserted'),
      ]);
      expect(patched, contains("// inserted"));
    });

    test('resolves member:last on class', () {
      final path = parsePathString('class:Settings @ member:last');
      final offset = interpreter.resolveOffset(settingsSource, path);

      expect(settingsSource.substring(offset - 1, offset + 1), '}\n');
    });

    test('resolves meta:before on class', () {
      final path = parsePathString('class:Settings @ meta:before');
      final offset = interpreter.resolveOffset(settingsSource, path);

      expect(settingsSource.substring(offset, offset + 7), 'class S');
    });

    test('resolves param:last on constructor', () {
      final path = parseStructuredPath({
        'at': [
          {'class': 'Widget'},
          'ctor:',
        ],
        'anchor': 'param:last',
      });
      final offset = interpreter.resolveOffset(settingsSource, path);

      expect(settingsSource.substring(offset - 3, offset), 'key');
    });

    test('resolves arg:last on constructor call', () {
      final path = parsePathString('call:MaterialApp @ arg:last');
      final offset = interpreter.resolveOffset(settingsSource, path);

      final patched = applyPatches(settingsSource, [
        SourcePatch(offset, 0, ', title: "x"'),
      ]);
      expect(patched, contains('MaterialApp(home: Container(), title: "x")'));
    });

    test('throws E_NODE_NOT_FOUND for missing class', () {
      final path = parsePathString('class:Missing @ member:last');

      expect(
        () => interpreter.resolveOffset(settingsSource, path),
        throwsA(
          predicate<AstPathResolutionException>(
            (error) => error.code == 'E_NODE_NOT_FOUND',
          ),
        ),
      );
    });

    test('throws E_ANCHOR_INVALID for stmt:last on class', () {
      final path = parsePathString('class:Settings @ stmt:last');

      expect(
        () => interpreter.resolveOffset(settingsSource, path),
        throwsA(
          predicate<AstPathResolutionException>(
            (error) => error.code == 'E_ANCHOR_INVALID',
          ),
        ),
      );
    });
  });
}
