import '../../context.dart';
import '../../dart_codegen/ast_helpers/ast_helpers.dart';
import '../../dart_codegen/code_editor.dart';
import '../../dart_codegen/field_spec.dart';
import '../../patch_helpers.dart';
import '../../transform.dart';

/// Adds a constructor parameter to a class unless it already exists.
class AddConstructorParamTransform implements CodeTransform {
  final String className;
  final String paramName;
  final String paramType;
  final String? defaultValue;
  final bool isNullable;
  final bool thisPrefix;
  final FieldConstructorArgs? constructorArgs;

  /// Creates a constructor parameter transform.
  const AddConstructorParamTransform({
    required this.className,
    required this.paramName,
    required this.paramType,
    this.defaultValue,
    this.isNullable = false,
    this.thisPrefix = true,
    this.constructorArgs,
  });

  @override
  Future<List<SourcePatch>> apply(String source, CodemodContext context) async {
    final focus = AstFocus.parse(source).classNamed(className);
    return CodeEditor(source, preferences: context.preferences)
        .addConstructorParamUnlessExists(
          focus,
          paramName,
          paramType,
          isNullable: isNullable,
          defaultValue: defaultValue,
          thisPrefix: thisPrefix,
          constructorArgs: constructorArgs,
        )
        .patches;
  }
}
