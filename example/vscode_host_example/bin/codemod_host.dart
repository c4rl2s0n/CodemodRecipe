import 'package:codemod_recipe/codemod_recipe_vscode.dart';

/// Host entry point that exposes example recipes to the VS Code extension.
///
/// The VS Code extension launches this program and exchanges JSON over stdio.
/// Register every recipe you want to surface in the extension by a stable id.
Future<void> main(List<String> args) {
  return CodemodHost.fromList([
    addMethodRecipe,
    scaffoldFeatureRecipe,
  ]).run(args);
}

final addMethodRecipe = CodemodRecipe(
  name: 'add_method',
  description: 'Adds a method to an existing Dart class',
  args: [
    CodemodArg.required(
      'file',
      help: 'Path to the Dart file to modify',
      inputKind: CodemodArgInputKind.file,
      contextKey: CodemodContextKey.file,
    ),
    CodemodArg.required(
      'class',
      help: 'Name of the class to add the method to',
      inputKind: CodemodArgInputKind.symbol,
      contextKey: CodemodContextKey.dartClass,
    ),
    CodemodArg.required(
      'method',
      help: 'Name of the method to add',
      inputKind: CodemodArgInputKind.symbol,
      contextKey: CodemodContextKey.word,
    ),
  ],
  operations: [
    EditDartFileOperation(
      path: (context) => context.require('file'),
      transforms: (context) => [
        AddMethodTransform(
          className: (context) => context.require('class'),
          methodName: (context) => context.camel('method'),
          body: const CodemodTemplate.inline('''
  void {{method:camel}}() {
    // TODO: Implement {{method:camel}}
  }
'''),
        ),
        AddPropertyTransform(
          className: (context) => context.require('class'),
          propertyName: (context) => context.camel('method'),
          body: const CodemodTemplate.inline('''
  void {{method:snake}}() {
    // TODO: Implement {{method:snake}}
  }
'''),
        ),
      ],
    ),
  ],
  postExecution: const [DartFormatPostExecution()],
);

final scaffoldFeatureRecipe = CodemodRecipe(
  name: 'scaffold_feature',
  description: 'Scaffolds a new feature with model and view files',
  args: [
    CodemodArg.required(
      'feature',
      help: 'Name of the feature to scaffold (e.g., user_profile)',
      inputKind: CodemodArgInputKind.symbol,
      contextKey: CodemodContextKey.word,
    ),
  ],
  operations: [
    CreateFileOperation.templatePath(
      pathTemplate:
          'lib/features/{{feature:snake}}/{{feature:snake}}_model.dart',
      previewLabel: 'Model file',
      template: const CodemodTemplate.inline('''
/// Data model for {{feature:pascal}} feature.
class {{feature:pascal}}Model {
  const {{feature:pascal}}Model();

  // TODO: Add model properties
}
'''),
    ),
    CreateFileOperation.templatePath(
      pathTemplate:
          'lib/features/{{feature:snake}}/{{feature:snake}}_view.dart',
      previewLabel: 'View file',
      template: const CodemodTemplate.inline('''
import 'package:flutter/material.dart';

import '{{feature:snake}}_model.dart';

/// View widget for {{feature:pascal}} feature.
class {{feature:pascal}}View extends StatelessWidget {
  const {{feature:pascal}}View({super.key});

  @override
  Widget build(BuildContext context) {
    return const Placeholder();
  }
}
'''),
    ),
  ],
  postExecution: const [DartFormatPostExecution()],
);
