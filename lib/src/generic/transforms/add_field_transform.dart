import '../../context.dart';
import '../../dart_codegen/ast_helpers/ast_helpers.dart';
import '../../dart_codegen/code_editor.dart';
import '../../dart_codegen/field_spec.dart';
import '../../patch_helpers.dart';
import '../../transform.dart';
import 'resolvers.dart';

/// Adds a field to a class unless that field already exists.
class AddFieldTransform implements CodeTransform {
  final StringResolver className;
  final StringResolver fieldName;
  final StringResolver fieldType;
  final StringResolver? defaultValue;
  final bool isNullable;
  final bool isFinal;
  final bool isConst;
  final bool isStatic;
  final FieldConstructorArgs? constructorArgs;

  /// Creates a field transform.
  const AddFieldTransform({
    required this.className,
    required this.fieldName,
    required this.fieldType,
    this.defaultValue,
    this.isNullable = false,
    this.isFinal = true,
    this.isConst = false,
    this.isStatic = false,
    this.constructorArgs,
  });

  @override
  Future<List<SourcePatch>> apply(String source, CodemodContext context) async {
    final focus = AstFocus.parse(source).classNamed(className(context));
    return CodeEditor(source, preferences: context.preferences)
        .addFieldUnlessExists(
          focus,
          fieldName(context),
          fieldType(context),
          isNullable: isNullable,
          defaultValue: defaultValue?.call(context),
          isFinal: isFinal,
          isConst: isConst,
          isStatic: isStatic,
          constructorArgs: constructorArgs,
        )
        .patches;
  }
}
