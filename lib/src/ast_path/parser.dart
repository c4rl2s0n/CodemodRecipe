import 'model.dart';
import 'navigate_parser.dart';
import '../core/errors.dart';

/// Parses structured YAML-style path maps/lists into [AstPath].
///
/// Accepts either:
/// - `{ navigate: [...], anchor: 'stmt:last' }`
/// - `{ at: [...], anchor: 'stmt:last' }` where the last navigate step may
///   be omitted when [anchor] is provided separately.
AstPath parseStructuredPath(Map<Object?, Object?> map) {
  final navigateValue = map['navigate'] ?? map['at'];
  if (navigateValue is! List) {
    throw AstPathParseException(
      'Expected "navigate" or "at" to be a list, got $navigateValue',
    );
  }

  final anchorValue = map['anchor'];
  if (anchorValue is! String) {
    throw AstPathParseException(
      'Expected "anchor" to be a string, got $anchorValue',
    );
  }

  return AstPath(
    navigate: [for (final step in navigateValue) _parseNavigateEntry(step)],
    anchor: parseAnchor(anchorValue),
  );
}

/// Parses a path string such as `class:Settings > method:update @ stmt:last`.
AstPath parsePathString(String input) {
  final trimmed = input.trim();
  if (trimmed.isEmpty) {
    throw AstPathParseException('Path string must not be empty');
  }

  final atIndex = trimmed.lastIndexOf('@');
  if (atIndex < 0) {
    throw AstPathParseException('Path string must contain "@" before anchor');
  }

  final navigatePart = trimmed.substring(0, atIndex).trim();
  final anchorPart = trimmed.substring(atIndex + 1).trim();
  if (anchorPart.isEmpty) {
    throw AstPathParseException('Anchor must not be empty');
  }

  final steps = <NavigateStep>[];
  if (navigatePart.isNotEmpty) {
    for (final segment in navigatePart.split('>')) {
      steps.add(_parseNavigateToken(segment.trim()));
    }
  }

  return AstPath(navigate: steps, anchor: parseAnchor(anchorPart));
}

/// Parses an anchor token string such as `stmt:last` or `param:name:key`.
Anchor parseAnchor(String token) {
  final normalized = token.trim();
  if (normalized.isEmpty) {
    throw AstPathParseException('Anchor token must not be empty');
  }

  if (normalized.startsWith('param:name:')) {
    final name = normalized.substring('param:name:'.length).trim();
    if (name.isEmpty) {
      throw AstPathParseException(
        'Anchor "param:name:" requires a parameter name',
      );
    }
    return Anchor(AnchorKind.paramName, name: name);
  }

  if (normalized.startsWith('arg:name:')) {
    final name = normalized.substring('arg:name:'.length).trim();
    if (name.isEmpty) {
      throw AstPathParseException(
        'Anchor "arg:name:" requires an argument name',
      );
    }
    return Anchor(AnchorKind.argName, name: name);
  }

  if (normalized.startsWith('initializer:name:')) {
    final name = normalized.substring('initializer:name:'.length).trim();
    if (name.isEmpty) {
      throw AstPathParseException(
        'Anchor "initializer:name:" requires a field name',
      );
    }
    return Anchor(AnchorKind.initializerName, name: name);
  }

  if (normalized.startsWith('redirection:arg:name:')) {
    final name = normalized.substring('redirection:arg:name:'.length).trim();
    if (name.isEmpty) {
      throw AstPathParseException(
        'Anchor "redirection:arg:name:" requires an argument name',
      );
    }
    return Anchor(AnchorKind.redirectionArgName, name: name);
  }

  final colonIndex = normalized.indexOf(':');
  if (colonIndex > 0) {
    final prefix = normalized.substring(0, colonIndex);
    final suffix = normalized.substring(colonIndex + 1);
    final index = int.tryParse(suffix);
    if (index != null) {
      return switch (prefix) {
        'param' => Anchor(AnchorKind.paramIndex, index: index),
        'arg' => Anchor(AnchorKind.argIndex, index: index),
        _ => _parseSimpleAnchor(normalized),
      };
    }
  }

  return _parseSimpleAnchor(normalized);
}

Anchor _parseSimpleAnchor(String normalized) {
  final kind = switch (normalized) {
    'body:start' => AnchorKind.bodyStart,
    'body:end' => AnchorKind.bodyEnd,
    'stmt:last' || r'stmt:$' => AnchorKind.stmtLast,
    'member:last' => AnchorKind.memberLast,
    'param:last' => AnchorKind.paramLast,
    'arg:last' => AnchorKind.argLast,
    'meta:before' => AnchorKind.metaBefore,
    'doc:before' => AnchorKind.docBefore,
    'doc:after' => AnchorKind.docAfter,
    'initializer:replace' => AnchorKind.initializerReplace,
    'initializer:last' => AnchorKind.initializerLast,
    'redirection:arg:last' => AnchorKind.redirectionArgLast,
    _ => throw AstPathParseException('Unknown anchor token "$normalized"'),
  };

  return Anchor(kind);
}

NavigateStep _parseNavigateEntry(Object? entry) {
  return NavigateParser.parseEntry(entry);
}

NavigateStep _parseNavigateToken(String token) {
  return NavigateParser.parseToken(token);
}
