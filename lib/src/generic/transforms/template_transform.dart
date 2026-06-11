import '../../context.dart';
import '../../patch_helpers.dart';
import '../../template.dart';
import '../../transform.dart';
import 'resolvers.dart';

/// Inserts a rendered snippet at a caller-provided offset.
class TemplateTransform implements CodeTransform {
  final CodemodTemplate template;
  final OffsetResolver offset;
  final String? description;

  /// Creates a rendered snippet transform.
  const TemplateTransform({
    required this.template,
    required this.offset,
    this.description,
  });

  @override
  Future<List<SourcePatch>> apply(String source, CodemodContext context) async {
    return [
      SourcePatch(
        offset(source, context),
        0,
        template.render(context),
        description: description ?? 'Insert rendered template',
      ),
    ];
  }
}
