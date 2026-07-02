import 'package:yaml/yaml.dart';

import '../core/context.dart';
import '../core/errors.dart';
import '../core/operation.dart';
import '../core/post_execution.dart';
import '../core/recipe.dart';
import '../core/template.dart';
import '../core/transform.dart';
import 'diagnostics.dart';
import 'host_config.dart';
import 'insert_transform.dart';
import 'patch_transform.dart';
import 'path_sandbox.dart';
import 'schema_validator.dart';
import '../ast_path/ast_path.dart';

/// Raw parse result for one YAML recipe file before registry linking.
class YamlRecipeDefinition {
  /// Creates a parsed definition.
  const YamlRecipeDefinition({
    required this.id,
    required this.filePath,
    required this.document,
    this.preferences,
  });

  /// Recipe id from the YAML file.
  final String id;

  /// Source file path.
  final String filePath;

  /// Parsed YAML root mapping.
  final YamlMap document;

  /// Optional per-recipe preference overrides.
  final CodemodPreferences? preferences;
}

/// Compiles [YamlRecipeDefinition] values into executable recipes.
class YamlRecipeCompiler {
  /// Creates a compiler with registry context.
  YamlRecipeCompiler({
    required this.config,
    required this.definitionsById,
    required this.dartRecipes,
    required this.mapsById,
  });

  /// Host configuration.
  final HostConfig config;

  /// Parsed YAML definitions keyed by id.
  final Map<String, YamlRecipeDefinition> definitionsById;

  /// Dart-registered recipes available for reference.
  final Map<String, CodemodRecipe> dartRecipes;

  /// Globally loaded YAML maps by id.
  final Map<String, Map<String, String>> mapsById;

  /// Compiles [definition] into a [CodemodRecipe].
  CompileResult compile(YamlRecipeDefinition definition) {
    final diagnostics = <RecipeDiagnostic>[];
    final doc = definition.document;
    final sandbox = PathSandbox(config);

    try {
      final mergedMaps = _mergeMaps(
        global: mapsById,
        inline: _parseInlineMaps(doc['maps'], definition.filePath, diagnostics),
      );
      final templateEnvironment = TemplateEnvironment(maps: mergedMaps);

      final name = _stringField(doc, 'name') ?? definition.id;
      final description = _stringField(doc, 'description') ?? '';
      final args = _parseArgs(doc['args'], definition.filePath, diagnostics);
      final operations = _parseSteps(
        doc['steps'],
        definition.filePath,
        sandbox,
        diagnostics,
        templateEnvironment,
      );
      final postExecution = _parsePostExecution(
        doc['postExecution'],
        definition.filePath,
        sandbox,
        diagnostics,
      );

      if (diagnostics.any(
        (item) => item.severity == DiagnosticSeverity.error,
      )) {
        return CompileResult(diagnostics: diagnostics);
      }

      return CompileResult(
        recipe: CodemodRecipe(
          name: name,
          description: description,
          args: args,
          operations: operations,
          postExecution: postExecution,
        ),
        diagnostics: diagnostics,
      );
    } catch (error) {
      diagnostics.add(
        RecipeDiagnostics.compileError('$error', definition.filePath),
      );
      return CompileResult(diagnostics: diagnostics);
    }
  }

  List<CodemodArgDescriptor> _parseArgs(
    Object? value,
    String filePath,
    List<RecipeDiagnostic> diagnostics,
  ) {
    if (value == null) return const [];
    if (value is! YamlList) {
      diagnostics.add(_schemaError('args must be a list', filePath));
      return const [];
    }

    final args = <CodemodArgDescriptor>[];
    for (final entry in value) {
      if (entry is! YamlMap) {
        diagnostics.add(
          _schemaError('Each args entry must be a map', filePath),
        );
        continue;
      }

      final name = _stringField(entry, 'name');
      if (name == null) {
        diagnostics.add(_schemaError('Arg missing name', filePath));
        continue;
      }

      final required = entry['required'] == true;
      final help = _stringField(entry, 'help');
      final abbr = _stringField(entry, 'abbr');
      final contextKey = _stringField(entry, 'contextKey');
      final inputKind = _parseInputKind(_stringField(entry, 'inputKind'));
      final options = _stringList(entry['options']);
      final allowCustomValue = entry['allowCustomValue'] == true;
      final defaultsTo = _stringField(entry, 'defaultsTo');

      if (required) {
        args.add(
          CodemodArg<String>.required(
            name,
            help: help,
            abbr: abbr,
            inputKind: inputKind,
            contextKey: contextKey,
            options: options,
            allowCustomValue: allowCustomValue,
          ),
        );
      } else {
        args.add(
          CodemodArg<String>.optional(
            name,
            help: help,
            abbr: abbr,
            defaultsTo: defaultsTo,
            inputKind: inputKind,
            contextKey: contextKey,
            options: options,
            allowCustomValue: allowCustomValue,
          ),
        );
      }
    }

    return args;
  }

  List<CodemodOperation> _parseSteps(
    Object? value,
    String filePath,
    PathSandbox sandbox,
    List<RecipeDiagnostic> diagnostics,
    TemplateEnvironment templateEnvironment,
  ) {
    if (value == null) return const [];
    if (value is! YamlList) {
      diagnostics.add(_schemaError('steps must be a list', filePath));
      return const [];
    }

    final operations = <CodemodOperation>[];
    for (final entry in value) {
      if (entry is! YamlMap) {
        diagnostics.add(_schemaError('Each step must be a map', filePath));
        continue;
      }

      if (entry.containsKey('recipe')) {
        final recipeId = _stringField(entry, 'recipe');
        if (recipeId == null) {
          diagnostics.add(_schemaError('recipe step missing id', filePath));
          continue;
        }
        final referenced = _resolveReferencedRecipe(
          recipeId,
          filePath,
          diagnostics,
        );
        if (referenced != null) {
          operations.addAll(referenced.operations);
        }
        continue;
      }

      if (entry.containsKey('edit')) {
        final edit = entry['edit'];
        if (edit is! YamlMap) {
          diagnostics.add(_schemaError('edit must be a map', filePath));
          continue;
        }
        final pathTemplate = _stringField(edit, 'path');
        if (pathTemplate == null) {
          diagnostics.add(_schemaError('edit.path is required', filePath));
          continue;
        }

        final transforms = _parseEditSteps(
          edit['steps'],
          filePath,
          diagnostics,
          templateEnvironment,
        );

        operations.add(
          EditDartFileOperation(
            path: (context) => context.render(pathTemplate),
            transforms: (_) => transforms,
          ),
        );
        continue;
      }

      if (entry.containsKey('create')) {
        final create = entry['create'];
        if (create is! YamlMap) {
          diagnostics.add(_schemaError('create must be a map', filePath));
          continue;
        }

        final pathTemplate = _stringField(create, 'path');
        if (pathTemplate == null) {
          diagnostics.add(_schemaError('create.path is required', filePath));
          continue;
        }

        final template = _parseTemplate(
          create,
          filePath,
          sandbox,
          diagnostics,
          templateEnvironment,
        );
        if (template == null) continue;

        final ifExists = _parseIfExists(_stringField(create, 'ifExists'));
        final format = create['format'] != false;

        operations.add(
          CreateFileOperation.templatePath(
            pathTemplate: pathTemplate,
            template: template,
            ifExists: ifExists,
            format: format,
          ),
        );
      }
    }

    return operations;
  }

  List<CodeTransform> _parseEditSteps(
    Object? value,
    String filePath,
    List<RecipeDiagnostic> diagnostics,
    TemplateEnvironment templateEnvironment,
  ) {
    if (value is! YamlList) {
      diagnostics.add(_schemaError('edit.steps must be a list', filePath));
      return const [];
    }

    final transforms = <CodeTransform>[];
    for (final entry in value) {
      if (entry is! YamlMap) {
        diagnostics.add(_schemaError('Each edit step must be a map', filePath));
        continue;
      }

      if (entry.containsKey('insert')) {
        final transform = _parseInsertStep(
          entry['insert'],
          filePath,
          diagnostics,
          templateEnvironment,
        );
        if (transform != null) transforms.add(transform);
        continue;
      }

      if (entry.containsKey('remove')) {
        final transform = _parseRemoveStep(
          entry['remove'],
          filePath,
          diagnostics,
        );
        if (transform != null) transforms.add(transform);
        continue;
      }

      if (entry.containsKey('replace')) {
        final transform = _parseReplaceStep(
          entry['replace'],
          filePath,
          diagnostics,
          templateEnvironment,
        );
        if (transform != null) transforms.add(transform);
        continue;
      }

      diagnostics.add(_schemaError('Unsupported edit step', filePath));
    }

    return transforms;
  }

  CodeTransform? _parseInsertStep(
    Object? insert,
    String filePath,
    List<RecipeDiagnostic> diagnostics,
    TemplateEnvironment templateEnvironment,
  ) {
    if (insert is! YamlMap) {
      diagnostics.add(_schemaError('insert must be a map', filePath));
      return null;
    }

    final path = _parseInsertPath(insert, filePath, diagnostics);
    final text = _stringField(insert, 'text');
    if (path == null || text == null) return null;

    _warnOnMissingMapIds(
      template: text,
      filePath: filePath,
      diagnostics: diagnostics,
      mapsById: templateEnvironment.maps,
    );

    return AstPathInsertTransform(
      path: path,
      template: CodemodTemplate.inline(text, environment: templateEnvironment),
    );
  }

  CodeTransform? _parseRemoveStep(
    Object? remove,
    String filePath,
    List<RecipeDiagnostic> diagnostics,
  ) {
    if (remove is! YamlMap) {
      diagnostics.add(_schemaError('remove must be a map', filePath));
      return null;
    }

    final target = _parsePatchTarget(
      remove,
      filePath,
      diagnostics,
      anchorRequired: false,
    );
    if (target == null) return null;

    return AstPathPatchTransform.remove(target: target);
  }

  CodeTransform? _parseReplaceStep(
    Object? replace,
    String filePath,
    List<RecipeDiagnostic> diagnostics,
    TemplateEnvironment templateEnvironment,
  ) {
    if (replace is! YamlMap) {
      diagnostics.add(_schemaError('replace must be a map', filePath));
      return null;
    }

    final target = _parsePatchTarget(
      replace,
      filePath,
      diagnostics,
      anchorRequired: false,
    );
    final text = _stringField(replace, 'text');
    if (target == null || text == null) return null;

    _warnOnMissingMapIds(
      template: text,
      filePath: filePath,
      diagnostics: diagnostics,
      mapsById: templateEnvironment.maps,
    );

    return AstPathReplaceTransform(
      target: target,
      template: CodemodTemplate.inline(text, environment: templateEnvironment),
    );
  }

  // Edit steps: insert, remove, replace.

  static Map<String, Map<String, String>> _mergeMaps({
    required Map<String, Map<String, String>> global,
    required Map<String, Map<String, String>> inline,
  }) {
    final merged = <String, Map<String, String>>{
      for (final entry in global.entries) entry.key: Map.of(entry.value),
    };
    for (final entry in inline.entries) {
      merged.putIfAbsent(entry.key, () => <String, String>{});
      merged[entry.key]!.addAll(entry.value);
    }
    return merged;
  }

  Map<String, Map<String, String>> _parseInlineMaps(
    Object? value,
    String filePath,
    List<RecipeDiagnostic> diagnostics,
  ) {
    if (value == null) return const {};
    if (value is! YamlMap) {
      diagnostics.add(_schemaError('maps must be a map', filePath));
      return const {};
    }

    final result = <String, Map<String, String>>{};
    for (final entry in value.entries) {
      final id = entry.key.toString();
      final entriesNode = entry.value;
      if (entriesNode is! YamlMap) {
        diagnostics.add(_schemaError('maps.$id must be a map', filePath));
        continue;
      }
      result[id] = {
        for (final mapEntry in entriesNode.entries)
          mapEntry.key.toString(): mapEntry.value?.toString() ?? '',
      };
    }
    return result;
  }

  static void _warnOnMissingMapIds({
    required String template,
    required String filePath,
    required List<RecipeDiagnostic> diagnostics,
    required Map<String, Map<String, String>> mapsById,
  }) {
    // Only warn when mapId is a literal string in the template.
    // Expected syntax: `{{\$map 'mapId' key}}` or `{{\$map \"mapId\" key}}`.
    var index = 0;
    while (true) {
      final start = template.indexOf('{{\$map', index);
      if (start < 0) return;
      var i = start + '{{\$map'.length;
      while (i < template.length && template[i].trim().isEmpty) {
        i++;
      }
      if (i >= template.length) return;

      final quote = template[i];
      if (quote != '\'' && quote != '"') {
        index = i;
        continue;
      }
      i++;
      final idStart = i;
      while (i < template.length && template[i] != quote) {
        i++;
      }
      if (i >= template.length) return;
      final mapId = template.substring(idStart, i);
      index = i + 1;

      if (mapsById.containsKey(mapId)) continue;
      diagnostics.add(RecipeDiagnostics.mapIdNotFound(mapId, filePath));
    }
  }

  AstPath? _parseInsertPath(
    YamlMap step,
    String filePath,
    List<RecipeDiagnostic> diagnostics,
  ) {
    final at = step['at'];
    if (at is String && at.contains('@')) {
      return _parseStepPath(
        step,
        filePath,
        diagnostics,
        anchorRequired: false,
      );
    }

    if (!step.containsKey('anchor')) {
      diagnostics.add(
        _schemaError('insert step requires "anchor"', filePath),
      );
      return null;
    }
    return _parseStepPath(step, filePath, diagnostics, anchorRequired: true);
  }

  AstPathPatchTarget? _parsePatchTarget(
    YamlMap step,
    String filePath,
    List<RecipeDiagnostic> diagnostics, {
    required bool anchorRequired,
  }) {
    final navigate = _parseNavigateSteps(step, filePath, diagnostics);
    if (navigate == null) return null;

    final anchorValue = step['anchor'];
    if (anchorRequired && anchorValue == null) {
      diagnostics.add(
        _schemaError('edit step requires "anchor"', filePath),
      );
      return null;
    }

    Anchor? anchor;
    if (anchorValue != null) {
      try {
        anchor = parseAnchor(anchorValue.toString());
      } on AstPathParseException catch (error) {
        diagnostics.add(
          RecipeDiagnostics.astPathParseError(error.message, filePath),
        );
        return null;
      }
    }

    return AstPathPatchTarget(navigate: navigate, anchor: anchor);
  }

  List<NavigateStep>? _parseNavigateSteps(
    YamlMap step,
    String filePath,
    List<RecipeDiagnostic> diagnostics,
  ) {
    final at = step['at'];
    if (at == null) {
      diagnostics.add(_schemaError('edit step requires "at"', filePath));
      return null;
    }

    try {
      if (at is String) {
        final path = parsePathString(at);
        return path.navigate;
      }
      if (at is YamlList) {
        return [
          for (final entry in at)
            NavigateParser.parseEntry(entry),
        ];
      }
      diagnostics.add(_schemaError('"at" must be a list or path string', filePath));
      return null;
    } on AstPathParseException catch (error) {
      diagnostics.add(
        RecipeDiagnostics.astPathParseError(error.message, filePath),
      );
      return null;
    }
  }

  AstPath? _parseStepPath(
    YamlMap step,
    String filePath,
    List<RecipeDiagnostic> diagnostics, {
    required bool anchorRequired,
  }) {
    try {
      if (step.containsKey('at') && step.containsKey('anchor')) {
        final at = step['at'];
        if (at is String) {
          return parsePathString('$at @ ${step['anchor']}');
        }
        if (at is YamlList) {
          return parseStructuredPath({
            'at': at.toList(),
            'anchor': step['anchor'].toString(),
          });
        }
      }

      if (step['at'] is String) {
        return parsePathString(step['at'].toString());
      }

      if (!anchorRequired && step.containsKey('at') && step['at'] is YamlList) {
        diagnostics.add(
          _schemaError('insert step requires "anchor"', filePath),
        );
        return null;
      }

      diagnostics.add(
        _schemaError('edit step requires at/anchor path', filePath),
      );
      return null;
    } on AstPathParseException catch (error) {
      diagnostics.add(
        RecipeDiagnostics.astPathParseError(error.message, filePath),
      );
      return null;
    }
  }

  CodemodTemplate? _parseTemplate(
    YamlMap create,
    String filePath,
    PathSandbox sandbox,
    List<RecipeDiagnostic> diagnostics,
    TemplateEnvironment templateEnvironment,
  ) {
    final inline = _stringField(create, 'template');
    if (inline != null) {
      _warnOnMissingMapIds(
        template: inline,
        filePath: filePath,
        diagnostics: diagnostics,
        mapsById: templateEnvironment.maps,
      );
      return CodemodTemplate.inline(inline, environment: templateEnvironment);
    }

    final templateFile = _stringField(create, 'templateFile');
    if (templateFile != null) {
      try {
        final resolved = sandbox.resolveTemplateRelative(templateFile);
        return CodemodTemplate.file(resolved, environment: templateEnvironment);
      } on PathSandboxException catch (error) {
        diagnostics.add(diagnosticFromSandbox(error, filePath));
        return null;
      }
    }

    diagnostics.add(
      _schemaError('create requires template or templateFile', filePath),
    );
    return null;
  }

  List<PostExecution> _parsePostExecution(
    Object? value,
    String filePath,
    PathSandbox sandbox,
    List<RecipeDiagnostic> diagnostics,
  ) {
    if (value == null) return const [];
    if (value is! YamlList) {
      diagnostics.add(_schemaError('postExecution must be a list', filePath));
      return const [];
    }

    final actions = <PostExecution>[];
    for (final entry in value) {
      if (entry is! YamlMap) {
        diagnostics.add(
          _schemaError('postExecution entries must be maps', filePath),
        );
        continue;
      }

      if (entry.containsKey('run')) {
        final command = _stringField(entry, 'run');
        if (command == null) continue;
        actions.add(_shellPostExecution(command));
        continue;
      }

      if (entry.containsKey('runScript')) {
        final script = _stringField(entry, 'runScript');
        if (script == null) continue;
        try {
          final resolved = sandbox.resolveTemplateRelative(script);
          actions.add(_shellPostExecution(resolved));
        } on PathSandboxException catch (error) {
          diagnostics.add(diagnosticFromSandbox(error, filePath));
        }
      } else {
        diagnostics.add(
          _schemaError(
            'postExecution entries must have "run" or "runScript"',
            filePath,
          ),
        );
      }
    }

    return actions;
  }

  CodemodRecipe? _resolveReferencedRecipe(
    String recipeId,
    String filePath,
    List<RecipeDiagnostic> diagnostics,
  ) {
    final yamlDefinition = definitionsById[recipeId];
    if (yamlDefinition != null) {
      final compiled = compile(yamlDefinition);
      if (compiled.recipe == null) {
        diagnostics.add(
          RecipeDiagnostics.recipeRefNotFound(
            'Referenced recipe "$recipeId" failed to compile',
            filePath,
          ),
        );
        return null;
      }
      return compiled.recipe;
    }

    final dartRecipe = dartRecipes[recipeId];
    if (dartRecipe != null) return dartRecipe;

    diagnostics.add(
      RecipeDiagnostics.recipeRefNotFound(
        'Referenced recipe "$recipeId" not found',
        filePath,
      ),
    );
    return null;
  }

  /// Uses the centralized schema validator for creating schema error diagnostics.
  RecipeDiagnostic _schemaError(String message, String filePath) {
    return YamlSchemaValidator.createError(message, filePath);
  }

  PostExecution _shellPostExecution(String command) {
    if (command.contains(' ')) {
      return ProcessPostExecution('bash', ['-lc', command]);
    }
    return ProcessPostExecution(command, const []);
  }
}

/// Result of compiling one YAML recipe.
class CompileResult {
  /// Creates a compile result.
  const CompileResult({this.recipe, this.diagnostics = const []});

  /// Compiled recipe when successful.
  final CodemodRecipe? recipe;

  /// Diagnostics from compilation.
  final List<RecipeDiagnostic> diagnostics;
}

FileExistsStrategy _parseIfExists(String? value) {
  return switch (value) {
    'skip' => FileExistsStrategy.skip,
    'overwrite' => FileExistsStrategy.overwrite,
    _ => FileExistsStrategy.fail,
  };
}

CodemodArgInputKind? _parseInputKind(String? value) {
  return switch (value) {
    'text' => CodemodArgInputKind.text,
    'file' => CodemodArgInputKind.file,
    'directory' => CodemodArgInputKind.directory,
    'enum' => CodemodArgInputKind.enumeration,
    'dartType' => CodemodArgInputKind.dartType,
    'symbol' => CodemodArgInputKind.symbol,
    'bool' => CodemodArgInputKind.boolean,
    _ => null,
  };
}

String? _stringField(YamlMap map, String key) {
  final value = map[key];
  if (value == null) return null;
  return value.toString();
}

List<String> _stringList(Object? value) {
  if (value is! YamlList) return const [];
  return value.map((item) => item.toString()).toList();
}

/// Parses a YAML recipe file from disk.
YamlRecipeDefinition parseYamlRecipeFile(String filePath, String contents) {
  final node = loadYaml(contents);
  if (node is! YamlMap) {
    throw FormatException('Recipe root must be a map', filePath);
  }

  final id = _stringField(node, 'id') ?? _stringField(node, 'name');
  if (id == null || id.isEmpty) {
    throw FormatException('Recipe must declare id or name', filePath);
  }

  return YamlRecipeDefinition(id: id, filePath: filePath, document: node);
}
