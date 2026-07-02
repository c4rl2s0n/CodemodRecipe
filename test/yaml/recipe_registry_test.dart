import 'dart:io';

import 'package:codemod_recipe/codemod_recipe.dart';
import 'package:test/test.dart';

void main() {
  test('loads valid yaml recipe', () async {
    final workspace = await Directory.systemTemp.createTemp('codemod_yaml_ok_');
    addTearDown(() => workspace.deleteSync(recursive: true));

    await _copyFile(
      'test/fixtures/yaml_recipes/add_log_line.yaml',
      '${workspace.path}/.codemod/recipes/add_log_line.yaml',
    );

    final result = YamlRecipeRegistry.load(
      HostConfig(workspaceRoot: workspace.path, codemodRoot: '.codemod'),
    );

    expect(
      result.diagnostics.where((d) => d.severity == DiagnosticSeverity.error),
      isEmpty,
    );
    expect(result.recipes['add_log_line'], isNotNull);
    expect(result.recipes['add_log_line']!.operations.length, 1);
  });

  test('compiles insert-based yaml recipe', () async {
    final workspace = await Directory.systemTemp.createTemp(
      'codemod_yaml_field_',
    );
    addTearDown(() => workspace.deleteSync(recursive: true));

    await _copyFile(
      'test/fixtures/yaml_recipes/add_counter_field.yaml',
      '${workspace.path}/.codemod/recipes/add_counter_field.yaml',
    );

    final result = YamlRecipeRegistry.load(
      HostConfig(workspaceRoot: workspace.path, codemodRoot: '.codemod'),
    );

    expect(
      result.diagnostics.where((d) => d.severity == DiagnosticSeverity.error),
      isEmpty,
    );
    expect(result.recipes['add_counter_field'], isNotNull);
    final operation = result.recipes['add_counter_field']!.operations.single;
    expect(operation, isA<EditDartFileOperation>());
  });

  test('compiles constructor param insertion yaml recipe', () async {
    final workspace = await Directory.systemTemp.createTemp(
      'codemod_yaml_ctor_',
    );
    addTearDown(() => workspace.deleteSync(recursive: true));

    await _copyFile(
      'test/fixtures/yaml_recipes/add_constructor_param.yaml',
      '${workspace.path}/.codemod/recipes/add_constructor_param.yaml',
    );

    final result = YamlRecipeRegistry.load(
      HostConfig(workspaceRoot: workspace.path, codemodRoot: '.codemod'),
    );

    expect(
      result.diagnostics.where((d) => d.severity == DiagnosticSeverity.error),
      isEmpty,
    );
    expect(result.recipes['add_constructor_param'], isNotNull);
    final operation =
        result.recipes['add_constructor_param']!.operations.single;
    expect(operation, isA<EditDartFileOperation>());
  });

  test('compiles buildRunner postExecution', () async {
    final workspace = await Directory.systemTemp.createTemp('codemod_yaml_br_');
    addTearDown(() => workspace.deleteSync(recursive: true));

    await _copyFile(
      'test/fixtures/yaml_recipes/with_build_runner.yaml',
      '${workspace.path}/.codemod/recipes/with_build_runner.yaml',
    );

    final result = YamlRecipeRegistry.load(
      HostConfig(workspaceRoot: workspace.path, codemodRoot: '.codemod'),
    );

    expect(
      result.diagnostics.where((d) => d.severity == DiagnosticSeverity.error),
      isEmpty,
    );
    final recipe = result.recipes['with_build_runner'];
    expect(recipe, isNotNull);
    expect(
      recipe!.postExecution.any((action) => action is ProcessPostExecution),
      isTrue,
    );
  });

  test('reports duplicate recipe ids', () async {
    final workspace = await Directory.systemTemp.createTemp(
      'codemod_yaml_dup_',
    );
    addTearDown(() => workspace.deleteSync(recursive: true));

    await _copyTree(
      'test/fixtures/yaml_recipes',
      '${workspace.path}/.codemod/recipes',
    );

    final result = YamlRecipeRegistry.load(
      HostConfig(workspaceRoot: workspace.path, codemodRoot: '.codemod'),
    );

    expect(result.recipes.containsKey('add_log_line'), isFalse);
    expect(
      result.diagnostics.any((item) => item.code == 'E_DUPLICATE_ID'),
      isTrue,
    );
  });

  test('rejects path traversal in template paths', () {
    final sandbox = PathSandbox(
      HostConfig(workspaceRoot: Directory.systemTemp.path),
    );

    expect(
      () => sandbox.resolveTemplateRelative('../outside.txt'),
      throwsA(isA<PathSandboxException>()),
    );
  });

  test('allows maps and recipes to share the same ID', () async {
    final workspace = await Directory.systemTemp.createTemp(
      'codemod_map_recipe_same_id_',
    );
    addTearDown(() => workspace.deleteSync(recursive: true));

    // Create a map with ID 'shared_id'
    await Directory('${workspace.path}/.codemod/maps').create(recursive: true);
    await File('${workspace.path}/.codemod/maps/shared_id.yaml').writeAsString(
      '''
id: shared_id
entries:
  key1: value1
  key2: value2
''',
    );

    // Create a recipe with the same ID 'shared_id'
    await Directory(
      '${workspace.path}/.codemod/recipes',
    ).create(recursive: true);
    await File(
      '${workspace.path}/.codemod/recipes/shared_id.yaml',
    ).writeAsString('''
dslVersion: 1
id: shared_id
name: Shared ID Recipe
args:
  - name: file
    required: true
    inputKind: file
steps:
  - edit:
      path: "{{file}}"
      steps:
        - insert:
            at: function:main @ body:end
            text: "// Shared ID recipe"
''');

    final result = YamlRecipeRegistry.load(
      HostConfig(workspaceRoot: workspace.path, codemodRoot: '.codemod'),
    );

    // Should have no duplicate ID errors
    expect(
      result.diagnostics.any((item) => item.code == 'E_DUPLICATE_ID'),
      isFalse,
      reason: 'Maps and recipes should not conflict when sharing the same ID',
    );

    // Should have no duplicate map ID errors
    expect(
      result.diagnostics.any((item) => item.code == 'E_DUPLICATE_MAP_ID'),
      isFalse,
      reason: 'Should not have duplicate map IDs',
    );

    // Recipe should be loaded successfully
    expect(result.recipes.containsKey('shared_id'), isTrue);
    expect(result.recipes['shared_id']!.name, 'Shared ID Recipe');
  });

  test('compiles remove-based yaml recipe', () async {
    final workspace = await Directory.systemTemp.createTemp(
      'codemod_yaml_remove_',
    );
    addTearDown(() => workspace.deleteSync(recursive: true));

    await _copyFile(
      'test/fixtures/yaml_recipes/remove_counter_field.yaml',
      '${workspace.path}/.codemod/recipes/remove_counter_field.yaml',
    );

    final result = YamlRecipeRegistry.load(
      HostConfig(workspaceRoot: workspace.path, codemodRoot: '.codemod'),
    );

    expect(
      result.diagnostics.where((d) => d.severity == DiagnosticSeverity.error),
      isEmpty,
    );
    expect(result.recipes['remove_counter_field'], isNotNull);
    final operation = result.recipes['remove_counter_field']!.operations.single;
    expect(operation, isA<EditDartFileOperation>());
  });

  test('reports duplicate map ids', () async {
    final workspace = await Directory.systemTemp.createTemp('codemod_dup_map_');
    addTearDown(() => workspace.deleteSync(recursive: true));

    // Create two maps with the same ID
    await Directory('${workspace.path}/.codemod/maps').create(recursive: true);
    await File('${workspace.path}/.codemod/maps/dup_map.yaml').writeAsString('''
id: duplicate_map
entries:
  key1: value1
''');
    await File('${workspace.path}/.codemod/maps/dup_map2.yaml').writeAsString(
      '''
id: duplicate_map
entries:
  key2: value2
''',
    );

    final result = YamlRecipeRegistry.load(
      HostConfig(workspaceRoot: workspace.path, codemodRoot: '.codemod'),
    );

    // Should report duplicate map ID error
    expect(
      result.diagnostics.any((item) => item.code == 'E_DUPLICATE_MAP_ID'),
      isTrue,
      reason: 'Should detect duplicate map IDs',
    );
  });
}

Future<void> _copyFile(String source, String destination) async {
  final file = File(source);
  await File(destination).parent.create(recursive: true);
  await file.copy(destination);
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
