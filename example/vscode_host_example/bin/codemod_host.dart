import 'package:codemod_recipe/codemod_recipe_vscode.dart';

/// Host entry point that exposes example recipes to the VS Code extension.
///
/// The VS Code extension launches this program and exchanges JSON over stdio.
/// Register every recipe you want to surface in the extension by a stable id.
Future<void> main(List<String> args) {
  return CodemodHost.fromList([
    addMethodRecipe,
    scaffoldFeatureRecipe,
    addPropertyAccessorsRecipe,
    scaffoldAndWireServiceRecipe,
  ]).run(args);
}

final addMethodRecipe = CodemodRecipe(
  name: 'add_method',
  description: 'Adds a method to an existing Dart class',
  args: [
    CodemodArg<String>.required(
      'file',
      help: 'Path to the Dart file to modify',
      inputKind: CodemodArgInputKind.file,
      contextKey: CodemodContextKey.file,
    ),
    CodemodArg<String>.required(
      'class',
      help: 'Name of the class to add the method to',
      inputKind: CodemodArgInputKind.symbol,
      contextKey: CodemodContextKey.dartClass,
    ),
    CodemodArg<String>.required(
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
        AddFieldTransform(
          className: (context) => context.require('class'),
          fieldName: (context) => context.camel('method'),
          fieldType: (_) => 'int',
          defaultValue: (_) => '0',
        ),
      ],
    ),
  ],
  postExecution: [DartFormatPostExecution()],
);

final scaffoldFeatureRecipe = CodemodRecipe(
  name: 'scaffold_feature',
  description: 'Scaffolds a new feature with model and view files',
  args: [
    CodemodArg<String>.required(
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

final addPropertyAccessorsRecipe = CodemodRecipe(
  name: 'add_property_accessors',
  description: 'Adds a private field with getter and setter methods',
  args: [
    CodemodArg<String>.required(
      'file',
      help: 'Path to the Dart file to modify',
      inputKind: CodemodArgInputKind.file,
      contextKey: CodemodContextKey.file,
    ),
    CodemodArg<String>.required(
      'class',
      help: 'Name of the class to update',
      inputKind: CodemodArgInputKind.symbol,
      contextKey: CodemodContextKey.dartClass,
    ),
    CodemodArg<String>.required(
      'property',
      help: 'Property name (for example: score)',
      inputKind: CodemodArgInputKind.symbol,
      contextKey: CodemodContextKey.word,
    ),
    CodemodArg<String>.optional(
      'type',
      help: 'Property type',
      defaultsTo: 'int',
      inputKind: CodemodArgInputKind.dartType,
    ),
  ],
  operations: [
    EditDartFileOperation(
      path: (context) => context.require('file'),
      transforms: (context) => [
        AddFieldTransform(
          className: (c) => c.require('class'),
          fieldName: (c) => '_${c.camel('property')}',
          fieldType: (c) => c.require('type'),
        ),
        AddMethodTransform(
          className: (c) => c.require('class'),
          methodName: (c) => 'get${c.pascal('property')}',
          body: const CodemodTemplate.inline(
            '  {{type}} get{{property:pascal}}() => _{{property:camel}};\n',
          ),
        ),
        AddMethodTransform(
          className: (c) => c.require('class'),
          methodName: (c) => 'set${c.pascal('property')}',
          body: const CodemodTemplate.inline(
            '  void set{{property:pascal}}({{type}} value) => _{{property:camel}} = value;\n',
          ),
        ),
      ],
    ),
  ],
  postExecution: const [DartFormatPostExecution()],
);

final scaffoldAndWireServiceRecipe = CodemodRecipe(
  name: 'scaffold_and_wire_service',
  description: 'Creates a service file and wires it into an existing class',
  args: [
    CodemodArg<String>.required(
      'service',
      help: 'Service name (for example: counter_sync)',
      inputKind: CodemodArgInputKind.symbol,
      contextKey: CodemodContextKey.word,
    ),
    CodemodArg<String>.optional(
      'file',
      help: 'Target Dart file to wire service into',
      defaultsTo: 'lib/counter.dart',
      inputKind: CodemodArgInputKind.file,
      contextKey: CodemodContextKey.file,
    ),
    CodemodArg<String>.optional(
      'class',
      help: 'Target class in the file',
      defaultsTo: 'Counter',
      inputKind: CodemodArgInputKind.symbol,
      contextKey: CodemodContextKey.dartClass,
    ),
  ],
  operations: [
    CreateFileOperation.templatePath(
      pathTemplate: 'lib/services/{{service:snake}}_service.dart',
      previewLabel: 'Service file',
      template: const CodemodTemplate.inline('''
class {{service:pascal}}Service {
  const {{service:pascal}}Service();
}
'''),
    ),
    EditDartFileOperation(
      path: (context) => context.require('file'),
      transforms: (context) => [
        AddImportTransform.uri(
          (c) => 'services/${c.snake('service')}_service.dart',
        ),
        AddFieldTransform(
          className: (c) => c.require('class'),
          fieldName: (c) => '${c.camel('service')}Service',
          fieldType: (c) => '${c.pascal('service')}Service',
          constructorArgs: const FieldConstructorArgs(),
        ),
      ],
    ),
  ],
  postExecution: const [DartFormatPostExecution()],
);
