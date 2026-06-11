import '../../context.dart';
import '../../dart/code_editor.dart';
import '../../patch_helpers.dart';
import '../../transform.dart';
import 'resolvers.dart';

/// Adds a field to a class unless that field already exists.
class AddFieldTransform implements CodeTransform {
  final StringResolver className;
  final StringResolver fieldName;
  final StringResolver fieldType;
  final StringResolver? defaultValue;
  final bool addToConstructor;

  /// Creates a field transform.
  const AddFieldTransform({
    required this.className,
    required this.fieldName,
    required this.fieldType,
    this.defaultValue,
    this.addToConstructor = true,
  });

  @override
  Future<List<SourcePatch>> apply(String source, CodemodContext context) async {
    return CodeEditor(source)
        .inClass(className(context))
        .addFieldUnlessExists(
          fieldName(context),
          fieldType(context),
          defaultValue: defaultValue?.call(context),
          addToConstructor: addToConstructor,
        )
        .patches;
  }
}
