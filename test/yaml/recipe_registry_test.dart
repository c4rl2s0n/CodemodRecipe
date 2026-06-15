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
      HostConfig(
        workspaceRoot: workspace.path,
        recipesDirectory: '.codemod/recipes',
      ),
    );

    expect(result.diagnostics.where((d) => d.severity == DiagnosticSeverity.error), isEmpty);
    expect(result.recipes['add_log_line'], isNotNull);
    expect(result.recipes['add_log_line']!.operations.length, 1);
  });

  test('compiles addField yaml recipe', () async {
    final workspace = await Directory.systemTemp.createTemp('codemod_yaml_field_');
    addTearDown(() => workspace.deleteSync(recursive: true));

    await _copyFile(
      'test/fixtures/yaml_recipes/add_counter_field.yaml',
      '${workspace.path}/.codemod/recipes/add_counter_field.yaml',
    );

    final result = YamlRecipeRegistry.load(
      HostConfig(
        workspaceRoot: workspace.path,
        recipesDirectory: '.codemod/recipes',
      ),
    );

    expect(result.diagnostics.where((d) => d.severity == DiagnosticSeverity.error), isEmpty);
    expect(result.recipes['add_counter_field'], isNotNull);
    final operation = result.recipes['add_counter_field']!.operations.single;
    expect(operation, isA<EditDartFileOperation>());
  });

  test('compiles addConstructorParam yaml recipe', () async {
    final workspace = await Directory.systemTemp.createTemp('codemod_yaml_ctor_');
    addTearDown(() => workspace.deleteSync(recursive: true));

    await _copyFile(
      'test/fixtures/yaml_recipes/add_constructor_param.yaml',
      '${workspace.path}/.codemod/recipes/add_constructor_param.yaml',
    );

    final result = YamlRecipeRegistry.load(
      HostConfig(
        workspaceRoot: workspace.path,
        recipesDirectory: '.codemod/recipes',
      ),
    );

    expect(result.diagnostics.where((d) => d.severity == DiagnosticSeverity.error), isEmpty);
    expect(result.recipes['add_constructor_param'], isNotNull);
    final operation = result.recipes['add_constructor_param']!.operations.single;
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
      HostConfig(
        workspaceRoot: workspace.path,
        recipesDirectory: '.codemod/recipes',
      ),
    );

    expect(result.diagnostics.where((d) => d.severity == DiagnosticSeverity.error), isEmpty);
    final recipe = result.recipes['with_build_runner'];
    expect(recipe, isNotNull);
    expect(
      recipe!.postExecution.any((action) => action is BuildRunnerPostExecution),
      isTrue,
    );
  });

  test('reports duplicate recipe ids', () async {
    final workspace = await Directory.systemTemp.createTemp('codemod_yaml_dup_');
    addTearDown(() => workspace.deleteSync(recursive: true));

    await _copyTree(
      'test/fixtures/yaml_recipes',
      '${workspace.path}/.codemod/recipes',
    );

    final result = YamlRecipeRegistry.load(
      HostConfig(
        workspaceRoot: workspace.path,
        recipesDirectory: '.codemod/recipes',
      ),
    );

    expect(result.recipes.containsKey('add_log_line'), isFalse);
    expect(
      result.diagnostics.any((item) => item.code == 'E_DUPLICATE_RECIPE_ID'),
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
