import 'dart:io';

import 'package:codemod_recipe/codemod_recipe.dart';
import 'package:test/test.dart';

enum _TestMode { increment, reset }

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

  group('arg codecs', () {
    test('round-trips bool values', () {
      const codec = BoolArgCodec();
      expect(codec.parse('true'), isTrue);
      expect(codec.parse('false'), isFalse);
      expect(codec.serialize(true), 'true');
    });

    test('round-trips int values', () {
      const codec = IntArgCodec();
      expect(codec.parse('42'), 42);
      expect(codec.serialize(42), '42');
    });
  });

  group('recipes', () {
    test('compose deduplicates shared args and concatenates operations', () {
      final arg = CodemodArg<String>.required('name');
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

    test('enum arg infers enumeration input kind and options', () {
      final arg = CodemodArg<_TestMode>.optional(
        'mode',
        defaultsTo: _TestMode.increment,
        enumValues: _TestMode.values,
      );

      expect(arg.resolvedInputKind, CodemodArgInputKind.enumeration);
      expect(arg.options, ['increment', 'reset']);
      expect(arg.serializedDefault, 'increment');
    });

    test('typed bool arg infers bool input kind and serializes default', () {
      final arg = CodemodArg<bool>.optional(
        'addToConstructor',
        defaultsTo: false,
      );

      expect(arg.resolvedInputKind, CodemodArgInputKind.boolean);
      expect(arg.serializedDefault, 'false');
    });

    test('fixed arg is hidden and serializes pinned default', () {
      final arg = CodemodArg<String>.fixed('root', 'lib/features');

      expect(arg.hidden, isTrue);
      expect(arg.isUserFacing, isFalse);
      expect(arg.serializedDefault, 'lib/features');
    });

    test('hidden fixed args inject defaults into context', () {
      final args = [
        CodemodArg<String>.fixed('root', 'lib/features'),
        CodemodArg<String>.required('feature'),
      ];

      final context = CodemodContext(const {});
      for (final arg in args) {
        arg.contributeToContext(
          context,
          rawValue: arg.name == 'feature' ? 'UserProfile' : null,
        );
      }

      expect(context.get<String>('root'), 'lib/features');
      expect(context.get<String>('feature'), 'UserProfile');
    });

    test('typed args store parsed values in context', () {
      final args = [
        CodemodArg<bool>.optional('addToConstructor', defaultsTo: true),
        CodemodArg<int>.optional('count', defaultsTo: 0),
        CodemodArg<_TestMode>.required(
          'mode',
          enumValues: _TestMode.values,
        ),
      ];

      final context = CodemodContext(const {});
      for (final arg in args) {
        final rawValue = switch (arg.name) {
          'addToConstructor' => 'false',
          'count' => '42',
          'mode' => 'reset',
          _ => null,
        };
        expect(arg.contributeToContext(context, rawValue: rawValue), isNull);
      }

      expect(context.get<bool>('addToConstructor'), isFalse);
      expect(context.get<int>('count'), 42);
      expect(context.get<_TestMode>('mode'), _TestMode.reset);
      expect(context.values['addToConstructor'], 'false');
      expect(context.values['count'], '42');
      expect(context.values['mode'], 'reset');
    });

    test('get throws on type mismatch', () {
      final context = CodemodContext({'flag': true});
      expect(() => context.get<String>('flag'), throwsStateError);
    });

    test('compose replaces step arg with explicit fixed arg', () {
      final stepRecipe = CodemodRecipe(
        name: 'step',
        args: [CodemodArg<String>.required('root')],
        operations: const [],
      );

      final composed = CodemodRecipe.compose(
        name: 'composed',
        args: [CodemodArg<String>.fixed('root', 'lib/features')],
        steps: [stepRecipe],
      );

      expect(composed.args.single.hidden, isTrue);
      expect(composed.args.single.serializedDefault, 'lib/features');
    });

    test('compose keeps explicit arg definitions before recipe args', () {
      final explicit = CodemodArg<String>.required('name', help: 'Explicit help');
      final recipeArg = CodemodArg<String>.optional('name', help: 'Recipe help');
      final recipe = CodemodRecipe(
        name: 'recipe',
        args: [recipeArg],
        operations: const [],
      );

      final composed = CodemodRecipe.compose(
        name: 'composed',
        args: [explicit],
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
      final format = DartFormatPostExecution();
      final recipeWithFormat = CodemodRecipe(
        name: 'recipe',
        operations: const [],
        postExecution: [format],
      );
      final buildRunner = BuildRunnerPostExecution();

      final composed = CodemodRecipe.compose(
        name: 'composed',
        steps: [recipeWithFormat, buildRunner],
      );

      expect(composed.postExecution, [format, buildRunner]);
    });

    test('compose accepts inline post-execution between recipe steps', () {
      final format = DartFormatPostExecution();
      final first = CodemodRecipe(
        name: 'first',
        operations: const [],
        postExecution: [format],
      );
      final buildRunner = BuildRunnerPostExecution();
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

  group('AstFocus', () {
    test('afterLastArgument follows the last named argument', () {
      final focus = AstFocus.parse(_settingsGetterSource)
          .classNamed('SettingsRepository')
          .methodNamed('get')
          .instanceCreation('Settings', returnExpressionOnly: true);

      final offset = focus.afterLastArgument;
      final result =
          _settingsGetterSource.substring(0, offset) +
          '\n      newField: _getNewField(),' +
          _settingsGetterSource.substring(offset);

      expect(result, contains('language: _getLanguage(),'));
      expect(result, contains('newField: _getNewField(),'));
      expect(
        result.indexOf('newField'),
        greaterThan(result.indexOf('language')),
      );
    });

    test('planArgument appends after the last argument', () {
      final focus = AstFocus.parse(_settingsGetterSource)
          .classNamed('SettingsRepository')
          .methodNamed('get')
          .instanceCreation('Settings', returnExpressionOnly: true);
      final plan = focus.planArgument('newField: _getNewField(),');

      final result = applyPatches(_settingsGetterSource, [
        SourcePatch(plan.offset, plan.length, plan.text),
      ]);

      expect(result, contains('language: _getLanguage(),'));
      expect(result, contains('newField: _getNewField(),'));
    });

    test('returnExpressionOnly ignores nested instance creations', () {
      const source = '''
class Repo {
  Settings get() {
  final helper = transform(Settings(darkMode: true));
  return Settings(language: _getLanguage());
  }
}
''';

      final focus = AstFocus.parse(source)
          .classNamed('Repo')
          .methodNamed('get')
          .instanceCreation('Settings', returnExpressionOnly: true);

      expect(focus.argumentsHaveNamed('language'), isTrue);
      expect(focus.argumentsHaveNamed('darkMode'), isFalse);
    });

    test('argumentsHaveNamed detects existing named arguments', () {
      final focus = AstFocus.parse(_settingsGetterSource)
          .classNamed('SettingsRepository')
          .methodNamed('get')
          .instanceCreation('Settings', returnExpressionOnly: true);

      expect(focus.argumentsHaveNamed('language'), isTrue);
      expect(focus.argumentsHaveNamed('missing'), isFalse);
    });
  });

  group('CodeEditor', () {
    test('adds a method once', () {
      final focus = AstFocus.parse(_source).classNamed('Counter');
      final patches = CodeEditor(_source)
          .addMethodUnlessExists(focus, 'increment', '''
  void increment() {
    value++;
  }''')
          .patches;

      final result = applyPatches(_source, patches);

      expect(result, contains('void increment()'));
      expect(
        CodeEditor(result)
            .addMethodUnlessExists(
              AstFocus.parse(result).classNamed('Counter'),
              'increment',
              'void increment() {}',
            )
            .patches,
        isEmpty,
      );
    });

    test('adds a method to an empty class body', () {
      const source = 'class Empty {}';
      final focus = AstFocus.parse(source).classNamed('Empty');

      final result = CodeEditor(source).addMethodUnlessExists(
        focus,
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
      final focus = AstFocus.parse(source).classNamed('Counter');

      final result = CodeEditor(source).addField(focus, 'value', 'int').generate();

      expect(result, contains('final int value;'));
      expect(result, contains('required this.value'));
      expect(result, isNot(contains('Counter(,')));
    });

    test('adds a nullable field type', () {
      const source = 'class Counter {}';
      final focus = AstFocus.parse(source).classNamed('Counter');

      final result = CodeEditor(source)
          .addField(focus, 'name', 'String', isNullable: true, addToConstructor: false)
          .generate();

      expect(result, contains('final String? name;'));
    });

    test('adds required named constructor param to existing brace list', () {
      const source = '''
class Counter {
  Counter({this.a});
  final int a;
}
''';
      final focus = AstFocus.parse(source).classNamed('Counter');

      final result = CodeEditor(source).addField(focus, 'b', 'int').generate();

      expect(result, contains('required this.b'));
    });

    test('adds optional named constructor param for nullable field', () {
      const source = '''
class Counter {
  Counter({this.a = 1});
  final int a;
}
''';
      final focus = AstFocus.parse(source).classNamed('Counter');

      final result = CodeEditor(source)
          .addField(focus, 'b', 'String', isNullable: true)
          .generate();

      expect(result, contains('final String? b;'));
      expect(result, contains('this.b'));
      expect(result, isNot(contains('required this.b')));
    });

    test('adds optional positional constructor param with default', () {
      const source = '''
class Counter {
  Counter([this.a]);
  final int? a;
}
''';
      final focus = AstFocus.parse(source).classNamed('Counter');

      final result = CodeEditor(source)
          .addField(focus, 'b', 'int', defaultValue: '0')
          .generate();

      expect(result, contains('this.b = 0'));
    });

    test('adds positional constructor param to positional constructor', () {
      const source = '''
class Counter {
  Counter(this.a);
  final int a;
}
''';
      final focus = AstFocus.parse(source).classNamed('Counter');

      final result = CodeEditor(source).addField(focus, 'b', 'int').generate();

      expect(
        result.contains('this.a, this.b') ||
            result.contains('this.a,\n    this.b'),
        isTrue,
      );
    });

    test('uses named style for empty constructor by default', () {
      const source = '''
class Counter {
  Counter();
}
''';
      final focus = AstFocus.parse(source).classNamed('Counter');

      final result = CodeEditor(source).addField(focus, 'value', 'int').generate();

      expect(result, contains('({required this.value})'));
    });

    test('uses positional style for empty constructor when preferred', () {
      const source = '''
class Counter {
  Counter();
}
''';
      final focus = AstFocus.parse(source).classNamed('Counter');

      final result = CodeEditor(
        source,
        preferences: const CodemodPreferences(
          emptyConstructorStyle: ConstructorParamStyle.positional,
        ),
      ).addField(focus, 'value', 'int').generate();

      expect(result, contains('Counter(this.value)'));
    });

    test('overrides empty constructor style per field', () {
      const source = '''
class Counter {
  Counter();
}
''';
      final focus = AstFocus.parse(source).classNamed('Counter');

      final result = CodeEditor(source)
          .addField(
            focus,
            'value',
            'int',
            constructorArgs: const FieldConstructorArgs(
              style: ConstructorParamStyle.positional,
            ),
          )
          .generate();

      expect(result, contains('Counter(this.value)'));
      expect(result, isNot(contains('required this.value')));
    });

    test('wires field to constructor once', () {
      const source = '''
class Counter {
  Counter();
}
''';
      const spec = FieldSpec(name: 'value', type: 'int');
      final focus = AstFocus.parse(source).classNamed('Counter');

      final result = CodeEditor(source)
          .addFieldToConstructorUnlessExists(focus, spec)
          .generate();

      expect(result, contains('required this.value'));
      expect(
        CodeEditor(result)
            .addFieldToConstructorUnlessExists(
              AstFocus.parse(result).classNamed('Counter'),
              spec,
            )
            .patches,
        isEmpty,
      );
    });

    test('adds a static field without constructor parameter', () {
      const source = '''
class Counter {
  Counter({required this.value});
  final int value;
}
''';
      final focus = AstFocus.parse(source).classNamed('Counter');

      final result = CodeEditor(source)
          .addField(focus, 'count', 'int', addToConstructor: true, isStatic: true)
          .generate();

      expect(result, contains('static final int count;'));
      expect(result, isNot(contains('this.count')));
    });

    test('adds a const field with initializer', () {
      const source = 'class Counter {}';
      final focus = AstFocus.parse(source).classNamed('Counter');

      final result = CodeEditor(source)
          .addField(
            focus,
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
      final focus = AstFocus.parse(source).classNamed('Counter');

      final result = CodeEditor(source)
          .addFieldUnlessExists(focus, 'value', 'int')
          .generate();

      expect(result, contains('final int value;'));
      expect(
        CodeEditor(result)
            .addFieldUnlessExists(
              AstFocus.parse(result).classNamed('Counter'),
              'value',
              'int',
            )
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

      expect(result, contains('required this.value'));
      expect(secondRunPatches, isEmpty);
    });

    test('adds a nullable field once', () async {
      const source = 'class Counter {}';
      final transform = AddFieldTransform(
        className: (_) => 'Counter',
        fieldName: (_) => 'name',
        fieldType: (_) => 'String',
        isNullable: true,
      );

      final patches = await transform.apply(source, CodemodContext());
      final result = applyPatches(source, patches);

      expect(result, contains('final String? name;'));
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

const _settingsGetterSource = '''
class SettingsRepository {
  @override
  Settings get() {
    return Settings(
      darkMode: _getDarkMode(),
      logTraffic: _getLogTraffic(),
      language: _getLanguage(),
    );
  }
}
''';
