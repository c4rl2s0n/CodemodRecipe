/// A single navigation step in an AST path.
class NavigateStep {
  /// Creates a navigation step.
  const NavigateStep(this.kind, {this.name, this.match});

  /// Step kind (for example `class`, `method`, `ctor`).
  final NavigateKind kind;

  /// Optional name or URI depending on [kind].
  final String? name;

  /// Optional source substring filter when multiple candidates match [name].
  final String? match;

  @override
  String toString() {
    final label = name == null ? kind.name : '${kind.name}:$name';
    if (match == null) return label;
    return '$label (match: $match)';
  }

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is NavigateStep &&
          kind == other.kind &&
          name == other.name &&
          match == other.match;

  @override
  int get hashCode => Object.hash(kind, name, match);
}

/// Supported navigation step kinds (v1).
enum NavigateKind {
  /// Compilation unit root.
  root,

  /// Class declaration by simple name.
  classDecl,

  /// Method declaration by name in the focused class.
  method,

  /// Constructor in the focused class (`ctor` with optional name).
  constructor,

  /// Constructor-like call by type name under the current node.
  call,

  /// Import directive by URI string.
  import,

  /// Field declaration by variable name in the focused class.
  field,
}

/// A resolved insertion anchor within the focused node (v1).
class Anchor {
  /// Creates an anchor token.
  const Anchor(this.kind, {this.name, this.index});

  /// Anchor kind (for example `stmt:last`, `member:last`).
  final AnchorKind kind;

  /// Named slot for [AnchorKind.paramName] and [AnchorKind.argName].
  final String? name;

  /// Positional index for [AnchorKind.paramIndex] and [AnchorKind.argIndex].
  final int? index;

  @override
  String toString() {
    return switch (kind) {
      AnchorKind.paramName => 'param:name:$name',
      AnchorKind.argName => 'arg:name:$name',
      AnchorKind.paramIndex => 'param:$index',
      AnchorKind.argIndex => 'arg:$index',
      _ => kind.name,
    };
  }

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is Anchor &&
          kind == other.kind &&
          name == other.name &&
          index == other.index;

  @override
  int get hashCode => Object.hash(kind, name, index);
}

/// Supported anchor tokens (v1).
enum AnchorKind {
  bodyStart,
  bodyEnd,
  stmtLast,
  memberLast,
  paramLast,
  argLast,
  metaBefore,
  paramName,
  argName,
  paramIndex,
  argIndex,
  docBefore,
  docAfter,
  initializerReplace,
}

/// Byte range resolved from an anchor (length 0 for pure insertion).
class AnchorSpan {
  /// Creates an anchor span.
  const AnchorSpan({required this.offset, this.length = 0});

  /// Start offset in source.
  final int offset;

  /// Non-zero length for replace-style anchors such as [AnchorKind.initializerReplace].
  final int length;
}

/// Navigate steps plus an anchor — the canonical AST path model.
class AstPath {
  /// Creates an AST path.
  const AstPath({required this.navigate, required this.anchor});

  /// Ordered navigation from compilation unit root to a target node.
  final List<NavigateStep> navigate;

  /// Insertion anchor relative to the focused node.
  final Anchor anchor;

  @override
  String toString() => 'AstPath($navigate @ $anchor)';

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is AstPath &&
          _listEquals(navigate, other.navigate) &&
          anchor == other.anchor;

  @override
  int get hashCode => Object.hash(Object.hashAll(navigate), anchor);
}

bool _listEquals<T>(List<T> a, List<T> b) {
  if (a.length != b.length) return false;
  for (var i = 0; i < a.length; i++) {
    if (a[i] != b[i]) return false;
  }
  return true;
}
