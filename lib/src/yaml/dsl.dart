// Copyright (c) 2026, the Dart project authors. Please see the AUTHORS file
// for details. All rights reserved. Use of this source code is governed by
// a BSD-style license that can be found in the LICENSE file.

import 'dart:io';
import 'package:yaml/yaml.dart';

/// Typed DSL classes for YAML recipe elements.
///
/// This module provides strongly typed classes that represent the structure
/// of YAML recipes, enabling type-safe access to recipe components and
/// better error handling during parsing.

/// Base class for all YAML DSL nodes.
abstract class YamlRecipeNode {
  /// The source file path where this node was parsed from.
  final String filePath;

  /// Creates a YAML recipe node.
  const YamlRecipeNode(this.filePath);
}

/// Represents a complete YAML recipe definition.
///
/// This is the root node that contains all recipe components.
class YamlRecipeNodeRoot extends YamlRecipeNode {
  /// Recipe identifier (from 'id' or 'name' field).
  final String? id;

  /// Human-readable name for the recipe.
  final String? name;

  /// Recipe description.
  final String? description;

  /// Recipe arguments.
  final List<YamlArgNode> args;

  /// Recipe steps.
  final List<YamlStepNode> steps;

  /// Maps definitions.
  final Map<String, YamlMapNode> maps;

  /// Post-execution actions.
  final List<YamlPostExecutionNode> postExecution;

  /// Creates a YAML recipe root node.
  YamlRecipeNodeRoot({
    required String filePath,
    this.id,
    this.name,
    this.description,
    this.args = const [],
    this.steps = const [],
    this.maps = const {},
    this.postExecution = const [],
  }) : super(filePath);

  /// Creates a YAML recipe root node from a parsed YamlMap.
  factory YamlRecipeNodeRoot.fromYamlMap(YamlMap map, String filePath) {
    return YamlRecipeNodeRoot(
      filePath: filePath,
      id: map['id']?.toString(),
      name: map['name']?.toString(),
      description: map['description']?.toString(),
      args: _parseArgs(map['args'], filePath),
      steps: _parseSteps(map['steps'], filePath),
      maps: _parseMaps(map['maps'], filePath),
      postExecution: _parsePostExecution(map['postExecution'], filePath),
    );
  }

  static List<YamlArgNode> _parseArgs(Object? value, String filePath) {
    if (value is! YamlList) return const [];
    return value
        .whereType<YamlMap>()
        .map((map) => YamlArgNode.fromYamlMap(map, filePath))
        .toList();
  }

  static List<YamlStepNode> _parseSteps(Object? value, String filePath) {
    if (value is! YamlList) return const [];
    return value
        .whereType<YamlMap>()
        .map((map) => YamlStepNode.fromYamlMap(map, filePath))
        .toList();
  }

  static Map<String, YamlMapNode> _parseMaps(Object? value, String filePath) {
    if (value is! YamlMap) return const {};
    return Map.fromEntries(
      value.entries
          .where((entry) => entry.value is YamlMap)
          .map((entry) => MapEntry(
                entry.key.toString(),
                YamlMapNode.fromYamlMap(entry.value as YamlMap, filePath),
              ))
          .toList(),
    );
  }

  static List<YamlPostExecutionNode> _parsePostExecution(Object? value, String filePath) {
    if (value is! YamlList) return const [];
    return value
        .map((item) => YamlPostExecutionNode.fromDynamic(item, filePath))
        .toList();
  }

  /// Validates this recipe definition and returns any diagnostics.
  List<String> validate() {
    final errors = <String>[];
    
    if (id == null && name == null) {
      errors.add('Recipe must have an id or name');
    }
    
    // Validate args
    for (final arg in args) {
      errors.addAll(arg.validate());
    }
    
    // Validate steps
    for (final step in steps) {
      errors.addAll(step.validate());
    }
    
    // Validate maps
    for (final map in maps.values) {
      errors.addAll(map.validate());
    }
    
    // Validate post execution
    for (final postExec in postExecution) {
      errors.addAll(postExec.validate());
    }
    
    return errors;
  }
}

/// Represents a YAML argument definition.
class YamlArgNode extends YamlRecipeNode {
  /// Argument name.
  final String? name;

  /// Whether the argument is required.
  final bool? required;

  /// Help text for the argument.
  final String? help;

  /// Short abbreviation for the argument.
  final String? abbr;

  /// Context key for auto-filling from editor context.
  final String? contextKey;

  /// Input kind (text, file, directory, etc.).
  final String? inputKind;

  /// Available options for the argument.
  final List<String>? options;

  /// Whether to allow custom values not in options.
  final bool? allowCustomValue;

  /// Default value.
  final String? defaultsTo;

  /// Creates a YAML argument node.
  YamlArgNode({
    required String filePath,
    this.name,
    this.required,
    this.help,
    this.abbr,
    this.contextKey,
    this.inputKind,
    this.options,
    this.allowCustomValue,
    this.defaultsTo,
  }) : super(filePath);

  /// Creates a YAML argument node from a parsed YamlMap.
  factory YamlArgNode.fromYamlMap(YamlMap map, String filePath) {
    return YamlArgNode(
      filePath: filePath,
      name: map['name']?.toString(),
      required: map['required'] as bool?,
      help: map['help']?.toString(),
      abbr: map['abbr']?.toString(),
      contextKey: map['contextKey']?.toString(),
      inputKind: map['inputKind']?.toString(),
      options: _parseStringList(map['options']),
      allowCustomValue: map['allowCustomValue'] as bool?,
      defaultsTo: map['defaultsTo']?.toString(),
    );
  }

  static List<String>? _parseStringList(Object? value) {
    if (value is! YamlList) return null;
    return value.map((item) => item.toString()).toList();
  }

  /// Validates this argument definition.
  List<String> validate() {
    final errors = <String>[];
    
    if (name == null || name!.isEmpty) {
      errors.add('Argument name is required');
    }
    
    return errors;
  }
}

/// Represents a YAML recipe step.
class YamlStepNode extends YamlRecipeNode {
  /// Recipe reference (for composition).
  final YamlRecipeReferenceNode? recipe;

  /// Edit operation.
  final YamlEditNode? edit;

  /// Create operation.
  final YamlCreateNode? create;

  /// Creates a YAML step node.
  YamlStepNode({
    required String filePath,
    this.recipe,
    this.edit,
    this.create,
  }) : super(filePath);

  /// Creates a YAML step node from a parsed YamlMap.
  factory YamlStepNode.fromYamlMap(YamlMap map, String filePath) {
    YamlRecipeReferenceNode? recipe;
    YamlEditNode? edit;
    YamlCreateNode? create;

    if (map.containsKey('recipe')) {
      recipe = YamlRecipeReferenceNode.fromYamlMap(map, filePath);
    } else if (map.containsKey('edit')) {
      edit = YamlEditNode.fromYamlMap(map, filePath);
    } else if (map.containsKey('create')) {
      create = YamlCreateNode.fromYamlMap(map, filePath);
    }

    return YamlStepNode(
      filePath: filePath,
      recipe: recipe,
      edit: edit,
      create: create,
    );
  }

  /// Validates this step definition.
  List<String> validate() {
    final errors = <String>[];
    
    final operationCount = 
        (recipe != null ? 1 : 0) + 
        (edit != null ? 1 : 0) + 
        (create != null ? 1 : 0);
    
    if (operationCount == 0) {
      errors.add('Step must have one of: recipe, edit, or create');
    } else if (operationCount > 1) {
      errors.add('Step can only have one operation type');
    }
    
    if (recipe != null) {
      errors.addAll(recipe!.validate());
    }
    if (edit != null) {
      errors.addAll(edit!.validate());
    }
    if (create != null) {
      errors.addAll(create!.validate());
    }
    
    return errors;
  }
}

/// Represents a recipe reference step.
class YamlRecipeReferenceNode extends YamlRecipeNode {
  /// The ID of the referenced recipe.
  final String? recipeId;

  /// Creates a YAML recipe reference node.
  YamlRecipeReferenceNode({
    required String filePath,
    this.recipeId,
  }) : super(filePath);

  /// Creates a YAML recipe reference node from a parsed YamlMap.
  factory YamlRecipeReferenceNode.fromYamlMap(YamlMap map, String filePath) {
    return YamlRecipeReferenceNode(
      filePath: filePath,
      recipeId: map['recipe']?.toString(),
    );
  }

  /// Validates this recipe reference.
  List<String> validate() {
    final errors = <String>[];
    
    if (recipeId == null || recipeId!.isEmpty) {
      errors.add('Recipe reference requires a recipe ID');
    }
    
    return errors;
  }
}

/// Represents an edit operation step.
class YamlEditNode extends YamlRecipeNode {
  /// Path template for the file to edit.
  final String? path;

  /// Edit steps (transformations).
  final List<YamlEditStepNode> steps;

  /// Creates a YAML edit node.
  YamlEditNode({
    required String filePath,
    this.path,
    this.steps = const [],
  }) : super(filePath);

  /// Creates a YAML edit node from a parsed YamlMap.
  factory YamlEditNode.fromYamlMap(YamlMap map, String filePath) {
    final editValue = map['edit'];
    
    if (editValue is! YamlMap) {
      return YamlEditNode(filePath: filePath);
    }

    return YamlEditNode(
      filePath: filePath,
      path: editValue['path']?.toString(),
      steps: _parseEditSteps(editValue['steps'], filePath),
    );
  }

  static List<YamlEditStepNode> _parseEditSteps(Object? value, String filePath) {
    if (value is! YamlList) return const [];
    return value
        .whereType<YamlMap>()
        .map((map) => YamlEditStepNode.fromYamlMap(map, filePath))
        .toList();
  }

  /// Validates this edit operation.
  List<String> validate() {
    final errors = <String>[];
    
    if (path == null || path!.isEmpty) {
      errors.add('Edit operation requires a path');
    }
    
    for (final step in steps) {
      errors.addAll(step.validate());
    }
    
    return errors;
  }
}

/// Represents an edit sub-step (like insert).
class YamlEditStepNode extends YamlRecipeNode {
  /// Insert operation.
  final YamlInsertNode? insert;

  /// Creates a YAML edit step node.
  YamlEditStepNode({
    required String filePath,
    this.insert,
  }) : super(filePath);

  /// Creates a YAML edit step node from a parsed YamlMap.
  factory YamlEditStepNode.fromYamlMap(YamlMap map, String filePath) {
    YamlInsertNode? insert;

    if (map.containsKey('insert')) {
      insert = YamlInsertNode.fromYamlMap(map, filePath);
    }

    return YamlEditStepNode(
      filePath: filePath,
      insert: insert,
    );
  }

  /// Validates this edit step.
  List<String> validate() {
    final errors = <String>[];
    
    if (insert == null) {
      errors.add('Edit step must have an operation');
    } else {
      errors.addAll(insert!.validate());
    }
    
    return errors;
  }
}

/// Represents an insert operation.
class YamlInsertNode extends YamlRecipeNode {
  /// AST path string or structured path.
  final dynamic at;

  /// Anchor position.
  final String? anchor;

  /// Text to insert.
  final String? text;

  /// Creates a YAML insert node.
  YamlInsertNode({
    required String filePath,
    this.at,
    this.anchor,
    this.text,
  }) : super(filePath);

  /// Creates a YAML insert node from a parsed YamlMap.
  factory YamlInsertNode.fromYamlMap(YamlMap map, String filePath) {
    return YamlInsertNode(
      filePath: filePath,
      at: map['at'],
      anchor: map['anchor']?.toString(),
      text: map['text']?.toString(),
    );
  }

  /// Validates this insert operation.
  List<String> validate() {
    final errors = <String>[];
    
    if (at == null) {
      errors.add('Insert operation requires "at" field');
    }
    
    if (text == null || text!.isEmpty) {
      errors.add('Insert operation requires "text" field');
    }
    
    return errors;
  }
}

/// Represents a create operation.
class YamlCreateNode extends YamlRecipeNode {
  /// Path template for the file to create.
  final String? path;

  /// Template content.
  final YamlTemplateNode? template;

  /// Template file path.
  final String? templateFile;

  /// How to handle if file exists.
  final String? ifExists;

  /// Whether to format the created file.
  final bool? format;

  /// Creates a YAML create node.
  YamlCreateNode({
    required String filePath,
    this.path,
    this.template,
    this.templateFile,
    this.ifExists,
    this.format,
  }) : super(filePath);

  /// Creates a YAML create node from a parsed YamlMap.
  factory YamlCreateNode.fromYamlMap(YamlMap map, String filePath) {
    YamlTemplateNode? templateNode;
    
    final templateValue = map['template']?.toString();
    if (templateValue != null) {
      templateNode = YamlTemplateNode.inline(templateValue, filePath);
    }

    return YamlCreateNode(
      filePath: filePath,
      path: map['path']?.toString(),
      template: templateNode,
      templateFile: map['templateFile']?.toString(),
      ifExists: map['ifExists']?.toString(),
      format: map['format'] as bool?,
    );
  }

  /// Validates this create operation.
  List<String> validate() {
    final errors = <String>[];
    
    if (path == null || path!.isEmpty) {
      errors.add('Create operation requires "path" field');
    }
    
    if (template == null && (templateFile == null || templateFile!.isEmpty)) {
      errors.add('Create operation requires "template" or "templateFile"');
    }
    
    return errors;
  }
}

/// Represents a template node (inline or file-based).
class YamlTemplateNode extends YamlRecipeNode {
  /// Template content (for inline templates).
  final String? content;

  /// Template file path (for file-based templates).
  final String? templateFilePath;

  /// Whether this is an inline template.
  final bool isInline;

  /// Creates a YAML template node.
  YamlTemplateNode.inline(String content, String sourceFilePath) 
      : this(
          content: content,
          templateFilePath: null,
          isInline: true,
          sourceFilePath: sourceFilePath,
        );

  /// Creates a YAML template node.
  YamlTemplateNode.file(String templateFilePath, String sourceFilePath) 
      : this(
          content: null,
          templateFilePath: templateFilePath,
          isInline: false,
          sourceFilePath: sourceFilePath,
        );

  /// Creates a YAML template node.
  YamlTemplateNode({
    required String sourceFilePath,
    this.content,
    this.templateFilePath,
    this.isInline = true,
  }) : super(sourceFilePath);

  /// Validates this template node.
  List<String> validate() {
    final errors = <String>[];
    
    if (isInline && (content == null || content!.isEmpty)) {
      errors.add('Inline template requires content');
    }
    
    if (!isInline && (filePath.isEmpty)) {
      errors.add('File template requires file path');
    }
    
    return errors;
  }
}

/// Represents a map definition.
class YamlMapNode extends YamlRecipeNode {
  /// Map entries.
  final Map<String, String> entries;

  /// Creates a YAML map node.
  YamlMapNode({
    required String filePath,
    this.entries = const {},
  }) : super(filePath);

  /// Creates a YAML map node from a parsed YamlMap.
  factory YamlMapNode.fromYamlMap(YamlMap map, String filePath) {
    final entries = <String, String>{};
    final entriesValue = map['entries'];
    
    if (entriesValue is YamlMap) {
      for (final entry in entriesValue.entries) {
        entries[entry.key.toString()] = entry.value?.toString() ?? '';
      }
    }

    return YamlMapNode(
      filePath: filePath,
      entries: entries,
    );
  }

  /// Validates this map definition.
  List<String> validate() {
    return []; // Maps are always valid
  }
}

/// Represents a post-execution action.
class YamlPostExecutionNode extends YamlRecipeNode {
  /// Run command.
  final String? run;

  /// Run script file.
  final String? runScript;

  /// Simple command name (for built-in post-executions).
  final String? command;

  /// Creates a YAML post-execution node.
  YamlPostExecutionNode({
    required String filePath,
    this.run,
    this.runScript,
    this.command,
  }) : super(filePath);

  /// Creates a YAML post-execution node from a dynamic value.
  factory YamlPostExecutionNode.fromDynamic(dynamic value, String filePath) {
    if (value is String) {
      return YamlPostExecutionNode(
        filePath: filePath,
        command: value,
      );
    } else if (value is YamlMap) {
      return YamlPostExecutionNode(
        filePath: filePath,
        run: value['run']?.toString(),
        runScript: value['runScript']?.toString(),
      );
    }
    
    return YamlPostExecutionNode(filePath: filePath);
  }

  /// Validates this post-execution action.
  List<String> validate() {
    final errors = <String>[];
    
    final operationCount = 
        (run != null ? 1 : 0) + 
        (runScript != null ? 1 : 0) + 
        (command != null ? 1 : 0);
    
    if (operationCount == 0) {
      errors.add('Post-execution action must have a command, run, or runScript');
    } else if (operationCount > 1) {
      errors.add('Post-execution action can only have one operation type');
    }
    
    return errors;
  }
}

/// Factory methods for creating typed YAML DSL nodes from raw YAML.
class YamlDslFactory {
  /// Parses a YAML recipe from string content.
  static YamlRecipeNodeRoot? parseRecipe(String content, String filePath) {
    try {
      final yaml = loadYaml(content);
      if (yaml is! YamlMap) {
        return null;
      }
      return YamlRecipeNodeRoot.fromYamlMap(yaml, filePath);
    } catch (_) {
      return null;
    }
  }

  /// Creates a YamlRecipeNodeRoot from a file.
  static Future<YamlRecipeNodeRoot?> parseRecipeFromFile(String filePath) async {
    try {
      final content = await File(filePath).readAsString();
      return parseRecipe(content, filePath);
    } catch (_) {
      return null;
    }
  }
}