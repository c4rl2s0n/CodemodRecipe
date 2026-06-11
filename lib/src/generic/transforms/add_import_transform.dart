import 'package:analyzer/dart/ast/ast.dart';

import '../../context.dart';
import '../../dart_codegen/ast_helpers/ast_helpers.dart';
import '../../patch_helpers.dart';
import '../../transform.dart';

/// Adds a Dart import directive unless it already exists.
class AddImportTransform implements CodeTransform {
  final String uri;

  /// Creates an import transform.
  const AddImportTransform({required this.uri});

  /// Creates an import transform from a URI resolver.
  const AddImportTransform.uri(this.uri);

  @override
  Future<List<SourcePatch>> apply(String source, CodemodContext context) async {
    final importUri = uri;
    final unit = parseSource(source);
    final imports = unit.directives.whereType<ImportDirective>().toList();

    if (imports.any((directive) => directive.uri.stringValue == importUri)) {
      return const [];
    }

    if (imports.isEmpty) {
      return [
        SourcePatch(
          0,
          0,
          "import '$importUri';\n\n",
          description: 'Add import $importUri',
        ),
      ];
    }

    return [
      SourcePatch(
        imports.last.end,
        0,
        "\nimport '$importUri';",
        description: 'Add import $importUri',
      ),
    ];
  }
}
