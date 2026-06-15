import 'model.dart';

/// Thrown when an AST path string or structured value cannot be parsed.
class AstPathParseException implements Exception {
  /// Creates a parse exception.
  AstPathParseException(this.message);

  /// Human-readable parse failure description.
  final String message;

  @override
  String toString() => 'AstPathParseException: $message';
}

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
    navigate: [
      for (final step in navigateValue) _parseNavigateEntry(step),
    ],
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

/// Parses an anchor token string such as `stmt:last` or `stmt:$`.
Anchor parseAnchor(String token) {
  final normalized = token.trim();
  if (normalized.isEmpty) {
    throw AstPathParseException('Anchor token must not be empty');
  }

  final kind = switch (normalized) {
    'body:start' => AnchorKind.bodyStart,
    'body:end' => AnchorKind.bodyEnd,
    'stmt:last' || r'stmt:$' => AnchorKind.stmtLast,
    'member:last' => AnchorKind.memberLast,
    'param:last' => AnchorKind.paramLast,
    'arg:last' => AnchorKind.argLast,
    'meta:before' => AnchorKind.metaBefore,
    _ => throw AstPathParseException('Unknown anchor token "$normalized"'),
  };

  return Anchor(kind);
}

NavigateStep _parseNavigateEntry(Object? entry) {
  if (entry is String) {
    return _parseNavigateToken(entry);
  }

  if (entry is Map) {
    if (entry.length != 1) {
      throw AstPathParseException(
        'Navigate map entry must have exactly one key, got $entry',
      );
    }
    final key = entry.keys.first;
    final value = entry.values.first;
    if (key is! String) {
      throw AstPathParseException('Navigate key must be a string, got $key');
    }
    final name = value == null ? null : value.toString();
    return _navigateStepForKey(key, name);
  }

  throw AstPathParseException(
    'Navigate entry must be a string or single-key map, got $entry',
  );
}

NavigateStep _parseNavigateToken(String token) {
  final trimmed = token.trim();
  if (trimmed.isEmpty) {
    throw AstPathParseException('Navigate token must not be empty');
  }

  if (trimmed == '.' || trimmed == 'root') {
    return const NavigateStep(NavigateKind.root);
  }

  final colonIndex = trimmed.indexOf(':');
  if (colonIndex < 0) {
    // Shorthand: bare identifier is treated as a class name at root context.
    return NavigateStep(NavigateKind.classDecl, name: trimmed);
  }

  final key = trimmed.substring(0, colonIndex);
  final value = trimmed.substring(colonIndex + 1).trim();
  final name = value.isEmpty ? null : value;
  return _navigateStepForKey(key, name);
}

NavigateStep _navigateStepForKey(String key, String? name) {
  return switch (key) {
    'class' => NavigateStep(NavigateKind.classDecl, name: _requireName(name, key)),
    'method' => NavigateStep(NavigateKind.method, name: _requireName(name, key)),
    'ctor' => NavigateStep(NavigateKind.constructor, name: name),
    'call' => NavigateStep(NavigateKind.call, name: _requireName(name, key)),
    'import' => NavigateStep(NavigateKind.import, name: _requireName(name, key)),
    _ => throw AstPathParseException('Unknown navigate step "$key"'),
  };
}

String _requireName(String? name, String key) {
  if (name == null || name.isEmpty) {
    throw AstPathParseException('Navigate step "$key" requires a name');
  }
  return name;
}
