import 'package:analyzer/dart/ast/ast.dart';

import '../dart_codegen/ast_helpers/offsets.dart';
import 'model.dart';

/// v1 allowlist for full declaration-span remove/replace.
bool isDeclarationSpanAllowed(AstNode node) {
  return node is FieldDeclaration ||
      node is MethodDeclaration ||
      node is ImportDirective ||
      node is ConstructorDeclaration ||
      node is FunctionDeclaration ||
      node is TopLevelVariableDeclaration;
}

/// Returns a trivia-inclusive span for removing/replacing [node] in [source].
AnchorSpan declarationSpan(String source, AstNode node) {
  if (node is ClassDeclaration) {
    throw StateError(
      'remove/replace on ClassDeclaration is not supported in v1',
    );
  }
  if (!isDeclarationSpanAllowed(node)) {
    throw StateError(
      'remove/replace does not support ${node.runtimeType}',
    );
  }

  final start = _fullDeclarationStart(source, node);
  final end = _fullDeclarationEnd(source, node);
  return AnchorSpan(offset: start, length: end - start);
}

int _fullDeclarationStart(String source, AstNode node) {
  final keywordOffset = _declarationKeywordOffset(node);
  final docStart = _docBlockStart(source, keywordOffset);
  final metadata = _metadataList(node);
  final metaStart = metadata.isNotEmpty
      ? metadata.first.offset
      : keywordOffset;

  var start = keywordOffset;
  if (docStart != null && docStart < start) {
    start = docStart;
  }
  if (metaStart < start) {
    start = metaStart;
  }
  return start;
}

int _fullDeclarationEnd(String source, AstNode node) {
  var end = node.end;
  if (end < source.length && source[end] == ';') {
    end++;
  }
  return skipTrailingComma(source, end, source.length);
}

int _declarationKeywordOffset(AstNode node) => declarationKeywordOffset(node);

List<Annotation> _metadataList(AstNode node) {
  return switch (node) {
    ClassDeclaration n => n.metadata,
    MethodDeclaration n => n.metadata,
    ConstructorDeclaration n => n.metadata,
    FieldDeclaration n => n.metadata,
    _ => const [],
  };
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

/// Whether [anchor] resolves to a zero-length insertion point.
bool isPointAnchor(Anchor anchor) {
  return switch (anchor.kind) {
    AnchorKind.initializerReplace => false,
    _ => true,
  };
}
