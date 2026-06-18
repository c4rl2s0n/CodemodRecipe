import 'package:analyzer/dart/ast/ast.dart';

/// Skips comma characters at [offset] up to [endBound].
int skipTrailingComma(String source, int offset, int endBound) {
  while (offset < endBound && source[offset] == ',') {
    offset++;
  }
  return offset;
}

// ==================== Constructor/Method Parameter Offsets ====================

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

int findLastStatementInsertOffset(MethodDeclaration method) {
  final body = method.body;
  if (body is! BlockFunctionBody) {
    throw StateError('Expected block body');
  }
  final statements = body.block.statements;
  if (statements.isEmpty) return body.block.leftBracket.end;
  return statements.last.end;
}

// ==================== Class Offsets ====================

/// Returns the source offset of [classNode]'s closing brace token.
int findClassEndOffset(ClassDeclaration classNode) {
  return classNode.endToken.offset;
}

/// Returns the insertion offset immediately after the class opening brace.
int findClassBodyStartOffset(ClassDeclaration classNode) {
  return classNode.leftBracket.end;
}

/// Returns a stable insertion offset for adding a new class member.
/// 
/// Prefers inserting after the last method, then after the last field,
/// then at the beginning of the class body.
int findOptimalInsertionOffset(ClassDeclaration classNode) {
  final methods = classNode.members.whereType<MethodDeclaration>().toList();
  if (methods.isNotEmpty) {
    return methods.last.end;
  }

  final fields = classNode.members.whereType<FieldDeclaration>().toList();
  if (fields.isNotEmpty) {
    return fields.last.end;
  }

  return findClassBodyStartOffset(classNode);
}
