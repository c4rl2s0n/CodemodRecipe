import 'package:analyzer/dart/ast/ast.dart';

import '../dart_codegen/ast_helpers/invocations.dart';
import '../dart_codegen/ast_helpers/localizers.dart';
import '../dart_codegen/ast_helpers/offsets.dart';
import 'model.dart';
import 'anchor_validators.dart';

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
    AnchorKind.initializerLast => AnchorSpan(offset: _initializerLast(source, node)),
    AnchorKind.initializerName => AnchorSpan(
      offset: _initializerNameEnd(source, node, anchor.name!),
    ),
    AnchorKind.redirectionArgLast => AnchorSpan(
      offset: _redirectionArgLast(source, node),
    ),
    AnchorKind.redirectionArgName => AnchorSpan(
      offset: _redirectionArgNameEnd(source, node, anchor.name!),
    ),
  };
}

/// Returns whether [anchor] is valid for a focused [node] of the given type.
/// 
/// This function delegates to the strategy pattern implementation in [AnchorValidatorRegistry]
/// for extensible and maintainable validation logic.
bool isAnchorValidFor(AstNode node, Anchor anchor) {
  return AnchorValidatorRegistry.isValidFor(node, anchor);
}

int _bodyStart(AstNode node) {
  if (node is ClassDeclaration) {
    return findClassBodyStartOffset(node);
  }
  if (node is FunctionDeclaration) {
    return _findFunctionBodyStartOffset(node);
  }
  throw StateError(
    'Anchor body:start requires a class or function, got ${node.runtimeType}',
  );
}

int _bodyEnd(AstNode node) {
  if (node is ClassDeclaration) {
    return findClassEndOffset(node);
  }
  if (node is FunctionDeclaration) {
    return _findFunctionBodyEndOffset(node);
  }
  throw StateError(
    'Anchor body:end requires a class or function, got ${node.runtimeType}',
  );
}

int _findFunctionBodyStartOffset(FunctionDeclaration function) {
  final body = function.functionExpression.body;
  if (body is! BlockFunctionBody) {
    throw StateError('Expected block body for function');
  }
  return body.block.leftBracket.end;
}

int _findFunctionBodyEndOffset(FunctionDeclaration function) {
  final body = function.functionExpression.body;
  if (body is! BlockFunctionBody) {
    throw StateError('Expected block body for function');
  }
  return body.block.rightBracket.offset;
}

int _stmtLast(AstNode node) {
  if (node is MethodDeclaration) {
    return findLastStatementInsertOffset(node);
  }
  if (node is FunctionDeclaration) {
    return _findLastStatementInsertOffsetForFunction(node);
  }
  throw StateError(
    'Anchor stmt:last requires a method or function, got ${node.runtimeType}',
  );
}

int _findLastStatementInsertOffsetForFunction(FunctionDeclaration function) {
  final body = function.functionExpression.body;
  if (body is! BlockFunctionBody) {
    throw StateError('Expected block body for function');
  }
  final statements = body.block.statements;
  if (statements.isEmpty) return body.block.leftBracket.end;
  return statements.last.end;
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
  if (node is ConstructorDeclaration) {
    return findLastParameterOffset(node);
  }
  if (node is FunctionDeclaration) {
    return _findLastParameterOffsetForFunction(node);
  }
  throw StateError(
    'Anchor param:last requires a constructor or function, got ${node.runtimeType}',
  );
}

int _findLastParameterOffsetForFunction(FunctionDeclaration function) {
  final parameters = function.functionExpression.parameters?.parameters;
  if (parameters == null || parameters.isEmpty) {
    return function.functionExpression.parameters!.end;
  }
  return parameters.last.end;
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
  if (node is ConstructorDeclaration) {
    final parameter = _findNamedFormalParameter(node.parameters, name);
    if (parameter == null) {
      throw StateError('Parameter "$name" not found in constructor');
    }
    return parameter.end;
  }
  if (node is FunctionDeclaration) {
    final parameter = _findNamedFormalParameterForFunction(node, name);
    if (parameter == null) {
      throw StateError('Parameter "$name" not found in function');
    }
    return parameter.end;
  }
  throw StateError(
    'Anchor param:name:$name requires a constructor or function, got ${node.runtimeType}',
  );
}

FormalParameter? _findNamedFormalParameterForFunction(
  FunctionDeclaration function,
  String name,
) {
  final parameters = function.functionExpression.parameters?.parameters;
  if (parameters == null) return null;
  
  for (final parameter in parameters) {
    final parameterName = parameter.name?.lexeme;
    if (parameterName == name) {
      return parameter;
    }
  }
  return null;
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
  if (node is ConstructorDeclaration) {
    final parameters = node.parameters.parameters;
    if (index < 0 || index >= parameters.length) {
      throw StateError('Parameter index $index out of range');
    }
    return parameters[index].end;
  }
  if (node is FunctionDeclaration) {
    final parameters = node.functionExpression.parameters?.parameters;
    if (parameters == null || index < 0 || index >= parameters.length) {
      throw StateError('Parameter index $index out of range');
    }
    return parameters[index].end;
  }
  throw StateError(
    'Anchor param:$index requires a constructor or function, got ${node.runtimeType}',
  );
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
    return node.fields.keyword?.offset ??
        node.fields.variables.first.name.offset;
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

    final line = source
        .substring(previousLineStart, previousLineEnd)
        .trimLeft();
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

int _initializerLast(String source, AstNode node) {
  if (node is! ConstructorDeclaration) {
    throw StateError(
      'Anchor initializer:last requires a constructor, got ${node.runtimeType}',
    );
  }
  
  // Find the initializer list in the constructor
  // The initializer list is between the parameter list and the body
  // For constructors with initializers: ConstructorDecl(params) : initializerList { body }
  // For constructors with redirection: ConstructorDecl(params) : this(...) { body }
  
  // Get the end of the parameter list
  final paramEnd = node.parameters.end;
  
  // Check if there's a colon after the parameters (indicating initializers or redirection)
  final afterParams = source.substring(paramEnd);
  if (!afterParams.trim().startsWith(':')) {
    // No initializer list, return paramEnd
    return paramEnd;
  }
  
  // Find the start of the initializer list
  final colonIndex = source.indexOf(':', paramEnd);
  if (colonIndex < 0) {
    throw StateError('No initializer list found in constructor');
  }
  
  // Find the end of the initializer list (either { or ,)
  // The initializer list ends at the opening brace or at a comma followed by { 
  var scan = colonIndex + 1;
  var braceDepth = 0;
  var inInitializer = true;
  
  while (scan < source.length && inInitializer) {
    final char = source[scan];
    
    if (char == '{') {
      if (braceDepth == 0) {
        // Found the opening brace of the body
        inInitializer = false;
      } else {
        braceDepth++;
      }
    } else if (char == '}') {
      braceDepth--;
    } else if (char == ',' && braceDepth == 0) {
      // Found a comma at the top level, continue to find the last initializer
      // This means there are multiple initializers
      scan++;
      continue;
    }
    
    scan++;
  }
  
  // Return the position after the last initializer
  // We need to find the actual end of the initializer list
  // For now, return the position before the opening brace
  if (scan < source.length && source[scan] == '{') {
    return scan;
  }
  
  return paramEnd;
}

int _initializerNameEnd(String source, AstNode node, String name) {
  if (node is! ConstructorDeclaration) {
    throw StateError(
      'Anchor initializer:name:$name requires a constructor, got ${node.runtimeType}',
    );
  }
  
  // Find the initializer that matches the field name
  // For now, return the end of the parameter list as a placeholder
  // Full implementation would require parsing the initializer list
  return node.parameters.end;
}

int _redirectionArgLast(String source, AstNode node) {
  if (node is! ConstructorDeclaration) {
    throw StateError(
      'Anchor redirection:arg:last requires a constructor, got ${node.runtimeType}',
    );
  }
  
  // Check if this is a redirecting factory constructor
  if (node.redirectedConstructor != null) {
    // The redirection is : this(...) or : super(...)
    // We need to find the argument list in the redirection
    final redirect = node.redirectedConstructor!;
    
    // Find the offset of the redirected constructor call
    // For now, return the end of the constructor's parameter list
    return node.parameters.end;
  }
  
  throw StateError('No redirection found in constructor');
}

int _redirectionArgNameEnd(String source, AstNode node, String name) {
  if (node is! ConstructorDeclaration) {
    throw StateError(
      'Anchor redirection:arg:name:$name requires a constructor, got ${node.runtimeType}',
    );
  }
  
  // Check if this is a redirecting factory constructor
  if (node.redirectedConstructor != null) {
    // For now, return the end of the constructor's parameter list
    return node.parameters.end;
  }
  
  throw StateError('No redirection found in constructor');
}
