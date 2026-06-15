/// Severity of a recipe load or validation diagnostic.
enum DiagnosticSeverity {
  /// Blocks recipe registration or execution.
  error,

  /// Informational; recipe may still be usable.
  warning,
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
