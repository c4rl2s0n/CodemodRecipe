/// Describes how constructor parameters are added to empty constructors.
enum ConstructorParamStyle {
  /// Named parameters in braces, e.g. `({required this.foo})`.
  named,

  /// Required positional parameters, e.g. `(this.foo)`.
  positional,

  /// Optional positional parameters in brackets, e.g. `([this.foo])`.
  optionalPositional,
}

/// Describes a field to add and how it maps to a constructor parameter.
class FieldSpec {
  /// Field and constructor parameter name.
  final String name;

  /// Base type without a nullable suffix.
  final String type;

  /// When true, appends `?` to [type] for declarations unless already present.
  final bool isNullable;

  /// Optional initializer expression (source text, not quoted).
  final String? defaultValue;

  /// Whether the field is declared `final`.
  final bool isFinal;

  /// Whether the field is declared `const`.
  final bool isConst;

  /// Whether the field is declared `static`.
  final bool isStatic;

  /// Creates a field specification.
  const FieldSpec({
    required this.name,
    required this.type,
    this.isNullable = false,
    this.defaultValue,
    this.isFinal = true,
    this.isConst = false,
    this.isStatic = false,
  });

  /// Type string for field and non-`this` constructor parameters.
  String get declarationType {
    final base = type.trim();
    if (!isNullable) return base;
    if (base.endsWith('?')) return base;
    return '$base?';
  }
}

/// Per-call overrides when wiring a field to a constructor.
class FieldConstructorArgs {
  /// Overrides [CodemodPreferences.emptyConstructorStyle] for empty constructors.
  ///
  /// When null, [CodemodPreferences.emptyConstructorStyle] is used.
  final ConstructorParamStyle? style;

  /// When true, emits `this.name`; otherwise emits `type name`.
  final bool thisPrefix;

  /// Creates constructor wiring overrides.
  const FieldConstructorArgs({this.style, this.thisPrefix = true});
}
