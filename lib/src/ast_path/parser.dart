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
  if (entry is String) {
    return _parseNavigateToken(entry);
  }

  if (entry is Map) {
    String? match;
    NavigateStep? step;

    for (final key in entry.keys) {
      if (key == 'match') {
        final value = entry[key];
        if (value == null) {
          throw AstPathParseException('Navigate "match" must not be null');
        }
        match = value.toString();
        continue;
      }

      if (step != null) {
        throw AstPathParseException(
          'Navigate map entry must have one step key plus optional "match", got $entry',
        );
      }

      if (key is! String) {
        throw AstPathParseException('Navigate key must be a string, got $key');
      }

      final value = entry[key];
      final name = value == null ? null : value.toString();
      step = _navigateStepForKey(key, name);
    }

    if (step == null) {
      throw AstPathParseException('Navigate map entry is empty: $entry');
    }

    return NavigateStep(step.kind, name: step.name, match: match);
  }

  throw AstPathParseException(
    'Navigate entry must be a string or map, got $entry',
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
    // Type-inferred navigation: bare identifier, kind is null
    return NavigateStep(null, name: trimmed);
  }

  final key = trimmed.substring(0, colonIndex);
  final value = trimmed.substring(colonIndex + 1).trim();
  final name = value.isEmpty ? null : value;
  return _navigateStepForKey(key, name);
}

NavigateStep _navigateStepForKey(String key, String? name) {
  return switch (key) {
    'class' => NavigateStep(
      NavigateKind.classDecl,
      name: _requireName(name, key),
    ),
    'method' => NavigateStep(
      NavigateKind.method,
      name: _requireName(name, key),
    ),
    'ctor' => NavigateStep(NavigateKind.constructor, name: name),
    'call' => NavigateStep(NavigateKind.call, name: _requireName(name, key)),
    'import' => NavigateStep(
      NavigateKind.import,
      name: _requireName(name, key),
    ),
    'field' => NavigateStep(NavigateKind.field, name: _requireName(name, key)),
    'function' => NavigateStep(
      NavigateKind.function,
      name: _requireName(name, key),
    ),
    'var' || 'variable' => NavigateStep(
      NavigateKind.variable,
      name: _requireName(name, key),
    ),
    'initializer' => NavigateStep(
      NavigateKind.initializer,
      name: name,
    ),
    'redirection' => NavigateStep(
      NavigateKind.redirection,
      name: name,
    ),
    _ => throw AstPathParseException('Unknown navigate step "$key"'),
  };
}

String _requireName(String? name, String key) {
  if (name == null || name.isEmpty) {
    throw AstPathParseException('Navigate step "$key" requires a name');
  }
  return name;
}
