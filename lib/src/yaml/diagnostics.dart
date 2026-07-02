/// Severity of a recipe load or validation diagnostic.
enum DiagnosticSeverity {
  /// Blocks recipe registration or execution.
  error,

  /// Informational; recipe may still be usable.
  warning,
}

/// Stable machine-readable diagnostic codes.
class DiagnosticCodes {
  static const String schemaError = 'E_YAML_SCHEMA';
  static const String compileError = 'E_YAML_COMPILE';
  static const String astPathParseError = 'E_AST_PATH_PARSE';
  static const String recipeRefNotFound = 'E_RECIPE_REF_NOT_FOUND';
  static const String duplicateId = 'E_DUPLICATE_ID';
  static const String duplicateMapId = 'E_DUPLICATE_MAP_ID';
  static const String yamlParse = 'E_YAML_PARSE';
  static const String missingId = 'E_MISSING_ID';
  static const String unknownYamlType = 'E_UNKNOWN_YAML_TYPE';
  static const String mapSchema = 'E_MAP_SCHEMA';
  static const String mapParse = 'E_MAP_PARSE';
  static const String mapIdNotFound = 'W_MAP_ID_NOT_FOUND';
  static const String pathTraversal = 'E_PATH_TRAVERSAL';
}

/// A source location for a diagnostic.
class DiagnosticSource {
  /// Creates a diagnostic source reference.
  const DiagnosticSource({required this.file, this.line, this.column});

  /// File path relative to workspace or absolute.
  final String file;

  /// 1-based line number when available.
  final int? line;

  /// 1-based column number when available.
  final int? column;

  Map<String, Object?> toJson() => {
    'file': file,
    if (line != null) 'line': line,
    if (column != null) 'column': column,
  };
}

/// Structured diagnostic from recipe loading or validation.
class RecipeDiagnostic {
  /// Creates a recipe diagnostic.
  const RecipeDiagnostic({
    required this.severity,
    required this.code,
    required this.message,
    this.sources = const [],
  });

  /// Error or warning.
  final DiagnosticSeverity severity;

  /// Stable machine-readable code.
  final String code;

  /// Human-readable description.
  final String message;

  /// Related file locations.
  final List<DiagnosticSource> sources;

  Map<String, Object?> toJson() => {
    'severity': severity.name,
    'code': code,
    'message': message,
    'sources': [for (final source in sources) source.toJson()],
  };
}

/// Factory helpers for consistent [RecipeDiagnostic] construction.
class RecipeDiagnostics {
  RecipeDiagnostics._();

  static RecipeDiagnostic error({
    required String code,
    required String message,
    List<DiagnosticSource> sources = const [],
  }) {
    return RecipeDiagnostic(
      severity: DiagnosticSeverity.error,
      code: code,
      message: message,
      sources: sources,
    );
  }

  static RecipeDiagnostic warning({
    required String code,
    required String message,
    List<DiagnosticSource> sources = const [],
  }) {
    return RecipeDiagnostic(
      severity: DiagnosticSeverity.warning,
      code: code,
      message: message,
      sources: sources,
    );
  }

  static RecipeDiagnostic forFile({
    required DiagnosticSeverity severity,
    required String code,
    required String message,
    required String file,
  }) {
    return RecipeDiagnostic(
      severity: severity,
      code: code,
      message: message,
      sources: [DiagnosticSource(file: file)],
    );
  }

  static RecipeDiagnostic schemaError(String message, String file) =>
      forFile(
        severity: DiagnosticSeverity.error,
        code: DiagnosticCodes.schemaError,
        message: message,
        file: file,
      );

  static RecipeDiagnostic compileError(String message, String file) =>
      forFile(
        severity: DiagnosticSeverity.error,
        code: DiagnosticCodes.compileError,
        message: message,
        file: file,
      );

  static RecipeDiagnostic astPathParseError(String message, String file) =>
      forFile(
        severity: DiagnosticSeverity.error,
        code: DiagnosticCodes.astPathParseError,
        message: message,
        file: file,
      );

  static RecipeDiagnostic recipeRefNotFound(String message, String file) =>
      forFile(
        severity: DiagnosticSeverity.error,
        code: DiagnosticCodes.recipeRefNotFound,
        message: message,
        file: file,
      );

  static RecipeDiagnostic duplicateRecipeId(
    String id,
    List<DiagnosticSource> sources,
  ) =>
      error(
        code: DiagnosticCodes.duplicateId,
        message: "Duplicate recipe id '$id'",
        sources: sources,
      );

  static RecipeDiagnostic duplicateMapId(
    String id,
    List<DiagnosticSource> sources,
  ) =>
      error(
        code: DiagnosticCodes.duplicateMapId,
        message: "Duplicate map id '$id'",
        sources: sources,
      );

  static RecipeDiagnostic yamlParseError(String message, String file) =>
      forFile(
        severity: DiagnosticSeverity.error,
        code: DiagnosticCodes.yamlParse,
        message: message,
        file: file,
      );

  static RecipeDiagnostic missingId(String file) => forFile(
    severity: DiagnosticSeverity.error,
    code: DiagnosticCodes.missingId,
    message: 'YAML file missing required "id" field',
    file: file,
  );

  static RecipeDiagnostic unknownYamlType(String file) => forFile(
    severity: DiagnosticSeverity.error,
    code: DiagnosticCodes.unknownYamlType,
    message:
        'YAML file has "id" but no "steps" (recipe) or "entries" (map)',
    file: file,
  );

  static RecipeDiagnostic mapSchemaError(String message, String file) =>
      forFile(
        severity: DiagnosticSeverity.error,
        code: DiagnosticCodes.mapSchema,
        message: message,
        file: file,
      );

  static RecipeDiagnostic mapParseError(String message, String file) =>
      forFile(
        severity: DiagnosticSeverity.error,
        code: DiagnosticCodes.mapParse,
        message: message,
        file: file,
      );

  static RecipeDiagnostic mapIdNotFound(String mapId, String file) => warning(
    code: DiagnosticCodes.mapIdNotFound,
    message: 'Template references missing map id "$mapId"',
    sources: [DiagnosticSource(file: file)],
  );

  static RecipeDiagnostic pathSandbox(
    String message,
    String file, {
    String? code,
  }) =>
      forFile(
        severity: DiagnosticSeverity.error,
        code: code ?? DiagnosticCodes.pathTraversal,
        message: message,
        file: file,
      );
}
