import 'package:codemod_recipe/codemod_recipe.dart';

/// Example showing how to compose multiple recipes using CodemodRecipe.compose.
///
/// This example combines two recipes that share arguments:
/// - add_model_import: Adds an import statement for the model
/// - add_model_property: Adds a property to the model class
///
/// Usage:
///   dart run bin/composed_codemod.dart --file=lib/models.dart --model=User --property=email
///   dart run bin/composed_codemod.dart --file=lib/models.dart --model=User --property=email --apply
Future<void> main(List<String> args) {
  return CodemodRunner(composedRecipe).run(args);
}

// Shared argument definitions
const fileArg = CodemodArg.required('file', help: 'Path to the Dart file');
const modelArg = CodemodArg.required('model', help: 'Name of the model class');
const propertyArg = CodemodArg.required(
  'property',
  help: 'Name of the property to add',
);

// First recipe: Add import for the model file
final addModelImportRecipe = CodemodRecipe(
  name: 'add_model_import',
  args: [fileArg, modelArg],
  operations: [
    EditDartFileOperation(
      path: (context) => context.require('file'),
      transforms: (context) => [
        AddImportTransform.uri(
          (ctx) => 'package:my_app/models/${ctx.snake('model')}.dart',
        ),
      ],
    ),
  ],
);

// Second recipe: Add property to the model class
final addModelPropertyRecipe = CodemodRecipe(
  name: 'add_model_property',
  args: [fileArg, modelArg, propertyArg],
  operations: [
    EditDartFileOperation(
      path: (context) => context.require('file'),
      transforms: (context) => [
        AddFieldTransform(
          className: (ctx) => ctx.require('model'),
          fieldName: (ctx) => ctx.camel('property'),
          fieldType: (_) => 'String',
          addToConstructor: true,
        ),
      ],
    ),
  ],
);

// Composed recipe that runs both operations
final composedRecipe = CodemodRecipe.compose(
  name: 'composed_codemod',
  description: 'Composes add_model_import and add_model_property recipes',
  steps: [
    addModelImportRecipe,
    addModelPropertyRecipe,
    const DartFormatPostExecution(),
  ],
);
