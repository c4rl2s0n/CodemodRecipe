import '../ast_path/ast_path.dart';
import '../core/context.dart';
import '../core/patch_helpers.dart';
import '../core/template.dart';
import '../core/transform.dart';
import 'patch_transform.dart';

/// Replaces a span at an AST path with rendered template text.
class AstPathReplaceTransform implements CodeTransform {
  const AstPathReplaceTransform({
    required this.target,
    required this.template,
    this.description,
  });

  final AstPathPatchTarget target;
  final CodemodTemplate template;
  final String? description;

  @override
  Future<List<SourcePatch>> apply(String source, CodemodContext context) async {
    return AstPathPatchTransform.replace(
      target: target,
      text: template.render(context),
      description: description ?? 'Replace ${target.navigate}',
    ).apply(source, context);
  }
}

/// Inserts rendered template text at an [AstPath] location.
class AstPathInsertTransform implements CodeTransform {
  const AstPathInsertTransform({
    required this.path,
    required this.template,
    this.description,
  });

  final AstPath path;
  final CodemodTemplate template;
  final String? description;

  @override
  Future<List<SourcePatch>> apply(String source, CodemodContext context) async {
    return AstPathPatchTransform.insert(
      path: path,
      text: template.render(context),
      description: description ?? 'Insert at $path',
    ).apply(source, context);
  }
}
