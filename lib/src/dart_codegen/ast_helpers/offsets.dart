import 'package:analyzer/dart/ast/ast.dart';

/// Skips comma characters at [offset] up to [endBound].
int skipTrailingComma(String source, int offset, int endBound) {
  while (offset < endBound && source[offset] == ',') {
    offset++;
  }
  return offset;
}

/// Returns the insertion offset after the last constructor parameter.
int findLastParameterOffset(ConstructorDeclaration constructor) {
  final parameters = constructor.parameters;
  if (parameters.parameters.isEmpty) {
    return parameters.leftParenthesis.end;
  }

  return parameters.parameters.last.end;
}

/// Returns the insertion offset after the last argument in [argumentList].
int findLastArgumentInsertOffset(String source, ArgumentList argumentList) {
  final args = argumentList.arguments;
  if (args.isEmpty) {
    return argumentList.leftParenthesis.end;
  }

  return skipTrailingComma(source, args.last.end, argumentList.end);
}
