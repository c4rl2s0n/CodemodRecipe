import 'dart:io';

import 'package:codemod_recipe/codemod_recipe.dart';
import 'package:codemod_recipe/codemod_recipe_vscode.dart';
import 'package:codemod_recipe/src/yaml/patch_transform.dart';
import 'package:test/test.dart';

CodemodRecipe _removeCountRecipe(String filePath) {
  return CodemodRecipe(
    name: 'remove_count',
    operations: [
      EditDartFileOperation(
        path: (_) => filePath,
        transforms: (_) => [
          AstPathPatchTransform.remove(
            target: const AstPathPatchTarget(
              navigate: [
                NavigateStep(NavigateKind.classDecl, name: 'Settings'),
                NavigateStep(NavigateKind.field, name: 'count'),
              ],
            ),
          ),
        ],
      ),
    ],
  );
}

void main() {
  group('CodemodHost inline + previewToken', () {
    late Directory workspace;
    late File settingsFile;
    late String settingsPath;

    setUp(() async {
      workspace = await Directory.systemTemp.createTemp('host_inline_');
      settingsFile = File('${workspace.path}/lib/settings.dart')
        ..createSync(recursive: true);
      settingsFile.writeAsStringSync('''
class Settings {
  final int count = 0;
  final String name = 'x';

  void update() {
    print('hi');
  }
}
''');

      settingsPath = settingsFile.path;

      await Directory('${workspace.path}/.codemod/recipes').create(recursive: true);
      await File('test/fixtures/yaml_recipes/add_log_line.yaml').copy(
        '${workspace.path}/.codemod/recipes/add_log_line.yaml',
      );
    });

    tearDown(() => workspace.deleteSync(recursive: true));

    CodemodHost yamlHost() => CodemodHost.fromConfig(
      HostConfig(workspaceRoot: workspace.path, codemodRoot: '.codemod'),
    );

    test('preview returns previewToken for registered dart recipe', () async {
      final host = CodemodHost({'remove_count': _removeCountRecipe(settingsPath)});
      final response = await host.dispatch({
        'command': 'preview',
        'recipe': 'remove_count',
      });

      expect(response['ok'], isTrue);
      expect(response['previewToken'], isA<String>());
      expect((response['previewToken'] as String).length, greaterThan(20));
    });

    test('apply rejects missing previewToken', () async {
      final host = CodemodHost({'remove_count': _removeCountRecipe(settingsPath)});
      final response = await host.dispatch({
        'command': 'apply',
        'recipe': 'remove_count',
      });

      expect(response['ok'], isFalse);
      expect(response['error'], contains('previewToken'));
    });

    test('inline remove recipe previews and applies with token', () async {
      final h = yamlHost();
      final inlineRecipe = {
        'id': '__inline_remove_count',
        'steps': [
          {
            'edit': {
              'path': settingsPath,
              'steps': [
                {
                  'remove': {
                    'at': [
                      {'class': 'Settings'},
                      {'field': 'count'},
                    ],
                  },
                },
              ],
            },
          },
        ],
      };

      final preview = await h.dispatch({
        'command': 'preview',
        'inlineRecipe': inlineRecipe,
      });

      expect(preview['ok'], isTrue);
      final token = preview['previewToken'] as String;

      final apply = await h.dispatch({
        'command': 'apply',
        'previewToken': token,
        'inlineRecipe': inlineRecipe,
      });

      expect(apply['ok'], isTrue);
      final content = settingsFile.readAsStringSync();
      expect(content, isNot(contains('final int count')));
      expect(content, contains('final String name'));
    });

    test('apply rejects stale previewToken after file changes', () async {
      final host = CodemodHost({'remove_count': _removeCountRecipe(settingsPath)});
      final preview = await host.dispatch({
        'command': 'preview',
        'recipe': 'remove_count',
      });
      final token = preview['previewToken'] as String;

      settingsFile.writeAsStringSync('// mutated\n${settingsFile.readAsStringSync()}');

      final apply = await host.dispatch({
        'command': 'apply',
        'recipe': 'remove_count',
        'previewToken': token,
      });

      expect(apply['ok'], isFalse);
      expect(responseError(apply), contains('Stale previewToken'));
    });

    test('remove step is idempotent on second preview', () async {
      final h = yamlHost();
      final inlineRecipe = {
        'id': '__inline_remove_count',
        'steps': [
          {
            'edit': {
              'path': settingsPath,
              'steps': [
                {
                  'remove': {
                    'at': [
                      {'class': 'Settings'},
                      {'field': 'count'},
                    ],
                  },
                },
              ],
            },
          },
        ],
      };

      final first = await h.dispatch({
        'command': 'preview',
        'inlineRecipe': inlineRecipe,
      });
      expect(first['ok'], isTrue);
      expect((first['files'] as List).isNotEmpty, isTrue);

      final apply = await h.dispatch({
        'command': 'apply',
        'previewToken': first['previewToken'],
        'inlineRecipe': inlineRecipe,
      });
      expect(apply['ok'], isTrue);

      final second = await h.dispatch({
        'command': 'preview',
        'inlineRecipe': inlineRecipe,
      });
      expect(second['ok'], isTrue);
      expect((second['files'] as List), isEmpty);
    });

    test('insert path string with embedded anchor still compiles', () async {
      final result = YamlRecipeRegistry.load(
        HostConfig(workspaceRoot: workspace.path, codemodRoot: '.codemod'),
      );
      expect(
        result.diagnostics.where((d) => d.severity == DiagnosticSeverity.error),
        isEmpty,
      );
      expect(result.recipes['add_log_line'], isNotNull);
    });
  });
}

String responseError(Map<String, Object?> response) =>
    response['error']?.toString() ?? '';
