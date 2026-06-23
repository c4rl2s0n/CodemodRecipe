import 'dart:io';

import 'package:codemod_recipe/codemod_recipe_vscode.dart';
import 'package:test/test.dart';

void main() {
  test('validate command returns diagnostics for duplicate ids', () async {
    final workspace = await Directory.systemTemp.createTemp('codemod_host_');
    addTearDown(() => workspace.deleteSync(recursive: true));

    await _copyTree(
      'test/fixtures/yaml_recipes',
      '${workspace.path}/.codemod/recipes',
    );

    final config = HostConfig(
      workspaceRoot: workspace.path,
      codemodRoot: '.codemod',
    );
    final host = CodemodHost.fromConfig(config);

    final response = await host.dispatch({'command': 'validate'});
    expect(response['ok'], isFalse);
    expect(response['diagnostics'], isA<List>());
    final diagnostics = response['diagnostics'] as List;
    expect(
      diagnostics.any((item) => (item as Map)['code'] == 'E_DUPLICATE_ID'),
      isTrue,
    );
  });

  test('reload refreshes registry from disk', () async {
    final workspace = await Directory.systemTemp.createTemp(
      'codemod_host_reload_',
    );
    addTearDown(() => workspace.deleteSync(recursive: true));

    await _copyFile(
      'test/fixtures/yaml_recipes/add_log_line.yaml',
      '${workspace.path}/.codemod/recipes/add_log_line.yaml',
    );

    final host = CodemodHost.fromConfig(
      HostConfig(workspaceRoot: workspace.path, codemodRoot: '.codemod'),
    );

    final initial = await host.dispatch({'command': 'list'});
    expect((initial['recipes'] as List).length, 1);

    await _copyFile(
      'test/fixtures/yaml_recipes/duplicate_id.yaml',
      '${workspace.path}/.codemod/recipes/duplicate_id.yaml',
    );

    final reloaded = await host.dispatch({'command': 'reload'});
    expect(reloaded['ok'], isTrue);
    expect(
      (reloaded['diagnostics'] as List).any(
        (item) => (item as Map)['code'] == 'E_DUPLICATE_ID',
      ),
      isTrue,
    );
  });
}

Future<void> _copyFile(String source, String destination) async {
  await File(destination).parent.create(recursive: true);
  await File(source).copy(destination);
}

Future<void> _copyTree(String source, String destination) async {
  final dir = Directory(source);
  await Directory(destination).create(recursive: true);
  await for (final entity in dir.list(recursive: true)) {
    if (entity is File) {
      final relative = entity.path.substring(dir.path.length + 1);
      await _copyFile(entity.path, '$destination/$relative');
    }
  }
}
