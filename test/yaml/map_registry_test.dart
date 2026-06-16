import 'dart:io';

import 'package:codemod_recipe/codemod_recipe.dart';
import 'package:test/test.dart';

void main() {
  test('loads maps by id', () async {
    final workspace = await Directory.systemTemp.createTemp('codemod_maps_ok_');
    addTearDown(() => workspace.deleteSync(recursive: true));

    final mapsDir = Directory('${workspace.path}/.codemod/maps');
    await mapsDir.create(recursive: true);

    await File('${mapsDir.path}/column_type.yaml').writeAsString('''
id: columnType
entries:
  int: intColumn
  String: textColumn
''');

    final result = YamlMapRegistry.load(
      workspaceRoot: workspace.path,
      mapsDirectoryPath: mapsDir.path,
    );

    expect(
      result.diagnostics.where((d) => d.severity == DiagnosticSeverity.error),
      isEmpty,
    );
    expect(result.mapsById['columnType']!['int'], 'intColumn');
  });

  test('reports duplicate map ids as error', () async {
    final workspace = await Directory.systemTemp.createTemp(
      'codemod_maps_dup_',
    );
    addTearDown(() => workspace.deleteSync(recursive: true));

    final mapsDir = Directory('${workspace.path}/.codemod/maps');
    await mapsDir.create(recursive: true);

    await File('${mapsDir.path}/a.yaml').writeAsString('''
id: columnType
entries: { int: intColumn }
''');
    await File('${mapsDir.path}/b.yaml').writeAsString('''
id: columnType
entries: { String: textColumn }
''');

    final result = YamlMapRegistry.load(
      workspaceRoot: workspace.path,
      mapsDirectoryPath: mapsDir.path,
    );

    expect(
      result.diagnostics.any((d) => d.code == 'E_DUPLICATE_MAP_ID'),
      isTrue,
    );
    expect(result.mapsById.containsKey('columnType'), isFalse);
  });

  test('warns when a template references a missing map id', () async {
    final workspace = await Directory.systemTemp.createTemp(
      'codemod_maps_warn_',
    );
    addTearDown(() => workspace.deleteSync(recursive: true));

    await Directory(
      '${workspace.path}/.codemod/recipes',
    ).create(recursive: true);
    await File(
      '${workspace.path}/.codemod/recipes/uses_map.yaml',
    ).writeAsString('''
dslVersion: 1
id: uses_map
name: uses_map
args:
  - name: file
    required: true
    inputKind: file
steps:
  - edit:
      path: \"{{file}}\"
      steps:
        - insert:
            at: \"class:Settings @ member:last\"
            text: |
              final x = {{\$map 'missing' type}};
''');

    final result = YamlRecipeRegistry.load(
      HostConfig(
        workspaceRoot: workspace.path,
        recipesDirectory: '.codemod/recipes',
        mapsDirectory: '.codemod/maps',
      ),
    );

    expect(
      result.diagnostics.any((d) => d.code == 'W_MAP_ID_NOT_FOUND'),
      isTrue,
    );
  });
}
