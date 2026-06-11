import 'dart:io';

import 'package:codemod_recipe/codemod_recipe.dart';
import 'package:test/test.dart';

void main() {
  group('patch helpers', () {
    test('preserves declaration order for same-offset insertions', () {
      final result = applyPatches('ab', [
        SourcePatch(1, 0, 'X'),
        SourcePatch(1, 0, 'Y'),
      ]);

      expect(result, 'aXYb');
    });

    test('rejects overlapping replacement patches', () {
      expect(
        () => validateNonOverlappingPatches([
          SourcePatch(1, 3, 'first'),
          SourcePatch(2, 1, 'second'),
        ]),
        throwsStateError,
      );
    });

    test('allows adjacent replacement patches', () {
      final patches = [SourcePatch(0, 1, 'A'), SourcePatch(1, 1, 'B')];

      validateNonOverlappingPatches(patches);

      expect(applyPatches('ab', patches), 'AB');
    });

    test('applies interleaved offsets deterministically', () {
      final result = applyPatches('abcd', [
        SourcePatch(3, 0, 'Z'),
        SourcePatch(1, 0, 'X'),
        SourcePatch(2, 0, 'Y'),
      ]);

      expect(result, 'aXbYcZd');
    });
  });

  group('recipes', () {
    test('compose deduplicates shared args and concatenates operations', () {
      final arg = CodemodArg.required('name');
      final first = CodemodRecipe(
        name: 'first',
        args: [arg],
        operations: [
          EditDartFileOperation(path: (_) => 'a.dart', transforms: (_) => []),
        ],
      );
      final second = CodemodRecipe(
        name: 'second',
        args: [arg],
        operations: [
          EditDartFileOperation(path: (_) => 'b.dart', transforms: (_) => []),
        ],
      );

      final composed = CodemodRecipe.compose(
        name: 'composed',
        steps: [first, second],
      );

      expect(composed.args, hasLength(1));
      expect(composed.operations, hasLength(2));
    });

    test('compose accepts an empty step list', () {
      final composed = CodemodRecipe.compose(name: 'empty', steps: const []);

      expect(composed.args, isEmpty);
      expect(composed.operations, isEmpty);
      expect(composed.postExecution, isEmpty);
    });

    test('compose keeps explicit arg definitions before recipe args', () {
      const explicit = CodemodArg.required('name', help: 'Explicit help');
      const recipeArg = CodemodArg.optional('name', help: 'Recipe help');
      final recipe = CodemodRecipe(
        name: 'recipe',
        args: const [recipeArg],
        operations: const [],
      );

      final composed = CodemodRecipe.compose(
        name: 'composed',
        args: const [explicit],
        steps: [recipe],
      );

      expect(composed.args.single.required, isTrue);
      expect(composed.args.single.help, 'Explicit help');
    });

    test('compose interleaves recipes and operations in step order', () {
      final first = CodemodRecipe(
        name: 'first',
        operations: [
          EditDartFileOperation(path: (_) => 'a.dart', transforms: (_) => []),
        ],
      );
      final inline = EditDartFileOperation(
        path: (_) => 'inline.dart',
        transforms: (_) => [],
      );
      final second = CodemodRecipe(
        name: 'second',
        operations: [
          EditDartFileOperation(path: (_) => 'b.dart', transforms: (_) => []),
        ],
      );

      final composed = CodemodRecipe.compose(
        name: 'composed',
        steps: [first, inline, second],
      );

      final context = CodemodContext();
      expect(
        composed.operations
            .map(
              (operation) =>
                  (operation as EditDartFileOperation).path(context),
            )
            .toList(),
        ['a.dart', 'inline.dart', 'b.dart'],
      );
    });

    test('compose preserves post-execution order from steps', () {
      const format = DartFormatPostExecution();
      final recipeWithFormat = CodemodRecipe(
        name: 'recipe',
        operations: const [],
        postExecution: [format],
      );
      const buildRunner = BuildRunnerPostExecution();

      final composed = CodemodRecipe.compose(
        name: 'composed',
        steps: [recipeWithFormat, buildRunner],
      );

      expect(composed.postExecution, [format, buildRunner]);
    });

    test('compose accepts inline post-execution between recipe steps', () {
      const format = DartFormatPostExecution();
      final first = CodemodRecipe(
        name: 'first',
        operations: const [],
        postExecution: [format],
      );
      const buildRunner = BuildRunnerPostExecution();
      final second = CodemodRecipe(name: 'second', operations: const []);

      final composed = CodemodRecipe.compose(
        name: 'composed',
        steps: [first, buildRunner, second],
      );

      expect(composed.postExecution, [format, buildRunner]);
    });
  });

  group('templates', () {
    test('renders explicit casing placeholders', () {
      final context = CodemodContext({'feature': 'FeedList'});
      final result = context.render(
        '{{feature}} {{feature:snake}} {{feature:camel}} {{feature:pascal}}',
      );

      expect(result, 'FeedList feed_list feedList FeedList');
    });

    test('fails when a placeholder value is missing', () {
      expect(
        () => CodemodTemplate.inline(
          '{{feature:snake}}',
        ).render(CodemodContext()),
        throwsStateError,
      );
    });

    test('renders templates that do not need context values', () {
      final result = CodemodTemplate.inline(
        'class StaticExample {}',
      ).render(CodemodContext());

      expect(result, 'class StaticExample {}');
    });

    test('preserves special characters in raw placeholders', () {
      final context = CodemodContext({'message': r'Hello $world {{literal}}'});

      final result = CodemodTemplate.inline('{{message}}').render(context);

      expect(result, r'Hello $world {{literal}}');
    });

    test('renders unicode values without dropping characters', () {
      final context = CodemodContext({'label': 'Über'});

      final result = CodemodTemplate.inline(
        '{{label}} {{label:snake}}',
      ).render(context);

      expect(result, 'Über über');
    });

    test('fails on unsupported casing filters', () {
      expect(
        () => CodemodTemplate.inline(
          '{{feature:kebab}}',
        ).render(CodemodContext({'feature': 'FeedList'})),
        throwsA(isA<Object>()),
      );
    });
  });

  group('operations', () {
    test('creates a file from a template', () async {
      final tempDir = await Directory.systemTemp.createTemp('codemod_recipe_');
      addTearDown(() => tempDir.delete(recursive: true));

      final operation = CreateFileOperation(
        path: (context) => '${tempDir.path}/lib/{{feature:snake}}.dart'
            .replaceAll('{{feature:snake}}', context.snake('feature')),
        template: const CodemodTemplate.inline('class {{feature:pascal}} {}'),
      );

      final changes = await operation.collect(
        CodemodContext({'feature': 'Feed'}),
      );

      expect(changes, hasLength(1));
      expect(changes.single.preview(), contains('class Feed {}'));

      await changes.single.apply();

      expect(
        await File('${tempDir.path}/lib/feed.dart').readAsString(),
        'class Feed {}',
      );
    });
  });

  group('post execution', () {
    test('reports only changed formattable paths', () {
      final result = CodemodRunResult(
        changes: [
          const PatchFileChange(
            path: 'lib/a.dart',
            source: 'class A {}',
            patches: [SourcePatch(0, 0, 'final a = 1;\n')],
          ),
          const CreateFileChange(
            path: 'README.md',
            content: '# Readme',
            exists: false,
            shouldFormat: false,
          ),
        ],
      );

      expect(result.formattablePaths, ['lib/a.dart']);
    });
  });

  group('CodeEditor', () {
    test('adds a method once', () {
      final patches = CodeEditor(_source)
          .inClass('Counter')
          .addMethodUnlessExists('increment', '''
  void increment() {
    value++;
  }''')
          .patches;

      final result = applyPatches(_source, patches);

      expect(result, contains('void increment()'));
      expect(
        CodeEditor(result)
            .inClass('Counter')
            .addMethodUnlessExists('increment', 'void increment() {}')
            .patches,
        isEmpty,
      );
    });

    test('adds a method to an empty class body', () {
      const source = 'class Empty {}';

      final result = CodeEditor(source).inClass('Empty').addMethodUnlessExists(
        'build',
        '''
  void build() {}''',
      ).generate();

      expect(result, contains('void build() {}'));
    });

    test('adds a final field with constructor parameter by default', () {
      const source = '''
class Counter {
  const Counter();
}
''';

      final result = CodeEditor(source)
          .inClass('Counter')
          .addField('value', 'int')
          .generate();

      expect(result, contains('final int value;'));
      expect(result, contains('this.value'));
    });

    test('adds a static field without constructor parameter', () {
      const source = '''
class Counter {
  Counter({required this.value});
  final int value;
}
''';

      final result = CodeEditor(source)
          .inClass('Counter')
          .addField('count', 'int', addToConstructor: true, isStatic: true)
          .generate();

      expect(result, contains('static final int count;'));
      expect(result, isNot(contains('this.count')));
    });

    test('adds a const field with initializer', () {
      const source = 'class Counter {}';

      final result = CodeEditor(source)
          .inClass('Counter')
          .addField(
            'zero',
            'int',
            defaultValue: '0',
            addToConstructor: false,
            isConst: true,
          )
          .generate();

      expect(result, contains('const int zero = 0;'));
    });

    test('adds a field once', () {
      const source = 'class Counter {}';

      final result = CodeEditor(source)
          .inClass('Counter')
          .addFieldUnlessExists('value', 'int')
          .generate();

      expect(result, contains('final int value;'));
      expect(
        CodeEditor(result)
            .inClass('Counter')
            .addFieldUnlessExists('value', 'int')
            .patches,
        isEmpty,
      );
    });
  });

  group('generic transforms', () {
    test('adds an import once', () async {
      final transform = AddImportTransform.uri((_) => 'package:app/app.dart');

      final patches = await transform.apply(_source, CodemodContext());
      final result = applyPatches(_source, patches);
      final secondRunPatches = await transform.apply(result, CodemodContext());

      expect(result, startsWith("import 'package:app/app.dart';"));
      expect(secondRunPatches, isEmpty);
    });

    test('adds a rendered method once', () async {
      final transform = AddMethodTransform(
        className: (_) => 'Counter',
        methodName: (context) => context.camel('method'),
        body: const CodemodTemplate.inline('''
  void {{method:camel}}() {}
'''),
      );
      final context = CodemodContext({'method': 'Increment'});

      final patches = await transform.apply(_source, context);
      final result = applyPatches(_source, patches);
      final secondRunPatches = await transform.apply(result, context);

      expect(result, contains('void increment() {}'));
      expect(secondRunPatches, isEmpty);
    });

    test('adds a constructor parameter once', () async {
      const source = '''
class Counter {
  const Counter();
}
''';
      final transform = AddConstructorParamTransform(
        className: (_) => 'Counter',
        paramName: (_) => 'value',
        paramType: (_) => 'int',
      );

      final patches = await transform.apply(source, CodemodContext());
      final result = applyPatches(source, patches);
      final secondRunPatches = await transform.apply(result, CodemodContext());

      expect(result, contains('this.value'));
      expect(secondRunPatches, isEmpty);
    });

    test('adds a static field once', () async {
      const source = '''
class Counter {
  Counter();
}
''';
      final transform = AddFieldTransform(
        className: (_) => 'Counter',
        fieldName: (_) => 'count',
        fieldType: (_) => 'int',
        isStatic: true,
      );

      final patches = await transform.apply(source, CodemodContext());
      final result = applyPatches(source, patches);
      final secondRunPatches = await transform.apply(result, CodemodContext());

      expect(result, contains('static final int count;'));
      expect(result, isNot(contains('this.count')));
      expect(secondRunPatches, isEmpty);
    });
  });
}

const _source = '''
class Counter {
  int value = 0;
}
''';
