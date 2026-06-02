import 'dart:io';

import 'package:codemod_recipe/codemod_recipe_vscode.dart';
import 'package:test/test.dart';

CodemodRecipe _addMethodRecipe() {
  return CodemodRecipe(
    name: 'add_method',
    description: 'Adds a method to a class',
    args: [
      CodemodArg.required(
        'file',
        inputKind: CodemodArgInputKind.file,
        contextKey: CodemodContextKey.file,
      ),
      CodemodArg.required(
        'class',
        inputKind: CodemodArgInputKind.symbol,
        contextKey: CodemodContextKey.dartClass,
      ),
      CodemodArg.required(
        'method',
        inputKind: CodemodArgInputKind.symbol,
        options: ['reset', 'increment'],
        contextKey: CodemodContextKey.word,
      ),
    ],
    operations: [
      EditDartFileOperation(
        path: (context) => context.require('file'),
        transforms: (context) => [
          AddMethodTransform(
            className: (c) => c.require('class'),
            methodName: (c) => c.camel('method'),
            body: const CodemodTemplate.inline(
              '  void {{method:camel}}() {}\n',
            ),
          ),
        ],
      ),
    ],
  );
}

void main() {
  group('RecipeSchema', () {
    test('serializes args and registry', () {
      final json = RecipeSchema.registryToJson({'add': _addMethodRecipe()});
      expect(json, hasLength(1));
      expect(json.first['id'], 'add');
      expect(json.first['name'], 'add_method');
      final args = json.first['args'] as List;
      expect(args.map((a) => (a as Map)['name']), ['file', 'class', 'method']);
      expect((args.first as Map)['required'], isTrue);
      expect((args.first as Map)['inputKind'], 'file');
      expect((args.first as Map)['contextKey'], CodemodContextKey.file);
      expect((args.last as Map)['options'], ['reset', 'increment']);
    });

    test('derives template previews from template-path create operations', () {
      final recipe = CodemodRecipe(
        name: 'scaffold',
        args: [CodemodArg.required('feature')],
        operations: [
          CreateFileOperation.templatePath(
            pathTemplate: 'lib/{{feature:snake}}.dart',
            previewLabel: 'Feature file',
            template: const CodemodTemplate.inline(
              'class {{feature:pascal}} {}\n',
            ),
          ),
        ],
      );
      final json = RecipeSchema.recipeToJson(recipe);
      final previews = json['previewTemplates'] as List;
      expect(previews, hasLength(1));
      expect((previews.first as Map)['label'], 'Feature file');
      expect((previews.first as Map)['path'], 'lib/{{feature:snake}}.dart');
      expect(
        (previews.first as Map)['content'],
        contains('{{feature:pascal}}'),
      );
    });
  });

  group('CodemodHost.fromList', () {
    test('keys recipes by recipe name', () async {
      final host = CodemodHost.fromList([_addMethodRecipe()]);
      final response = await host.dispatch({'command': 'list'});
      final recipes = response['recipes'] as List;
      expect((recipes.first as Map)['id'], 'add_method');
    });
  });

  group('PatchSelector', () {
    final base = PatchFileChange(
      path: 'a.dart',
      source: 'abc',
      patches: [
        SourcePatch(0, 0, 'X', description: 'first'),
        SourcePatch(1, 0, 'Y', description: 'second'),
        SourcePatch(2, 0, 'Z', description: 'third'),
      ],
    );

    test('selectPatches keeps only requested indices in order', () {
      final filtered = PatchSelector.selectPatches(base, [2, 0]);
      expect(filtered.patches.map((p) => p.replacement), ['X', 'Z']);
    });

    test('apply drops files marked not included', () {
      final result = PatchSelector.apply(
        [base],
        {'a.dart': const FileSelection(include: false)},
      );
      expect(result, isEmpty);
    });

    test('apply keeps all patches when no selection given', () {
      final result = PatchSelector.apply([base], const {});
      expect(result, hasLength(1));
      expect((result.first as PatchFileChange).patches, hasLength(3));
    });

    test('apply drops change when selected patch list is empty', () {
      final result = PatchSelector.apply(
        [base],
        {'a.dart': const FileSelection(patchIndices: [])},
      );
      expect(result, isEmpty);
    });
  });

  group('CodemodHost', () {
    test('list returns registered recipes', () async {
      final host = CodemodHost({'add': _addMethodRecipe()});
      final response = await host.dispatch({'command': 'list'});
      expect(response['ok'], isTrue);
      expect(response['recipes'], hasLength(1));
      final recipe = (response['recipes'] as List).first as Map;
      expect(recipe['templatesLoaded'], isFalse);
      expect(response['_hostMetrics'], isA<Map>());
    });

    test('describe returns fully hydrated recipe schema', () async {
      final host = CodemodHost({'add': _addMethodRecipe()});
      final response = await host.dispatch({
        'command': 'describe',
        'recipe': 'add',
      });
      expect(response['ok'], isTrue);
      final recipe = response['recipe'] as Map;
      expect(recipe['id'], 'add');
      expect(recipe['templatesLoaded'], isTrue);
    });

    test('preview returns structured diff data', () async {
      final dir = await Directory.systemTemp.createTemp('codemod_host_test');
      addTearDown(() => dir.delete(recursive: true));
      final file = File('${dir.path}/counter.dart')
        ..writeAsStringSync('class Counter {\n  int value = 0;\n}\n');

      final host = CodemodHost({'add': _addMethodRecipe()});
      final response = await host.dispatch({
        'command': 'preview',
        'recipe': 'add',
        'args': {'file': file.path, 'class': 'Counter', 'method': 'reset'},
      });

      expect(response['ok'], isTrue);
      final files = response['files'] as List;
      expect(files, hasLength(1));
      final entry = files.first as Map;
      expect(entry['kind'], 'edit');
      expect(entry.containsKey('original'), isFalse);
      expect(entry.containsKey('modified'), isFalse);
      expect(entry['patches'], hasLength(1));
      final patch = (entry['patches'] as List).first as Map;
      expect(patch['replacementPreview'], isA<String>());
      expect(response['_timingsMs'], isA<Map>());
      expect(response['_hostMetrics'], isA<Map>());
    });

    test('diff returns full file data for a preview path', () async {
      final dir = await Directory.systemTemp.createTemp('codemod_host_diff');
      addTearDown(() => dir.delete(recursive: true));
      final file = File('${dir.path}/counter.dart')
        ..writeAsStringSync('class Counter {\n  int value = 0;\n}\n');

      final host = CodemodHost({'add': _addMethodRecipe()});
      final response = await host.dispatch({
        'command': 'diff',
        'recipe': 'add',
        'path': file.path,
        'args': {'file': file.path, 'class': 'Counter', 'method': 'reset'},
      });

      expect(response['ok'], isTrue);
      final entry = response['file'] as Map;
      expect(entry['original'], contains('class Counter'));
      expect(entry['modified'], contains('void reset()'));
      expect(response['_timingsMs'], isA<Map>());
      expect(response['_hostMetrics'], isA<Map>());
    });

    test('preview reports missing required arguments', () async {
      final host = CodemodHost({'add': _addMethodRecipe()});
      final response = await host.dispatch({
        'command': 'preview',
        'recipe': 'add',
        'args': {'file': 'x.dart'},
      });
      expect(response['ok'], isFalse);
      expect(response['error'], contains('Missing required arguments'));
    });

    test('apply writes only selected patches to disk', () async {
      final dir = await Directory.systemTemp.createTemp('codemod_host_apply');
      addTearDown(() => dir.delete(recursive: true));
      final file = File('${dir.path}/counter.dart')
        ..writeAsStringSync('class Counter {\n  int value = 0;\n}\n');

      final host = CodemodHost({'add': _addMethodRecipe()});
      await host.dispatch({
        'command': 'preview',
        'recipe': 'add',
        'args': {'file': file.path, 'class': 'Counter', 'method': 'reset'},
      });
      final response = await host.dispatch({
        'command': 'apply',
        'recipe': 'add',
        'args': {'file': file.path, 'class': 'Counter', 'method': 'reset'},
        'selection': {
          'files': {
            file.path: {
              'include': true,
              'patches': [0],
            },
          },
        },
      });

      expect(response['ok'], isTrue);
      expect(response['applied'], [file.path]);
      expect(file.readAsStringSync(), contains('void reset()'));
      expect(response['_timingsMs'], isA<Map>());
      final timings = response['_timingsMs'] as Map;
      expect(timings['reusedPreviewCache'], 1);
      expect(response['_hostMetrics'], isA<Map>());
    });

    test('unknown command returns error', () async {
      final host = CodemodHost(const {});
      final response = await host.dispatch({'command': 'nope'});
      expect(response['ok'], isFalse);
    });
  });
}
