import '../../context.dart';
import '../../dart_codegen/ast_helpers/ast_helpers.dart';
import '../../dart_codegen/code_editor.dart';
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
    final focus = AstFocus.parse(source).classNamed(className(context));
    return CodeEditor(source)
        .addMethodUnlessExists(focus, methodName(context), body.render(context))
        .patches;
  }
}
