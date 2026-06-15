import 'package:analyzer/dart/ast/ast.dart';

import '../dart_codegen/ast_helpers/invocations.dart';
import '../dart_codegen/ast_helpers/localizers.dart';
import '../dart_codegen/ast_helpers/offsets.dart';
import 'model.dart';

/// Resolves [anchor] to a byte offset for [node] within [source].
int resolveAnchorOffset({
  required String source,
  required AstNode node,
  required Anchor anchor,
}) {
  return switch (anchor.kind) {
    AnchorKind.bodyStart => _bodyStart(node),
    AnchorKind.bodyEnd => _bodyEnd(node),
    AnchorKind.stmtLast => _stmtLast(node),
    AnchorKind.memberLast => _memberLast(node),
    AnchorKind.paramLast => _paramLast(node),
    AnchorKind.argLast => _argLast(source, node),
    AnchorKind.metaBefore => node.offset,
  };
}

/// Returns whether [anchor] is valid for a focused [node] of the given type.
bool isAnchorValidFor(AstNode node, Anchor anchor) {
  return switch (anchor.kind) {
    AnchorKind.bodyStart || AnchorKind.bodyEnd || AnchorKind.memberLast =>
      node is ClassDeclaration,
    AnchorKind.stmtLast => node is MethodDeclaration,
    AnchorKind.paramLast => node is ConstructorDeclaration,
    AnchorKind.argLast => _isCallLike(node),
    AnchorKind.metaBefore =>
      node is ClassDeclaration ||
          node is MethodDeclaration ||
          node is ConstructorDeclaration ||
          node is FieldDeclaration,
  };
}

int _bodyStart(AstNode node) {
  if (node is! ClassDeclaration) {
    throw StateError('Anchor body:start requires a class, got ${node.runtimeType}');
  }
  return findClassBodyStartOffset(node);
}

int _bodyEnd(AstNode node) {
  if (node is! ClassDeclaration) {
    throw StateError('Anchor body:end requires a class, got ${node.runtimeType}');
  }
  return findClassEndOffset(node);
}

int _stmtLast(AstNode node) {
  if (node is! MethodDeclaration) {
    throw StateError('Anchor stmt:last requires a method, got ${node.runtimeType}');
  }
  return findLastStatementInsertOffset(node);
}

int _memberLast(AstNode node) {
  if (node is! ClassDeclaration) {
    throw StateError(
      'Anchor member:last requires a class, got ${node.runtimeType}',
    );
  }
  return findOptimalInsertionOffset(node);
}

int _paramLast(AstNode node) {
  if (node is! ConstructorDeclaration) {
    throw StateError(
      'Anchor param:last requires a constructor, got ${node.runtimeType}',
    );
  }
  return findLastParameterOffset(node);
}

int _argLast(String source, AstNode node) {
  if (!_isCallLike(node)) {
    throw StateError(
      'Anchor arg:last requires a constructor call, got ${node.runtimeType}',
    );
  }
  return findLastArgumentInsertOffset(source, argumentListOf(node));
}

bool _isCallLike(AstNode node) {
  return node is InstanceCreationExpression || node is MethodInvocation;
}
