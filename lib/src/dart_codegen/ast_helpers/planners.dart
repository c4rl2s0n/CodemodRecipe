import 'package:analyzer/dart/ast/ast.dart';

import '../field_spec.dart';
import 'offsets.dart';

/// Describes where and what text to insert or replace in source.
class InsertionPlan {
  /// Start offset in the source.
  final int offset;

  /// Number of characters to replace, or 0 for pure insertion.
  final int length;

  /// Text to insert or substitute.
  final String text;

  /// Creates an insertion plan.
  const InsertionPlan({
    required this.offset,
    this.length = 0,
    required this.text,
  });
}

/// Backwards-compatible alias for [InsertionPlan].
typedef ConstructorParamInsertion = InsertionPlan;

/// Plans where and how to insert [paramText] into a constructor parameter list.
InsertionPlan planConstructorParamInsertion(
  ConstructorDeclaration constructor,
  String source,
  ConstructorParamStyle kind,
  String paramText,
) {
  final params = constructor.parameters;
  final hasParams = params.parameters.isNotEmpty;

  if (!hasParams) {
    final innerStart = params.leftParenthesis.end;
    final innerEnd = params.rightParenthesis.offset;
    final inner = source.substring(innerStart, innerEnd).trim();

    if (inner.isEmpty) {
      switch (kind) {
        case ConstructorParamStyle.named:
          return InsertionPlan(
            offset: innerStart,
            length: innerEnd - innerStart,
            text: '{$paramText}',
          );
        case ConstructorParamStyle.optionalPositional:
          return InsertionPlan(
            offset: innerStart,
            length: innerEnd - innerStart,
            text: '[$paramText]',
          );
        case ConstructorParamStyle.positional:
          return InsertionPlan(
            offset: innerStart,
            length: innerEnd - innerStart,
            text: paramText,
          );
      }
    }

    if (inner.startsWith('{')) {
      final braceOffset = source.indexOf('{', params.leftParenthesis.offset);
      return InsertionPlan(
        offset: braceOffset + 1,
        length: 0,
        text: paramText,
      );
    }

    if (inner.startsWith('[')) {
      final bracketOffset = source.indexOf('[', params.leftParenthesis.offset);
      return InsertionPlan(
        offset: bracketOffset + 1,
        length: 0,
        text: paramText,
      );
    }
  }

  final buffer = StringBuffer();
  if (hasParams) {
    buffer.writeln(',');
    buffer.write('    ');
  }
  buffer.write(paramText);

  return InsertionPlan(
    offset: findLastParameterOffset(constructor),
    length: 0,
    text: buffer.toString(),
  );
}

/// Plans insertion of [argumentText] after the last argument in [argumentList].
InsertionPlan planArgumentInsertion(
  String source,
  ArgumentList argumentList,
  String argumentText, {
  String indent = '      ',
}) {
  final hasArgs = argumentList.arguments.isNotEmpty;
  if (!hasArgs) {
    return InsertionPlan(
      offset: argumentList.leftParenthesis.end,
      length: 0,
      text: argumentText,
    );
  }

  final buffer = StringBuffer()
    ..writeln(',')
    ..write(indent)
    ..write(argumentText);

  return InsertionPlan(
    offset: findLastArgumentInsertOffset(source, argumentList),
    length: 0,
    text: buffer.toString(),
  );
}
