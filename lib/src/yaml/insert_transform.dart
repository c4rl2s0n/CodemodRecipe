import '../ast_path/ast_path.dart';
import '../context.dart';
import '../patch_helpers.dart';
import '../template.dart';
import '../transform.dart';

/// Inserts rendered template text at an [AstPath] location.
class AstPathInsertTransform implements CodeTransform {
  /// Creates an AST-path insert transform.
  const AstPathInsertTransform({
    required this.path,
    required this.template,
    this.description,
  });

  /// Navigate + anchor path resolved against the target file.
  final AstPath path;

  /// Text to render and insert.
  final CodemodTemplate template;

  /// Optional patch description.
  final String? description;

  @override
  Future<List<SourcePatch>> apply(String source, CodemodContext context) async {
    final interpreter = AstPathInterpreter();
    final offset = interpreter.resolveOffset(source, path);
    return [
      SourcePatch(
        offset,
        0,
        template.render(context),
        description: description ?? 'Insert at $path',
      ),
    ];
  }
}
