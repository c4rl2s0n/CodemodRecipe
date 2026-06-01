import '../../context.dart';
import '../../dart/code_editor.dart';
import '../../patch_helpers.dart';
import '../../template.dart';
import '../../transform.dart';
import 'resolvers.dart';

/// Adds a rendered method to a class unless that method already exists.
class AddMethodTransform implements CodeTransform {
  final StringResolver className;
  final StringResolver methodName;
  final CodemodTemplate body;

  /// Creates a method transform.
  const AddMethodTransform({
    required this.className,
    required this.methodName,
    required this.body,
  });

  @override
  Future<List<SourcePatch>> apply(String source, CodemodContext context) async {
    return CodeEditor(source)
        .inClass(className(context))
        .addMethodUnlessExists(methodName(context), body.render(context))
        .patches;
  }
}
