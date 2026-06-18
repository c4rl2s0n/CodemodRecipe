// Copyright (c) 2026, the Dart project authors. Please see the AUTHORS file
// for details. All rights reserved. Use of this source code is governed by
// a BSD-style license that can be found in the LICENSE file.

/// Base exception for all codemod-related errors.
///
/// This exception hierarchy provides standardized error handling
/// and better error reporting across the codebase.
class CodemodException implements Exception {
  /// Creates a codemod exception.
  const CodemodException(this.message, {this.code});

  /// Human-readable error message.
  final String message;

  /// Optional error code for categorization.
  final String? code;

  @override
  String toString() {
    if (code != null) {
      return '$code: $message';
    }
    return message;
  }

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is CodemodException &&
          message == other.message &&
          code == other.code;

  @override
  int get hashCode => Object.hash(message, code);
}

/// Exception thrown when there's an issue with file operations.
class FileSystemException extends CodemodException {
  /// Creates a file system exception.
  FileSystemException(String message, {this.path, String? code})
      : super(message, code: code ?? 'E_FILE_SYSTEM');

  /// Path involved in the error, if available.
  final String? path;
}

/// Exception thrown when YAML parsing or processing fails.
class YamlProcessingException extends CodemodException {
  /// Creates a YAML processing exception.
  YamlProcessingException(String message, {this.filePath, String? code})
      : super(message, code: code ?? 'E_YAML_COMPILE');

  /// File path where the error occurred, if available.
  final String? filePath;
}

/// Exception thrown when AST path parsing fails.
class AstPathParseException implements Exception {
  /// Creates a parse exception.
  AstPathParseException(this.message);

  /// Human-readable parse failure description.
  final String message;

  @override
  String toString() => 'AstPathParseException: $message';
}

/// Exception thrown when a recipe reference cannot be resolved.
class RecipeReferenceException extends CodemodException {
  /// Creates a recipe reference exception.
  RecipeReferenceException(String message, {this.recipeId, this.filePath})
      : super(message, code: 'E_RECIPE_REF_NOT_FOUND');

  /// The recipe ID that could not be found.
  final String? recipeId;

  /// The file path where the reference was made.
  final String? filePath;
}

/// Exception thrown when a required argument is missing.
class MissingArgumentException extends CodemodException {
  /// Creates a missing argument exception.
  MissingArgumentException(this.argumentName, {String? code})
      : super('Missing required argument: $argumentName', code: code);

  /// Name of the missing argument.
  final String argumentName;
}

/// Exception thrown when there's a validation error.
class ValidationException extends CodemodException {
  /// Creates a validation exception.
  ValidationException(String message, {String? code})
      : super(message, code: code ?? 'E_YAML_SCHEMA');
}

/// Exception thrown when a resource is not found.
class ResourceNotFoundException extends CodemodException {
  /// Creates a resource not found exception.
  ResourceNotFoundException(this.resourceType, this.resourceId, {String? code})
      : super('$resourceType "$resourceId" not found', code: code);

  /// Type of resource that was not found.
  final String resourceType;

  /// Identifier of the resource that was not found.
  final String resourceId;
}

/// Exception thrown when there's a schema validation error.
class SchemaValidationException extends ValidationException {
  /// Creates a schema validation exception.
  SchemaValidationException(String message, {this.field, this.filePath, String? code})
      : super(message, code: code ?? 'E_YAML_SCHEMA');

  /// Field that failed validation.
  final String? field;

  /// File path where the validation failed.
  final String? filePath;
}

/// Exception thrown when there's an issue with the host configuration.
class HostConfigurationException extends CodemodException {
  /// Creates a host configuration exception.
  HostConfigurationException(String message, {String? code})
      : super(message, code: code ?? 'E_HOST_CONFIG');
}

/// Exception thrown when there's an issue with template processing.
class TemplateException extends CodemodException {
  /// Creates a template exception.
  TemplateException(String message, {this.template, String? code})
      : super(message, code: code);

  /// Template that caused the error.
  final String? template;
}

/// Exception thrown when there's an issue with path sandbox operations.
class PathSandboxException implements Exception {
  /// Creates a path sandbox exception.
  PathSandboxException(this.message, {this.code});

  /// Human-readable error message.
  final String message;

  /// Optional error code for categorization.
  final String? code;

  @override
  String toString() {
    if (code != null) {
      return '$code: $message';
    }
    return 'PathSandboxException: $message';
  }
}