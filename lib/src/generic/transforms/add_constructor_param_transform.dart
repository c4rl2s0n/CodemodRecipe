import '../../context.dart';
import '../../dart/code_editor.dart';
import '../../dart/field_spec.dart';
import '../../patch_helpers.dart';
import '../../transform.dart';
import 'resolvers.dart';

/// Adds a constructor parameter to a class unless it already exists.
class AddConstructorParamTransform implements CodeTransform {
  final StringResolver className;
  final StringResolver paramName;
  final StringResolver paramType;
  final StringResolver? defaultValue;
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
    return CodeEditor(source, preferences: context.preferences)
        .inClass(className(context))
        .addConstructorParamUnlessExists(
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
