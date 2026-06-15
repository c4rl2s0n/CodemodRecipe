import '../../ast_path/ast_path.dart';
import '../../ast_path/class_focus.dart';
import '../../context.dart';
import '../../dart_codegen/code_editor.dart';
import '../../patch_helpers.dart';
import '../../template.dart';
import '../../transform.dart';
import 'resolvers.dart';

/// Adds a rendered method to a class unless that method already exists.
class AddMethodTransform implements CodeTransform {
  final List<NavigateStep>? navigate;
  final StringResolver? className;
  final StringResolver methodName;
  final CodemodTemplate body;

  /// Creates a method transform.
  const AddMethodTransform({
    this.navigate,
    this.className,
    required this.methodName,
    required this.body,
  }) : assert(navigate != null || className != null);

  @override
  Future<List<SourcePatch>> apply(String source, CodemodContext context) async {
    final focus = resolveClassFocus(
      source,
      context,
      navigate: navigate,
      className: className,
    );

    return CodeEditor(source)
        .addMethodUnlessExists(focus, methodName(context), body.render(context))
        .patches;
  }
}
