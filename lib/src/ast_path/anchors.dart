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
  return resolveAnchorSpan(source: source, node: node, anchor: anchor).offset;
}

/// Resolves [anchor] to a byte span for [node] within [source].
AnchorSpan resolveAnchorSpan({
  required String source,
  required AstNode node,
  required Anchor anchor,
}) {
  return switch (anchor.kind) {
    AnchorKind.bodyStart => AnchorSpan(offset: _bodyStart(node)),
    AnchorKind.bodyEnd => AnchorSpan(offset: _bodyEnd(node)),
    AnchorKind.stmtLast => AnchorSpan(offset: _stmtLast(node)),
    AnchorKind.memberLast => AnchorSpan(offset: _memberLast(node)),
    AnchorKind.paramLast => AnchorSpan(offset: _paramLast(node)),
    AnchorKind.argLast => AnchorSpan(offset: _argLast(source, node)),
    AnchorKind.metaBefore => AnchorSpan(offset: _metaBefore(node)),
    AnchorKind.paramName => AnchorSpan(
      offset: _namedParamEnd(source, node, anchor.name!),
    ),
    AnchorKind.argName => AnchorSpan(
      offset: _namedArgEnd(source, node, anchor.name!),
    ),
    AnchorKind.paramIndex => AnchorSpan(
      offset: _paramIndexEnd(node, anchor.index!),
    ),
    AnchorKind.argIndex => AnchorSpan(
      offset: _argIndexEnd(source, node, anchor.index!),
    ),
    AnchorKind.docBefore => AnchorSpan(offset: _docBefore(source, node)),
    AnchorKind.docAfter => AnchorSpan(offset: _docAfter(node)),
    AnchorKind.initializerReplace => _initializerReplace(source, node),
  };
}

/// Returns whether [anchor] is valid for a focused [node] of the given type.
bool isAnchorValidFor(AstNode node, Anchor anchor) {
  return switch (anchor.kind) {
    AnchorKind.bodyStart || AnchorKind.bodyEnd || AnchorKind.memberLast =>
      node is ClassDeclaration,
    AnchorKind.stmtLast => node is MethodDeclaration,
    AnchorKind.paramLast ||
    AnchorKind.paramName ||
    AnchorKind.paramIndex =>
      node is ConstructorDeclaration,
    AnchorKind.argLast ||
    AnchorKind.argName ||
    AnchorKind.argIndex =>
      _isCallLike(node),
    AnchorKind.metaBefore ||
    AnchorKind.docBefore ||
    AnchorKind.docAfter =>
      node is ClassDeclaration ||
          node is MethodDeclaration ||
          node is ConstructorDeclaration ||
          node is FieldDeclaration,
    AnchorKind.initializerReplace => node is FieldDeclaration,
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

int _namedParamEnd(String source, AstNode node, String name) {
  if (node is! ConstructorDeclaration) {
    throw StateError(
      'Anchor param:name:$name requires a constructor, got ${node.runtimeType}',
    );
  }

  final parameter = _findNamedFormalParameter(node.parameters, name);
  if (parameter == null) {
    throw StateError('Parameter "$name" not found in constructor');
  }

  return parameter.end;
}

int _namedArgEnd(String source, AstNode node, String name) {
  if (!_isCallLike(node)) {
    throw StateError(
      'Anchor arg:name:$name requires a constructor call, got ${node.runtimeType}',
    );
  }

  final named = findNamedArgument(argumentListOf(node), name);
  if (named == null) {
    throw StateError('Argument "$name" not found in call');
  }

  return skipTrailingComma(source, named.end, argumentListOf(node).end);
}

int _paramIndexEnd(AstNode node, int index) {
  if (node is! ConstructorDeclaration) {
    throw StateError(
      'Anchor param:$index requires a constructor, got ${node.runtimeType}',
    );
  }

  final parameters = node.parameters.parameters;
  if (index < 0 || index >= parameters.length) {
    throw StateError('Parameter index $index out of range');
  }

  return parameters[index].end;
}

int _argIndexEnd(String source, AstNode node, int index) {
  if (!_isCallLike(node)) {
    throw StateError(
      'Anchor arg:$index requires a constructor call, got ${node.runtimeType}',
    );
  }

  final args = argumentListOf(node).arguments;
  if (index < 0 || index >= args.length) {
    throw StateError('Argument index $index out of range');
  }

  return skipTrailingComma(source, args[index].end, argumentListOf(node).end);
}

int _docBefore(String source, AstNode node) {
  final keywordOffset = _declarationKeywordOffset(node);
  return _docBlockStart(source, keywordOffset) ?? keywordOffset;
}

int _docAfter(AstNode node) {
  return _declarationKeywordOffset(node);
}

int _metaBefore(AstNode node) {
  final metadata = _metadataList(node);
  if (metadata.isNotEmpty) {
    return metadata.first.offset;
  }
  return _declarationKeywordOffset(node);
}

int _declarationKeywordOffset(AstNode node) {
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
    return node.fields.keyword?.offset ?? node.fields.variables.first.name.offset;
  }
  return node.offset;
}

List<Annotation> _metadataList(AstNode node) {
  return switch (node) {
    ClassDeclaration n => n.metadata,
    MethodDeclaration n => n.metadata,
    ConstructorDeclaration n => n.metadata,
    FieldDeclaration n => n.metadata,
    _ => const [],
  };
}

AnchorSpan _initializerReplace(String source, AstNode node) {
  if (node is! FieldDeclaration) {
    throw StateError(
      'Anchor initializer:replace requires a field, got ${node.runtimeType}',
    );
  }

  for (final variable in node.fields.variables) {
    final initializer = variable.initializer;
    if (initializer != null) {
      return AnchorSpan(
        offset: initializer.offset,
        length: initializer.end - initializer.offset,
      );
    }
  }

  throw StateError('Field has no initializer to replace');
}

FormalParameter? _findNamedFormalParameter(
  FormalParameterList parameters,
  String name,
) {
  for (final parameter in parameters.parameters) {
    final parameterName = parameter.name?.lexeme;
    if (parameterName == name) {
      return parameter;
    }
  }
  return null;
}

int? _docBlockStart(String source, int declarationOffset) {
  var lineStart = declarationOffset;
  while (lineStart > 0 && source[lineStart - 1] != '\n') {
    lineStart--;
  }

  var scan = lineStart;
  while (scan > 0) {
    final previousLineEnd = scan - 1;
    var previousLineStart = previousLineEnd;
    while (previousLineStart > 0 && source[previousLineStart - 1] != '\n') {
      previousLineStart--;
    }

    final line = source.substring(previousLineStart, previousLineEnd).trimLeft();
    if (!line.startsWith('///')) {
      break;
    }

    scan = previousLineStart;
  }

  return scan == lineStart ? null : scan;
}

bool _isCallLike(AstNode node) {
  return node is InstanceCreationExpression || node is MethodInvocation;
}
