import 'package:analyzer/dart/ast/ast.dart';
import 'package:analyzer/dart/ast/visitor.dart';

/// Finds a constructor-like call named [typeName] under [root].
///
/// Matches [InstanceCreationExpression] and unresolved [MethodInvocation]
/// calls such as `Settings(...)`.
///
/// When [returnExpressionOnly] is true, only matches a call that is the
/// direct expression of a [ReturnStatement].
AstNode? findConstructorCall(
  AstNode root,
  String typeName, {
  bool returnExpressionOnly = false,
}) {
  AstNode? found;

  if (returnExpressionOnly) {
    root.accept(
      _ReturnConstructorCallFinder(typeName, (node) => found ??= node),
    );
  } else {
    root.accept(_ConstructorCallFinder(typeName, (node) => found ??= node));
  }

  return found;
}

/// Backwards-compatible alias for [findConstructorCall].
AstNode? findInstanceCreation(
  AstNode root,
  String typeName, {
  bool returnExpressionOnly = false,
}) {
  return findConstructorCall(
    root,
    typeName,
    returnExpressionOnly: returnExpressionOnly,
  );
}

/// Returns the [ArgumentList] for a constructor-like [node].
ArgumentList argumentListOf(AstNode node) {
  if (node is InstanceCreationExpression) {
    return node.argumentList;
  }
  if (node is MethodInvocation && node.target == null) {
    return node.argumentList;
  }
  throw StateError('Expected constructor call, got ${node.runtimeType}');
}

/// Returns whether [node] is a constructor-like call named [typeName].
bool isConstructorCallNamed(AstNode node, String typeName) {
  if (node is InstanceCreationExpression) {
    return node.constructorName.type.name.lexeme == typeName;
  }
  if (node is MethodInvocation && node.target == null) {
    return node.methodName.name == typeName;
  }
  return false;
}

/// Returns the named argument [name] in [args], if present.
NamedExpression? findNamedArgument(ArgumentList args, String name) {
  for (final arg in args.arguments) {
    if (arg is NamedExpression && arg.name.label.name == name) {
      return arg;
    }
  }
  return null;
}

/// Returns whether [args] contains a named argument [name].
bool hasNamedArgument(ArgumentList args, String name) {
  return findNamedArgument(args, name) != null;
}

/// Returns the last argument in [args], or null when empty.
Expression? lastArgument(ArgumentList args) {
  if (args.arguments.isEmpty) return null;
  return args.arguments.last;
}

class _ConstructorCallFinder extends RecursiveAstVisitor<void> {
  _ConstructorCallFinder(this.typeName, this.onMatch);

  final String typeName;
  final void Function(AstNode) onMatch;

  @override
  void visitInstanceCreationExpression(InstanceCreationExpression node) {
    if (isConstructorCallNamed(node, typeName)) {
      onMatch(node);
      return;
    }
    super.visitInstanceCreationExpression(node);
  }

  @override
  void visitMethodInvocation(MethodInvocation node) {
    if (isConstructorCallNamed(node, typeName)) {
      onMatch(node);
      return;
    }
    super.visitMethodInvocation(node);
  }
}

class _ReturnConstructorCallFinder extends RecursiveAstVisitor<void> {
  _ReturnConstructorCallFinder(this.typeName, this.onMatch);

  final String typeName;
  final void Function(AstNode) onMatch;

  @override
  void visitReturnStatement(ReturnStatement node) {
    final expr = node.expression;
    if (expr != null && isConstructorCallNamed(expr, typeName)) {
      onMatch(expr);
    }
  }
}
