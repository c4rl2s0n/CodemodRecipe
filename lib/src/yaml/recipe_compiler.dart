import 'package:yaml/yaml.dart';

import '../context.dart';
import '../generic/post_execution/build_runner_post_execution.dart';
import '../generic/post_execution/dart_format_post_execution.dart';
import '../operation.dart';
import '../generic/post_execution/process_post_execution.dart';
import '../post_execution.dart';
import '../transform.dart';
import '../recipe.dart';
import '../template.dart';
import 'diagnostics.dart';
import 'host_config.dart';
import 'insert_transform.dart';
import 'path_sandbox.dart';
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
        RecipeDiagnostic(
          severity: DiagnosticSeverity.error,
          code: 'E_YAML_COMPILE',
          message: '$error',
          sources: [DiagnosticSource(file: definition.filePath)],
        ),
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

    final path = _parseAstPath(insert, filePath, diagnostics);
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

  // Intentionally no additional edit steps beyond `insert` in the YAML surface.

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
      diagnostics.add(
        RecipeDiagnostic(
          severity: DiagnosticSeverity.warning,
          code: 'W_MAP_ID_NOT_FOUND',
          message: 'Template references missing map id \"$mapId\"',
          sources: [DiagnosticSource(file: filePath)],
        ),
      );
    }
  }

  AstPath? _parseAstPath(
    YamlMap insert,
    String filePath,
    List<RecipeDiagnostic> diagnostics,
  ) {
    try {
      if (insert.containsKey('at') && insert.containsKey('anchor')) {
        final at = insert['at'];
        if (at is String) {
          return parsePathString('$at @ ${insert['anchor']}');
        }
        if (at is YamlList) {
          return parseStructuredPath({
            'at': at.toList(),
            'anchor': insert['anchor'].toString(),
          });
        }
      }

      if (insert['at'] is String) {
        return parsePathString(insert['at'].toString());
      }

      diagnostics.add(_schemaError('insert requires at/anchor path', filePath));
      return null;
    } on AstPathParseException catch (error) {
      diagnostics.add(
        RecipeDiagnostic(
          severity: DiagnosticSeverity.error,
          code: 'E_AST_PATH_PARSE',
          message: error.message,
          sources: [DiagnosticSource(file: filePath)],
        ),
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
        final name = entry.toString();
        final builtin = _builtinPostExecution(name);
        if (builtin != null) {
          actions.add(builtin);
        } else {
          diagnostics.add(
            _schemaError('Unknown postExecution "$name"', filePath),
          );
        }
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
          RecipeDiagnostic(
            severity: DiagnosticSeverity.error,
            code: 'E_RECIPE_REF_NOT_FOUND',
            message: 'Referenced recipe "$recipeId" failed to compile',
            sources: [DiagnosticSource(file: filePath)],
          ),
        );
        return null;
      }
      return compiled.recipe;
    }

    final dartRecipe = dartRecipes[recipeId];
    if (dartRecipe != null) return dartRecipe;

    diagnostics.add(
      RecipeDiagnostic(
        severity: DiagnosticSeverity.error,
        code: 'E_RECIPE_REF_NOT_FOUND',
        message: 'Referenced recipe "$recipeId" not found',
        sources: [DiagnosticSource(file: filePath)],
      ),
    );
    return null;
  }

  RecipeDiagnostic _schemaError(String message, String filePath) {
    return RecipeDiagnostic(
      severity: DiagnosticSeverity.error,
      code: 'E_YAML_SCHEMA',
      message: message,
      sources: [DiagnosticSource(file: filePath)],
    );
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

PostExecution? _builtinPostExecution(String name) {
  return switch (name) {
    'dartFormat' => const DartFormatPostExecution(),
    'buildRunner' => BuildRunnerPostExecution(),
    _ => null,
  };
}

PostExecution _shellPostExecution(String command) {
  if (command.contains(' ')) {
    return ProcessPostExecution('bash', ['-lc', command]);
  }
  return ProcessPostExecution(command, const []);
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
