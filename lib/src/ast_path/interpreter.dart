import 'package:analyzer/dart/ast/ast.dart';

import '../dart_codegen/ast_helpers/ast_focus.dart';
import '../dart_codegen/ast_helpers/planners.dart';
import 'anchors.dart';
import 'model.dart';

/// Thrown when navigation or anchor resolution fails at runtime.
class AstPathResolutionException implements Exception {
  /// Creates a resolution exception.
  AstPathResolutionException(this.message, {this.code});

  /// Human-readable failure description.
  final String message;

  /// Optional stable error code for tooling.
  final String? code;

  @override
  String toString() {
    if (code == null) return 'AstPathResolutionException: $message';
    return 'AstPathResolutionException($code): $message';
  }
}

/// Navigates Dart source using [AstPath] and resolves insertion offsets.
class AstPathInterpreter {
  /// Resolves [path] to a byte offset in [source].
  int resolveOffset(
    String source,
    AstPath path, {
    String filePath = '<unknown>',
  }) {
    final focus = navigateTo(source, path.navigate, filePath: filePath);
    final node = focus.node;

    if (!isAnchorValidFor(node, path.anchor)) {
      throw AstPathResolutionException(
        "Anchor '${path.anchor}' invalid for focused node ${node.runtimeType}",
        code: 'E_ANCHOR_INVALID',
      );
    }

    return resolveAnchorOffset(source: source, node: node, anchor: path.anchor);
  }

  /// Navigates [source] using [steps] and returns the focused node.
  AstFocus navigateTo(
    String source,
    List<NavigateStep> steps, {
    String filePath = '<unknown>',
  }) {
    return _navigate(source, steps, filePath: filePath);
  }

  /// Resolves [path] to an insertion plan for [text] at the anchored offset.
  InsertionPlan resolveInsertionPlan(
    String source,
    AstPath path,
    String text, {
    String filePath = '<unknown>',
  }) {
    return InsertionPlan(
      offset: resolveOffset(source, path, filePath: filePath),
      text: text,
    );
  }

  AstFocus _navigate(
    String source,
    List<NavigateStep> steps, {
    required String filePath,
  }) {
    var focus = AstFocus.parse(source, path: filePath);

    for (final step in steps) {
      focus = _applyStep(focus, step);
    }

    return focus;
  }

  AstFocus _applyStep(AstFocus focus, NavigateStep step) {
    return switch (step.kind) {
      NavigateKind.root => focus,
      NavigateKind.classDecl => _classNamed(focus, step.name!),
      NavigateKind.method => _methodNamed(focus, step.name!),
      NavigateKind.constructor => _constructor(focus, step.name),
      NavigateKind.call => _call(focus, step.name!),
      NavigateKind.import => _import(focus, step.name!),
    };
  }

  AstFocus _classNamed(AstFocus focus, String name) {
    try {
      return focus.classNamed(name);
    } on StateError catch (error) {
      throw AstPathResolutionException(
        error.message,
        code: 'E_NODE_NOT_FOUND',
      );
    }
  }

  AstFocus _methodNamed(AstFocus focus, String name) {
    try {
      return focus.methodNamed(name);
    } on StateError catch (error) {
      throw AstPathResolutionException(
        error.message,
        code: 'E_NODE_NOT_FOUND',
      );
    }
  }

  AstFocus _constructor(AstFocus focus, String? name) {
    try {
      return focus.constructor(name: name);
    } on StateError catch (error) {
      throw AstPathResolutionException(
        error.message,
        code: 'E_NODE_NOT_FOUND',
      );
    }
  }

  AstFocus _call(AstFocus focus, String typeName) {
    try {
      return focus.instanceCreation(typeName);
    } on StateError catch (error) {
      throw AstPathResolutionException(
        error.message,
        code: 'E_NODE_NOT_FOUND',
      );
    }
  }

  AstFocus _import(AstFocus focus, String uri) {
    final imports = focus.unit.directives.whereType<ImportDirective>();
    for (final directive in imports) {
      if (directive.uri.stringValue == uri) {
        return AstFocus(focus.source, focus.unit, directive);
      }
    }

    throw AstPathResolutionException(
      'Import "$uri" not found',
      code: 'E_NODE_NOT_FOUND',
    );
  }
}
