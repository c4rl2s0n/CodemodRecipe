import 'context.dart';
import 'patch_helpers.dart';

/// Function signature for transform operations that convert source code to patches.
typedef TransformFunction = Future<List<SourcePatch>> Function(
  String source,
  CodemodContext context,
);

/// A deterministic source-to-patches operation.
abstract interface class CodeTransform {
  /// Produces patches for [source] using values from [context].
  Future<List<SourcePatch>> apply(String source, CodemodContext context);
}

/// Allows simple function-based transforms in recipes.
class FunctionTransform implements CodeTransform {
  final TransformFunction _apply;

  /// Creates a transform from a callback.
  const FunctionTransform(this._apply);

  /// Creates a transform from a callback function.
  static FunctionTransform fromCallback(TransformFunction apply) {
    return FunctionTransform(apply);
  }

  /// Delegates patch generation to the callback passed to the constructor.
  @override
  Future<List<SourcePatch>> apply(String source, CodemodContext context) {
    return _apply(source, context);
  }
}
