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
      final path = parsePathString(
        'class:Settings > method:update @ stmt:last',
      );

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

    test('parses v2 anchors', () {
      expect(
        parseAnchor('param:name:key'),
        const Anchor(AnchorKind.paramName, name: 'key'),
      );
      expect(
        parseAnchor('arg:name:home'),
        const Anchor(AnchorKind.argName, name: 'home'),
      );
      expect(parseAnchor('doc:before'), const Anchor(AnchorKind.docBefore));
      expect(parseAnchor('doc:after'), const Anchor(AnchorKind.docAfter));
      expect(
        parseAnchor('initializer:replace'),
        const Anchor(AnchorKind.initializerReplace),
      );
      expect(
        parseAnchor('param:0'),
        const Anchor(AnchorKind.paramIndex, index: 0),
      );
    });

    test('parses field navigate step', () {
      final path = parsePathString(
        'class:Settings > field:count @ initializer:replace',
      );
      expect(path.navigate.last.kind, NavigateKind.field);
      expect(path.navigate.last.name, 'count');
      expect(path.anchor.kind, AnchorKind.initializerReplace);
    });

    test('parses navigate match filter', () {
      final path = parseStructuredPath({
        'at': [
          {'class': 'DerivedSettings', 'match': 'extends BaseSettings'},
        ],
        'anchor': 'member:last',
      });

      expect(path.navigate.single.match, 'extends BaseSettings');
    });
  });

  group('AstPathInterpreter', () {
    final interpreter = AstPathInterpreter();

    test('resolves stmt:last in method body', () {
      final path = parsePathString(
        'class:Settings > method:update @ stmt:last',
      );
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

    test('resolves param:name anchor on constructor', () {
      final path = parsePathString('class:Widget > ctor: @ param:name:key');
      final offset = interpreter.resolveOffset(settingsSource, path);

      expect(settingsSource.substring(offset - 3, offset), 'key');
    });

    test('resolves arg:name anchor on constructor call', () {
      final path = parsePathString('call:MaterialApp @ arg:name:home');
      final offset = interpreter.resolveOffset(settingsSource, path);

      final patched = applyPatches(settingsSource, [
        SourcePatch(offset, 0, ', title: "x"'),
      ]);
      expect(patched, contains('MaterialApp(home: Container(), title: "x")'));
    });

    test('resolves doc:before on class', () {
      final path = parsePathString('class:Settings @ doc:before');
      final offset = interpreter.resolveOffset(settingsSource, path);

      expect(settingsSource.substring(offset, offset + 3), '///');
    });

    test('resolves doc:after on class', () {
      final path = parsePathString('class:Settings @ doc:after');
      final offset = interpreter.resolveOffset(settingsSource, path);

      expect(settingsSource.substring(offset, offset + 5), 'class');
    });

    test('resolves initializer:replace on field', () {
      final path = parsePathString(
        'class:Settings > field:count @ initializer:replace',
      );
      final span = interpreter.resolveSpan(settingsSource, path);

      expect(
        settingsSource.substring(span.offset, span.offset + span.length),
        '0',
      );
    });

    test('disambiguates classes with match filter', () {
      final path = parseStructuredPath({
        'at': [
          {'class': 'DerivedSettings', 'match': 'extends BaseSettings'},
        ],
        'anchor': 'member:last',
      });
      final offset = interpreter.resolveOffset(settingsSource, path);

      expect(settingsSource.substring(offset - 4, offset), '= 0;');
    });

    test('resolves stmt:last in function', () {
      final path = parsePathString('function:build @ stmt:last');
      final offset = interpreter.resolveOffset(settingsSource, path);

      expect(settingsSource.substring(offset - 1, offset), ';');
    });

    test('resolves body:end in function', () {
      final path = parsePathString('function:build @ body:end');
      final offset = interpreter.resolveOffset(settingsSource, path);

      expect(settingsSource.substring(offset, offset + 1), '}');
    });

    test('resolves arg:last in call', () {
      final path = parsePathString('call:MaterialApp @ arg:last');
      final offset = interpreter.resolveOffset(settingsSource, path);

      final patched = applyPatches(settingsSource, [
        SourcePatch(offset, 0, ', title: "test"'),
      ]);
      expect(patched, contains('MaterialApp(home: Container(), title: "test")'));
    });

    test('resolves doc:before on main function', () {
      final path = parsePathString('function:main @ doc:before');
      final offset = interpreter.resolveOffset(settingsSource, path);

      expect(settingsSource.substring(offset, offset + 6), 'void m');
    });

    test('resolves type-inferred navigation to class', () {
      final path = parsePathString('Settings @ member:last');
      final offset = interpreter.resolveOffset(settingsSource, path);

      expect(settingsSource.substring(offset - 1, offset + 1), '}\n');
    });

    test('resolves type-inferred navigation to method', () {
      final path = parseStructuredPath({
        'at': ['Settings', 'update'],
        'anchor': 'stmt:last',
      });
      final offset = interpreter.resolveOffset(settingsSource, path);

      expect(settingsSource.substring(offset - 1, offset), ';');
    });

    test('parses string path with full navigation and anchor', () {
      final path = parsePathString('class:Settings > method:update @ stmt:last');
      
      expect(path.navigate.length, 2);
      expect(path.navigate[0].kind, NavigateKind.classDecl);
      expect(path.navigate[0].name, 'Settings');
      expect(path.navigate[1].kind, NavigateKind.method);
      expect(path.navigate[1].name, 'update');
      expect(path.anchor.kind, AnchorKind.stmtLast);
    });

    test('parses string path with type-inferred steps', () {
      final path = parsePathString('Settings > update @ stmt:last');
      
      expect(path.navigate.length, 2);
      expect(path.navigate[0].kind, isNull);
      expect(path.navigate[0].name, 'Settings');
      expect(path.navigate[1].kind, isNull);
      expect(path.navigate[1].name, 'update');
      expect(path.anchor.kind, AnchorKind.stmtLast);
    });

    test('resolves stmt:last in main function', () {
      final path = parsePathString('function:main @ stmt:last');
      final offset = interpreter.resolveOffset(settingsSource, path);

      expect(settingsSource.substring(offset - 1, offset + 1), ';\n');
    });

    test('throws E_NODE_NOT_FOUND for missing function', () {
      final path = parsePathString('function:nonExistent @ stmt:last');

      expect(
        () => interpreter.resolveOffset(settingsSource, path),
        throwsA(
          predicate<AstPathResolutionException>(
            (error) => error.code == 'E_NODE_NOT_FOUND',
          ),
        ),
      );
    });

    test('resolves body:end in main function', () {
      final path = parsePathString('function:main @ body:end');
      final offset = interpreter.resolveOffset(settingsSource, path);

      expect(settingsSource.substring(offset, offset + 1), '}');
    });
  });
}
