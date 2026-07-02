import 'package:analyzer/dart/ast/ast.dart';

/// Returns the declared class name lexeme.
String classNameLexeme(ClassDeclaration node) => node.namePart.typeName.lexeme;

/// Returns members from a block class body.
Iterable<ClassMember> classMembers(ClassDeclaration node) {
  return switch (node.body) {
    BlockClassBody(:final members) => members,
    _ => const <ClassMember>[],
  };
}

/// Returns the offset immediately after the class opening brace.
int classBodyLeftBracketEnd(ClassDeclaration node) {
  return switch (node.body) {
    BlockClassBody(:final leftBracket) => leftBracket.end,
    _ => node.offset,
  };
}

/// Returns the method name lexeme.
String methodNameLexeme(MethodDeclaration node) => node.name.lexeme;

/// Returns the offset of the declaration keyword for [node].
int declarationKeywordOffset(AstNode node) {
  if (node is ClassDeclaration) {
    return node.classKeyword.offset;
  }
  if (node is MethodDeclaration) {
    return node.returnType?.offset ?? node.name.offset;
  }
  if (node is ConstructorDeclaration) {
    return node.returnType.offset;
  }
  if (node is FieldDeclaration) {
    return node.fields.keyword?.offset ??
        node.fields.variables.first.name.offset;
  }
  return node.offset;
}

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
  return classBodyLeftBracketEnd(classNode);
}

/// Returns a stable insertion offset for adding a new class member.
///
/// Prefers inserting after the last method, then after the last field,
/// then at the beginning of the class body.
int findOptimalInsertionOffset(ClassDeclaration classNode) {
  final methods = classMembers(classNode).whereType<MethodDeclaration>().toList();
  if (methods.isNotEmpty) {
    return methods.last.end;
  }

  final fields = classMembers(classNode).whereType<FieldDeclaration>().toList();
  if (fields.isNotEmpty) {
    return fields.last.end;
  }

  return findClassBodyStartOffset(classNode);
}
