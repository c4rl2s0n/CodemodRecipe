import '../../context.dart';
import '../../dart/ast_helpers.dart';
import '../../patch_helpers.dart';
import '../../transform.dart';
import 'resolvers.dart';

/// Adds an annotation above a class unless it already exists.
class AddClassAnnotationTransform implements CodeTransform {
  final StringResolver className;
  final StringResolver annotation;

  /// Creates a class annotation transform.
  const AddClassAnnotationTransform({
    required this.className,
    required this.annotation,
  });

  @override
  Future<List<SourcePatch>> apply(String source, CodemodContext context) async {
    final targetClassName = className(context);
    final unit = parseSource(source);
    final classNode = findClassByName(unit, targetClassName);
    if (classNode == null) {
      throw StateError('Class "$targetClassName" not found in source');
    }

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
