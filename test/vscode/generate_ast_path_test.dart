import 'dart:io';

import 'package:codemod_recipe/codemod_recipe_vscode.dart';
import 'package:test/test.dart';

void main() {
  group('CodemodHost generateAstPath', () {
    late Directory workspace;
    late File dartFile;
    late String dartPath;

    setUp(() async {
      workspace = await Directory.systemTemp.createTemp('gen_ast_path_');
      dartFile = File('${workspace.path}/lib/settings.dart')
        ..createSync(recursive: true);
      dartFile.writeAsStringSync('''
class Settings {
  final int count = 0;

  void update() {}
}
''');
      dartPath = dartFile.path;
    });

    tearDown(() => workspace.deleteSync(recursive: true));

    test('returns navigate steps for a valid offset', () async {
      final offset = dartFile.readAsStringSync().indexOf('count');
      final host = CodemodHost.fromConfig(
        HostConfig(workspaceRoot: workspace.path, codemodRoot: '.codemod'),
      );

      final response = await host.dispatch({
        'command': 'generateAstPath',
        'path': dartPath,
        'offset': offset,
      });

      expect(response['ok'], isTrue);
      final path = response['path'] as Map<String, Object?>;
      final navigate = path['navigate'] as List;
      expect(navigate, isNotEmpty);
      expect(
        navigate.map((step) => (step as Map)['kind']),
        containsAll(['classDecl', 'field']),
      );
      expect(path['anchor'], isNotNull);
    });

    test('returns error for invalid offset', () async {
      final host = CodemodHost.fromConfig(
        HostConfig(workspaceRoot: workspace.path, codemodRoot: '.codemod'),
      );

      final response = await host.dispatch({
        'command': 'generateAstPath',
        'path': dartPath,
        'offset': 100_000,
      });

      expect(response['ok'], isFalse);
      expect(response['error'], contains('No AST node found'));
    });

    test('returns error when path or offset missing', () async {
      final host = CodemodHost.fromConfig(
        HostConfig(workspaceRoot: workspace.path, codemodRoot: '.codemod'),
      );

      final response = await host.dispatch({
        'command': 'generateAstPath',
        'path': dartPath,
      });

      expect(response['ok'], isFalse);
      expect(response['error'], contains('Missing path or offset'));
    });
  });
}
