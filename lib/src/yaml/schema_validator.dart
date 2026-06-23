// Copyright (c) 2026, the Dart project authors. Please see the AUTHORS file
// for details. All rights reserved. Use of this source code is governed by
// a BSD-style license that can be found in the LICENSE file.

import 'package:yaml/yaml.dart';

import 'diagnostics.dart';

/// Schema validation error codes.
class SchemaErrorCodes {
  static const String schemaError = 'E_YAML_SCHEMA';
  static const String compileError = 'E_YAML_COMPILE';
  static const String astPathParseError = 'E_AST_PATH_PARSE';
  static const String recipeRefNotFound = 'E_RECIPE_REF_NOT_FOUND';
}

/// Validates YAML recipe schema and generates diagnostics.
///
/// This class provides centralized schema validation for YAML recipes,
/// following the Single Responsibility Principle and allowing for
/// independent validation without compilation.
class YamlSchemaValidator {
  /// Validates a recipe definition document and returns any schema violations.
  ///
  /// This method checks the structure and fields of the YAML document without
  /// attempting to compile or resolve references.
  static List<RecipeDiagnostic> validateRecipeDocument(
    YamlMap document,
    String filePath,
  ) {
    final diagnostics = <RecipeDiagnostic>[];

    // Validate root structure
    if (document.isEmpty) {
      diagnostics.add(createError('Recipe document is empty', filePath));
      return diagnostics;
    }

    // Validate required fields
    final id = document['id']?.toString() ?? document['name']?.toString();
    if (id == null || id.isEmpty) {
      diagnostics.add(
        createError('Recipe must declare "id" or "name"', filePath),
      );
    }

    // Validate optional fields
    _validateOptionalStringField(
      document,
      'description',
      filePath,
      diagnostics,
    );
    _validateArgsField(document, filePath, diagnostics);
    _validateStepsField(document, filePath, diagnostics);
    _validateMapsField(document, filePath, diagnostics);
    _validatePostExecutionField(document, filePath, diagnostics);

    return diagnostics;
  }

  /// Validates the args field structure.
  static void _validateArgsField(
    YamlMap document,
    String filePath,
    List<RecipeDiagnostic> diagnostics,
  ) {
    final argsValue = document['args'];
    if (argsValue == null) return;

    if (argsValue is! YamlList) {
      diagnostics.add(createError('Field "args" must be a list', filePath));
      return;
    }

    for (final entry in argsValue) {
      if (entry is! YamlMap) {
        diagnostics.add(createError('Each args entry must be a map', filePath));
        continue;
      }

      _validateArgEntry(entry, filePath, diagnostics);
    }
  }

  /// Validates a single arg entry.
  static void _validateArgEntry(
    YamlMap entry,
    String filePath,
    List<RecipeDiagnostic> diagnostics,
  ) {
    final name = entry['name']?.toString();
    if (name == null || name.isEmpty) {
      diagnostics.add(
        createError('Arg entry missing required field "name"', filePath),
      );
      return;
    }

    // Validate known fields
    _validateOptionalStringField(entry, 'help', filePath, diagnostics);
    _validateOptionalStringField(entry, 'abbr', filePath, diagnostics);
    _validateOptionalStringField(entry, 'contextKey', filePath, diagnostics);
    _validateOptionalStringField(entry, 'defaultsTo', filePath, diagnostics);
    _validateOptionalStringField(entry, 'inputKind', filePath, diagnostics);

    // Validate options if present
    final options = entry['options'];
    if (options != null && options is! YamlList) {
      diagnostics.add(createError('Field "options" must be a list', filePath));
    }

    // Validate boolean fields
    _validateOptionalBooleanField(entry, 'required', filePath, diagnostics);
    _validateOptionalBooleanField(
      entry,
      'allowCustomValue',
      filePath,
      diagnostics,
    );
  }

  /// Validates the steps field structure.
  static void _validateStepsField(
    YamlMap document,
    String filePath,
    List<RecipeDiagnostic> diagnostics,
  ) {
    final stepsValue = document['steps'];
    if (stepsValue == null) return;

    if (stepsValue is! YamlList) {
      diagnostics.add(createError('Field "steps" must be a list', filePath));
      return;
    }

    for (final entry in stepsValue) {
      if (entry is! YamlMap) {
        diagnostics.add(createError('Each step must be a map', filePath));
        continue;
      }

      _validateStepEntry(entry, filePath, diagnostics);
    }
  }

  /// Validates a single step entry.
  static void _validateStepEntry(
    YamlMap entry,
    String filePath,
    List<RecipeDiagnostic> diagnostics,
  ) {
    // A step can be a recipe reference, edit operation, or create operation
    final hasRecipe = entry.containsKey('recipe');
    final hasEdit = entry.containsKey('edit');
    final hasCreate = entry.containsKey('create');

    final operationCount =
        (hasRecipe ? 1 : 0) + (hasEdit ? 1 : 0) + (hasCreate ? 1 : 0);
    if (operationCount > 1) {
      diagnostics.add(
        createError('Step cannot have multiple operation types', filePath),
      );
      return;
    }

    if (hasRecipe) {
      _validateRecipeStep(entry, filePath, diagnostics);
    } else if (hasEdit) {
      _validateEditStep(entry, filePath, diagnostics);
    } else if (hasCreate) {
      _validateCreateStep(entry, filePath, diagnostics);
    } else {
      diagnostics.add(
        createError('Step must have one of: recipe, edit, or create', filePath),
      );
    }
  }

  /// Validates a recipe reference step.
  static void _validateRecipeStep(
    YamlMap entry,
    String filePath,
    List<RecipeDiagnostic> diagnostics,
  ) {
    final recipeId = entry['recipe']?.toString();
    if (recipeId == null || recipeId.isEmpty) {
      diagnostics.add(
        createError('Recipe step missing required field "recipe"', filePath),
      );
    }
  }

  /// Validates an edit step.
  static void _validateEditStep(
    YamlMap entry,
    String filePath,
    List<RecipeDiagnostic> diagnostics,
  ) {
    final editValue = entry['edit'];
    if (editValue == null || editValue is! YamlMap) {
      diagnostics.add(
        createError('Edit step must have a "edit" map', filePath),
      );
      return;
    }

    final path = editValue['path']?.toString();
    if (path == null || path.isEmpty) {
      diagnostics.add(
        createError('Edit step missing required field "path"', filePath),
      );
      return;
    }

    // Validate steps within edit
    final editSteps = editValue['steps'];
    if (editSteps != null && editSteps is! YamlList) {
      diagnostics.add(createError('Edit "steps" must be a list', filePath));
      return;
    }

    // Validate each edit step
    if (editSteps is YamlList) {
      for (final editStep in editSteps) {
        if (editStep is! YamlMap) {
          diagnostics.add(
            createError('Each edit step must be a map', filePath),
          );
          continue;
        }
        _validateEditSubStep(editStep, filePath, diagnostics);
      }
    }
  }

  /// Validates an edit sub-step (insert, etc.).
  static void _validateEditSubStep(
    YamlMap entry,
    String filePath,
    List<RecipeDiagnostic> diagnostics,
  ) {
    final hasInsert = entry.containsKey('insert');
    // Add other edit step types as needed

    if (hasInsert) {
      _validateInsertStep(entry, filePath, diagnostics);
    } else {
      diagnostics.add(
        createError('Edit step must have an operation type', filePath),
      );
    }
  }

  /// Validates an insert step.
  static void _validateInsertStep(
    YamlMap entry,
    String filePath,
    List<RecipeDiagnostic> diagnostics,
  ) {
    final insertValue = entry['insert'];
    if (insertValue == null || insertValue is! YamlMap) {
      diagnostics.add(
        createError('Insert step must have an "insert" map', filePath),
      );
      return;
    }

    // Insert requires "at" field
    if (!insertValue.containsKey('at')) {
      diagnostics.add(
        createError('Insert step missing required field "at"', filePath),
      );
      return;
    }

    // Anchor is optional - it might be parsed from the path string
  }

  /// Validates a create step.
  static void _validateCreateStep(
    YamlMap entry,
    String filePath,
    List<RecipeDiagnostic> diagnostics,
  ) {
    final createValue = entry['create'];
    if (createValue == null || createValue is! YamlMap) {
      diagnostics.add(
        createError('Create step must have a "create" map', filePath),
      );
      return;
    }

    final path = createValue['path']?.toString();
    if (path == null || path.isEmpty) {
      diagnostics.add(
        createError('Create step missing required field "path"', filePath),
      );
      return;
    }

    // Validate that either template or templateFile is provided
    final hasTemplate = createValue.containsKey('template');
    final hasTemplateFile = createValue.containsKey('templateFile');

    if (!hasTemplate && !hasTemplateFile) {
      diagnostics.add(
        createError(
          'Create step requires "template" or "templateFile"',
          filePath,
        ),
      );
    } else if (hasTemplate && hasTemplateFile) {
      diagnostics.add(
        createError(
          'Create step cannot have both "template" and "templateFile"',
          filePath,
        ),
      );
    }

    // Validate optional fields
    _validateOptionalStringField(
      createValue,
      'ifExists',
      filePath,
      diagnostics,
    );
    _validateOptionalBooleanField(createValue, 'format', filePath, diagnostics);
  }

  /// Validates the maps field structure.
  static void _validateMapsField(
    YamlMap document,
    String filePath,
    List<RecipeDiagnostic> diagnostics,
  ) {
    final mapsValue = document['maps'];
    if (mapsValue == null) return;

    if (mapsValue is! YamlMap) {
      diagnostics.add(createError('Field "maps" must be a map', filePath));
      return;
    }

    for (final entry in mapsValue.entries) {
      final mapId = entry.key.toString();
      final mapContent = entry.value;

      if (mapContent is! YamlMap) {
        diagnostics.add(createError('Map "$mapId" must be a map', filePath));
        continue;
      }

      // Validate that the map has entries
      if (!mapContent.containsKey('entries')) {
        diagnostics.add(
          createError(
            'Map "$mapId" missing required field "entries"',
            filePath,
          ),
        );
        continue;
      }

      final entries = mapContent['entries'];
      if (entries is! YamlMap) {
        diagnostics.add(
          createError('Map "$mapId" entries must be a map', filePath),
        );
      }
    }
  }

  /// Validates the postExecution field structure.
  static void _validatePostExecutionField(
    YamlMap document,
    String filePath,
    List<RecipeDiagnostic> diagnostics,
  ) {
    final postExecutionValue = document['postExecution'];
    if (postExecutionValue == null) return;

    if (postExecutionValue is! YamlList) {
      diagnostics.add(
        createError('Field "postExecution" must be a list', filePath),
      );
      return;
    }

    for (final entry in postExecutionValue) {
      if (entry is String) {
        // Simple string command
        continue;
      } else if (entry is YamlMap) {
        // Map with command configuration
        _validatePostExecutionEntry(entry, filePath, diagnostics);
      } else {
        diagnostics.add(
          createError('Post execution entry must be a string or map', filePath),
        );
      }
    }
  }

  /// Validates a post-execution entry.
  static void _validatePostExecutionEntry(
    YamlMap entry,
    String filePath,
    List<RecipeDiagnostic> diagnostics,
  ) {
    final hasRun = entry.containsKey('run');
    final hasRunScript = entry.containsKey('runScript');

    if (!hasRun && !hasRunScript) {
      diagnostics.add(
        createError(
          'Post execution entry must have "run" or "runScript"',
          filePath,
        ),
      );
    } else if (hasRun && hasRunScript) {
      diagnostics.add(
        createError(
          'Post execution entry cannot have both "run" and "runScript"',
          filePath,
        ),
      );
    }
  }

  /// Validates an optional string field.
  static void _validateOptionalStringField(
    YamlMap map,
    String fieldName,
    String filePath,
    List<RecipeDiagnostic> diagnostics,
  ) {
    final value = map[fieldName];
    if (value != null && value is! String) {
      diagnostics.add(
        createError('Field "$fieldName" must be a string', filePath),
      );
    }
  }

  /// Validates an optional boolean field.
  static void _validateOptionalBooleanField(
    YamlMap map,
    String fieldName,
    String filePath,
    List<RecipeDiagnostic> diagnostics,
  ) {
    final value = map[fieldName];
    if (value != null && value is! bool) {
      diagnostics.add(
        createError('Field "$fieldName" must be a boolean', filePath),
      );
    }
  }

  /// Creates a schema error diagnostic.
  static RecipeDiagnostic createError(String message, String filePath) {
    return RecipeDiagnostic(
      severity: DiagnosticSeverity.error,
      code: SchemaErrorCodes.schemaError,
      message: message,
      sources: [DiagnosticSource(file: filePath)],
    );
  }

  /// Creates a compile error diagnostic.
  static RecipeDiagnostic compileError(String message, String filePath) {
    return RecipeDiagnostic(
      severity: DiagnosticSeverity.error,
      code: SchemaErrorCodes.compileError,
      message: message,
      sources: [DiagnosticSource(file: filePath)],
    );
  }

  /// Creates a diagnostic for AST path parse errors.
  static RecipeDiagnostic astPathParseError(String message, String filePath) {
    return RecipeDiagnostic(
      severity: DiagnosticSeverity.error,
      code: SchemaErrorCodes.astPathParseError,
      message: message,
      sources: [DiagnosticSource(file: filePath)],
    );
  }

  /// Creates a diagnostic for recipe reference not found errors.
  static RecipeDiagnostic recipeRefNotFoundError(
    String message,
    String filePath,
  ) {
    return RecipeDiagnostic(
      severity: DiagnosticSeverity.error,
      code: SchemaErrorCodes.recipeRefNotFound,
      message: message,
      sources: [DiagnosticSource(file: filePath)],
    );
  }
}
