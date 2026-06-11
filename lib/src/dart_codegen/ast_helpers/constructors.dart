import 'package:analyzer/dart/ast/ast.dart';

import '../field_spec.dart';

/// Infers the constructor parameter list kind from existing parameters.
///
/// Returns null when the list is empty `()` and no delimiter is present.
ConstructorParamStyle? inferConstructorParamStyle(
  ConstructorDeclaration constructor,
  String source,
) {
  final params = constructor.parameters;
  if (params.parameters.isNotEmpty) {
    for (final parameter in params.parameters) {
      if (parameter is DefaultFormalParameter) {
        return parameter.isPositional
            ? ConstructorParamStyle.optionalPositional
            : ConstructorParamStyle.named;
      }
    }
    return ConstructorParamStyle.positional;
  }

  final inner = source
      .substring(params.leftParenthesis.end, params.rightParenthesis.offset)
      .trim();
  if (inner.startsWith('{')) return ConstructorParamStyle.named;
  if (inner.startsWith('[')) return ConstructorParamStyle.optionalPositional;
  if (inner.isEmpty) return null;

  return ConstructorParamStyle.positional;
}

/// Resolves the constructor parameter list kind, using [emptyStyle] for `()`.
ConstructorParamStyle resolveConstructorParamStyle(
  ConstructorDeclaration constructor,
  String source, {
  ConstructorParamStyle? emptyStyle,
}) {
  final inferred = inferConstructorParamStyle(constructor, source);
  if (inferred != null) return inferred;

  return emptyStyle ?? ConstructorParamStyle.named;
}

/// Builds the constructor parameter text for a [FieldSpec].
String buildConstructorFieldParam(
  FieldSpec spec,
  ConstructorParamStyle kind, {
  bool thisPrefix = true,
}) {
  final paramName = thisPrefix
      ? 'this.${spec.name}'
      : '${spec.declarationType} ${spec.name}';

  switch (kind) {
    case ConstructorParamStyle.named:
      if (spec.defaultValue != null) {
        return '$paramName = ${spec.defaultValue}';
      }
      if (!spec.isNullable) {
        return 'required $paramName';
      }
      return paramName;
    case ConstructorParamStyle.positional:
      if (spec.defaultValue != null) {
        throw StateError(
          'Positional constructor parameters cannot have default values',
        );
      }
      return paramName;
    case ConstructorParamStyle.optionalPositional:
      if (!spec.isNullable && spec.defaultValue == null) {
        throw StateError(
          'Non-nullable optional positional parameters require a defaultValue',
        );
      }
      if (spec.defaultValue != null) {
        return '$paramName = ${spec.defaultValue}';
      }
      return paramName;
  }
}
