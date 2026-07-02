import '../ast_path/ast_path.dart';
import '../core/context.dart';
import '../core/patch_helpers.dart';
import '../core/transform.dart';

/// Patch operation mode for [AstPathPatchTransform].
enum AstPathPatchMode { insert, remove, replace }

/// Target for an AST-path patch: navigation plus optional anchor.
class AstPathPatchTarget {
  const AstPathPatchTarget({required this.navigate, this.anchor});

  final List<NavigateStep> navigate;
  final Anchor? anchor;

  factory AstPathPatchTarget.fromPath(AstPath path) {
    return AstPathPatchTarget(navigate: path.navigate, anchor: path.anchor);
  }

  bool get usesDeclarationSpan => anchor == null;
}

/// Applies insert, replace, or remove edits at an AST-path location.
class AstPathPatchTransform implements CodeTransform {
  const AstPathPatchTransform({
    required this.target,
    required this.mode,
    required this.replacement,
    this.description,
    this.idempotency = const AstPathPatchIdempotency(),
  });

  final AstPathPatchTarget target;
  final AstPathPatchMode mode;
  final String replacement;
  final String? description;
  final AstPathPatchIdempotency idempotency;

  factory AstPathPatchTransform.insert({
    required AstPath path,
    required String text,
    String? description,
    AstPathPatchIdempotency idempotency = const AstPathPatchIdempotency(),
  }) {
    return AstPathPatchTransform(
      target: AstPathPatchTarget.fromPath(path),
      mode: AstPathPatchMode.insert,
      replacement: text,
      description: description ?? 'Insert at $path',
      idempotency: idempotency,
    );
  }

  factory AstPathPatchTransform.remove({
    required AstPathPatchTarget target,
    String? description,
    AstPathPatchIdempotency idempotency = const AstPathPatchIdempotency(),
  }) {
    return AstPathPatchTransform(
      target: target,
      mode: AstPathPatchMode.remove,
      replacement: '',
      description: description ?? 'Remove ${target.navigate}',
      idempotency: idempotency,
    );
  }

  factory AstPathPatchTransform.replace({
    required AstPathPatchTarget target,
    required String text,
    String? description,
    AstPathPatchIdempotency idempotency = const AstPathPatchIdempotency(),
  }) {
    return AstPathPatchTransform(
      target: target,
      mode: AstPathPatchMode.replace,
      replacement: text,
      description: description ?? 'Replace ${target.navigate}',
      idempotency: idempotency,
    );
  }

  @override
  Future<List<SourcePatch>> apply(String source, CodemodContext context) async {
    final interpreter = AstPathInterpreter();

    if (mode == AstPathPatchMode.insert &&
        idempotency.insertTargetExists(source, target, interpreter)) {
      return const [];
    }

    late final AnchorSpan span;
    try {
      span = _resolveSpan(source, interpreter);
    } on AstPathResolutionException catch (error) {
      if (error.code == 'E_NODE_NOT_FOUND' && mode == AstPathPatchMode.remove) {
        return const [];
      }
      rethrow;
    }

    if (idempotency.shouldNoOp(
      mode: mode,
      source: source,
      target: target,
      span: span,
      replacement: replacement,
      interpreter: interpreter,
    )) {
      return const [];
    }

    return [
      SourcePatch(
        span.offset,
        span.length,
        replacement,
        description: description,
      ),
    ];
  }

  AnchorSpan _resolveSpan(String source, AstPathInterpreter interpreter) {
    final focus = interpreter.navigateTo(source, target.navigate);

    if (mode == AstPathPatchMode.insert) {
      final anchor = target.anchor;
      if (anchor == null) {
        throw StateError('insert requires an anchor');
      }
      return interpreter.resolveSpan(
        source,
        AstPath(navigate: target.navigate, anchor: anchor),
      );
    }

    if (target.usesDeclarationSpan) {
      return declarationSpan(source, focus.node);
    }

    final anchor = target.anchor!;
    if (isPointAnchor(anchor)) {
      throw StateError(
        '$mode step cannot use insertion anchor "${anchor.toString()}"',
      );
    }

    return interpreter.resolveSpan(
      source,
      AstPath(navigate: target.navigate, anchor: anchor),
    );
  }
}

/// Per-step idempotency detection for [AstPathPatchTransform].
class AstPathPatchIdempotency {
  const AstPathPatchIdempotency();

  bool shouldNoOp({
    required AstPathPatchMode mode,
    required String source,
    required AstPathPatchTarget target,
    required AnchorSpan span,
    required String replacement,
    required AstPathInterpreter interpreter,
  }) {
    if (mode == AstPathPatchMode.replace) {
      if (span.length == 0) return true;
      final current = source.substring(span.offset, span.offset + span.length);
      return _normalizeWhitespace(current) ==
          _normalizeWhitespace(replacement);
    }

    return false;
  }

  bool insertTargetExists(
    String source,
    AstPathPatchTarget target,
    AstPathInterpreter interpreter,
  ) {
    return _insertTargetExists(source, target, interpreter);
  }

  bool _insertTargetExists(
    String source,
    AstPathPatchTarget target,
    AstPathInterpreter interpreter,
  ) {
    if (target.navigate.isEmpty) return false;

    final last = target.navigate.last;
    if (last.kind == null || last.name == null) {
      return false;
    }

    if (last.kind == NavigateKind.classDecl ||
        last.kind == NavigateKind.root ||
        last.kind == NavigateKind.import) {
      return false;
    }

    try {
      interpreter.navigateTo(source, target.navigate);
      return true;
    } on AstPathResolutionException catch (error) {
      if (error.code == 'E_NODE_NOT_FOUND') {
        return false;
      }
      rethrow;
    }
  }

  static String _normalizeWhitespace(String value) {
    return value.trim().replaceAll(RegExp(r'\s+'), ' ');
  }
}
