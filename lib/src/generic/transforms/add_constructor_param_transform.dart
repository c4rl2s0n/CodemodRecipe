import '../../ast_path/ast_path.dart';
import '../../ast_path/class_focus.dart';
import '../../context.dart';
import '../../dart_codegen/code_editor.dart';
import '../../dart_codegen/field_spec.dart';
import '../../patch_helpers.dart';
import '../../transform.dart';
import 'resolvers.dart';

/// Adds a constructor parameter to a class unless it already exists.
class AddConstructorParamTransform implements CodeTransform {
  final List<NavigateStep>? navigate;
  final StringResolver? className;
  final StringResolver paramName;
  final StringResolver paramType;
  final StringResolver? defaultValue;
  final bool isNullable;
  final bool thisPrefix;
  final FieldConstructorArgs? constructorArgs;

  /// Creates a constructor parameter transform.
  const AddConstructorParamTransform({
    this.navigate,
    this.className,
    required this.paramName,
    required this.paramType,
    this.defaultValue,
    this.isNullable = false,
    this.thisPrefix = true,
    this.constructorArgs,
  }) : assert(navigate != null || className != null);

  @override
  Future<List<SourcePatch>> apply(String source, CodemodContext context) async {
    final focus = resolveClassFocus(
      source,
      context,
      navigate: navigate,
      className: className,
    );

    return CodeEditor(source, preferences: context.preferences)
        .addConstructorParamUnlessExists(
          focus,
          paramName(context),
          paramType(context),
          isNullable: isNullable,
          defaultValue: defaultValue?.call(context),
          thisPrefix: thisPrefix,
          constructorArgs: constructorArgs,
        )
        .patches;
  }
}
