import '../../ast_path/ast_path.dart';
import '../../ast_path/class_focus.dart';
import '../../context.dart';
import '../../patch_helpers.dart';
import '../../transform.dart';
import 'resolvers.dart';

/// Adds an annotation above a class unless it already exists.
class AddClassAnnotationTransform implements CodeTransform {
  final List<NavigateStep>? navigate;
  final StringResolver? className;
  final StringResolver annotation;

  /// Creates a class annotation transform.
  const AddClassAnnotationTransform({
    this.navigate,
    this.className,
    required this.annotation,
  }) : assert(navigate != null || className != null);

  @override
  Future<List<SourcePatch>> apply(String source, CodemodContext context) async {
    final focus = resolveClassFocus(
      source,
      context,
      navigate: navigate,
      className: className,
    );
    final classNode = focus.asClass;

    final annotationCode = annotation(context);
    final annotationName = annotationCode
        .replaceFirst('@', '')
        .split(RegExp(r'[\s(]'))
        .first;

    final exists = classNode.metadata.any((metadata) {
      return metadata.name.name == annotationName;
    });
    if (exists) return const [];

    return [
      SourcePatch(
        classNode.offset,
        0,
        '$annotationCode\n',
        description: 'Add annotation $annotationCode',
      ),
    ];
  }
}
