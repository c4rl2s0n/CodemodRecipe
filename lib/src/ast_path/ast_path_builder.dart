import 'model.dart';

/// A fluent builder for constructing [AstPath] instances.
/// 
/// This class provides a more readable and type-safe way to build AST paths
/// compared to manually constructing [NavigateStep] lists and [Anchor] objects.
/// 
/// Example usage:
/// ```dart
/// final path = AstPathBuilder()
///   .navigateToClass('Settings')
///   .thenToMethod('update')
///   .atAnchor(AnchorKind.stmtLast)
///   .build();
/// ```
class AstPathBuilder {
  final List<NavigateStep> _navigate = [];
  Anchor? _anchor;

  /// Starts navigation from the root.
  AstPathBuilder root() {
    _navigate.add(const NavigateStep(NavigateKind.root));
    return this;
  }

  /// Navigates to a class declaration by name.
  AstPathBuilder navigateToClass(String name, {String? match}) {
    _navigate.add(NavigateStep(
      NavigateKind.classDecl,
      name: name,
      match: match,
    ));
    return this;
  }

  /// Navigates to a method declaration by name.
  AstPathBuilder navigateToMethod(String name, {String? match}) {
    _navigate.add(NavigateStep(
      NavigateKind.method,
      name: name,
      match: match,
    ));
    return this;
  }

  /// Navigates to a constructor.
  /// 
  /// If [name] is null, navigates to the unnamed constructor.
  AstPathBuilder navigateToConstructor({String? name, String? match}) {
    _navigate.add(NavigateStep(
      NavigateKind.constructor,
      name: name,
      match: match,
    ));
    return this;
  }

  /// Navigates to a function call by type name.
  AstPathBuilder navigateToCall(String typeName, {String? match}) {
    _navigate.add(NavigateStep(
      NavigateKind.call,
      name: typeName,
      match: match,
    ));
    return this;
  }

  /// Navigates to an import directive by URI.
  AstPathBuilder navigateToImport(String uri) {
    _navigate.add(NavigateStep(
      NavigateKind.import,
      name: uri,
    ));
    return this;
  }

  /// Navigates to a field declaration by name.
  AstPathBuilder navigateToField(String name, {String? match}) {
    _navigate.add(NavigateStep(
      NavigateKind.field,
      name: name,
      match: match,
    ));
    return this;
  }

  /// Navigates to a function declaration by name.
  AstPathBuilder navigateToFunction(String name, {String? match}) {
    _navigate.add(NavigateStep(
      NavigateKind.function,
      name: name,
      match: match,
    ));
    return this;
  }

  /// Navigates to a variable declaration by name.
  AstPathBuilder navigateToVariable(String name, {String? match}) {
    _navigate.add(NavigateStep(
      NavigateKind.variable,
      name: name,
      match: match,
    ));
    return this;
  }

  /// Navigates using type inference (kind is null).
  /// 
  /// This will search for the best matching node by name.
  AstPathBuilder navigateToInferred(String name, {String? match}) {
    _navigate.add(NavigateStep(
      null,
      name: name,
      match: match,
    ));
    return this;
  }

  /// Navigates to a constructor's initializer list.
  AstPathBuilder navigateToInitializer({String? match}) {
    _navigate.add(NavigateStep(
      NavigateKind.initializer,
      match: match,
    ));
    return this;
  }

  /// Navigates to a constructor's redirection target.
  AstPathBuilder navigateToRedirection({String? match}) {
    _navigate.add(NavigateStep(
      NavigateKind.redirection,
      match: match,
    ));
    return this;
  }

  /// Sets the insertion anchor.
  AstPathBuilder atAnchor(AnchorKind kind, {String? name, int? index}) {
    _anchor = Anchor(kind, name: name, index: index);
    return this;
  }

  /// Sets the anchor to a specific named anchor.
  AstPathBuilder atParamName(String name) {
    _anchor = Anchor(AnchorKind.paramName, name: name);
    return this;
  }

  /// Sets the anchor to a specific argument name.
  AstPathBuilder atArgName(String name) {
    _anchor = Anchor(AnchorKind.argName, name: name);
    return this;
  }

  /// Sets the anchor to a specific parameter index.
  AstPathBuilder atParamIndex(int index) {
    _anchor = Anchor(AnchorKind.paramIndex, index: index);
    return this;
  }

  /// Sets the anchor to a specific argument index.
  AstPathBuilder atArgIndex(int index) {
    _anchor = Anchor(AnchorKind.argIndex, index: index);
    return this;
  }

  /// Sets the anchor to the beginning of the body.
  AstPathBuilder atBodyStart() {
    _anchor = const Anchor(AnchorKind.bodyStart);
    return this;
  }

  /// Sets the anchor to the end of the body.
  AstPathBuilder atBodyEnd() {
    _anchor = const Anchor(AnchorKind.bodyEnd);
    return this;
  }

  /// Sets the anchor to the last statement.
  AstPathBuilder atStmtLast() {
    _anchor = const Anchor(AnchorKind.stmtLast);
    return this;
  }

  /// Sets the anchor to the last member.
  AstPathBuilder atMemberLast() {
    _anchor = const Anchor(AnchorKind.memberLast);
    return this;
  }

  /// Sets the anchor to the last parameter.
  AstPathBuilder atParamLast() {
    _anchor = const Anchor(AnchorKind.paramLast);
    return this;
  }

  /// Sets the anchor to the last argument.
  AstPathBuilder atArgLast() {
    _anchor = const Anchor(AnchorKind.argLast);
    return this;
  }

  /// Sets the anchor before metadata.
  AstPathBuilder atMetaBefore() {
    _anchor = const Anchor(AnchorKind.metaBefore);
    return this;
  }

  /// Sets the anchor before documentation.
  AstPathBuilder atDocBefore() {
    _anchor = const Anchor(AnchorKind.docBefore);
    return this;
  }

  /// Sets the anchor after documentation.
  AstPathBuilder atDocAfter() {
    _anchor = const Anchor(AnchorKind.docAfter);
    return this;
  }

  /// Builds the [AstPath] from the configured navigation and anchor.
  /// 
  /// Throws a [StateError] if no anchor has been set.
  AstPath build() {
    if (_anchor == null) {
      throw StateError('Anchor must be set before building the path');
    }
    return AstPath(navigate: List.unmodifiable(_navigate), anchor: _anchor!);
  }

  /// Returns the number of navigation steps configured.
  int get stepCount => _navigate.length;

  /// Returns whether an anchor has been set.
  bool get hasAnchor => _anchor != null;
}
