/// A single navigation step in an AST path.
class NavigateStep {
  /// Creates a navigation step.
  const NavigateStep(this.kind, {this.name});

  /// Step kind (for example `class`, `method`, `ctor`).
  final NavigateKind kind;

  /// Optional name or URI depending on [kind].
  final String? name;

  @override
  String toString() => name == null ? kind.name : '${kind.name}:$name';

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is NavigateStep && kind == other.kind && name == other.name;

  @override
  int get hashCode => Object.hash(kind, name);
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
}

/// A resolved insertion anchor within the focused node (v1).
class Anchor {
  /// Creates an anchor token.
  const Anchor(this.kind);

  /// Anchor kind (for example `stmt:last`, `member:last`).
  final AnchorKind kind;

  @override
  String toString() => kind.name;

  @override
  bool operator ==(Object other) =>
      identical(this, other) || other is Anchor && kind == other.kind;

  @override
  int get hashCode => kind.hashCode;
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
