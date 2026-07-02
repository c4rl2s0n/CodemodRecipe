import 'model.dart';

/// Parses navigation step tokens and creates [NavigateStep] instances.
///
/// This class consolidates duplicate parsing logic that was previously
/// duplicated in both [AstPathParser] and [ClassFocusResolver].
class NavigateParser {
  /// Parses a navigation step from a key-value pair.
  ///
  /// Examples:
  /// - `class:MyClass` -> ClassDeclaration navigation
  /// - `method:update` -> MethodDeclaration navigation
  /// - `ctor` -> Constructor navigation (unnamed)
  static NavigateStep stepForKey(String key, String? name) {
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
      'field' => NavigateStep(
        NavigateKind.field,
        name: _requireName(name, key),
      ),
      'function' => NavigateStep(
        NavigateKind.function,
        name: _requireName(name, key),
      ),
      'var' || 'variable' => NavigateStep(
        NavigateKind.variable,
        name: _requireName(name, key),
      ),
      'initializer' => NavigateStep(NavigateKind.initializer, name: name),
      'redirection' => NavigateStep(NavigateKind.redirection, name: name),
      _ => throw FormatException('Unknown navigate step "$key"'),
    };
  }

  /// Parses a navigation token string into a [NavigateStep].
  ///
  /// Examples:
  /// - `class:Settings` -> ClassDeclaration navigation for "Settings"
  /// - `method:update` -> MethodDeclaration navigation for "update"
  /// - `root` or `.` -> Root navigation
  /// - `Settings` (no colon) -> Type-inferred navigation
  static NavigateStep parseToken(String token) {
    final trimmed = token.trim();
    if (trimmed.isEmpty) {
      throw FormatException('Navigate token must not be empty');
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
    return stepForKey(key, name);
  }

  /// Parses a navigation entry (either a string or a map) into a [NavigateStep].
  ///
  /// For maps, supports a "match" key for filtering and one navigation key.
  /// Example: `{class: MyClass, match: "final"}`
  static NavigateStep parseEntry(Object? entry) {
    if (entry is String) {
      return parseToken(entry);
    }

    if (entry is Map) {
      String? match;
      NavigateStep? step;

      for (final key in entry.keys) {
        if (key == 'match') {
          match = entry[key]?.toString();
          continue;
        }
        if (step != null) {
          throw FormatException(
            'Navigate map entry must have one step key plus optional "match"',
          );
        }
        final name = entry[key]?.toString();
        step = stepForKey(key.toString(), name);
      }

      if (step == null) {
        throw FormatException('Navigate map entry is empty');
      }

      return NavigateStep(step.kind, name: step.name, match: match);
    }

    throw FormatException('Navigate entry must be a string or map, got $entry');
  }

  /// Requires a non-null, non-empty name for the given step key.
  static String _requireName(String? name, String key) {
    if (name == null || name.isEmpty) {
      throw FormatException('Navigate step "$key" requires a name');
    }
    return name;
  }
}
