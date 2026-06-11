import 'context.dart';
import 'patch_helpers.dart';

/// A deterministic source-to-patches operation.
abstract interface class CodeTransform {
  /// Produces patches for [source] using values from [context].
  Future<List<SourcePatch>> apply(String source, CodemodContext context);
}

/// Allows simple function-based transforms in recipes.
class FunctionTransform implements CodeTransform {
  final Future<List<SourcePatch>> Function(
    String source,
    CodemodContext context,
  )
  _apply;

  /// Creates a transform from a callback.
  const FunctionTransform(this._apply);

  /// Delegates patch generation to the callback passed to the constructor.
  @override
  Future<List<SourcePatch>> apply(String source, CodemodContext context) {
    return _apply(source, context);
  }
}
