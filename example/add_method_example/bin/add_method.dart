import 'package:codemod_recipe/codemod_recipe.dart';

/// Example codemod that adds a method to an existing Dart class.
///
/// Usage:
///   dart run bin/add_method.dart --file=lib/counter.dart --class=Counter --method=decrement
///   dart run bin/add_method.dart --file=lib/counter.dart --class=Counter --method=decrement --apply
///
/// The first run shows a preview of changes. Add --apply to execute.
Future<void> main(List<String> args) {
  return CodemodRunner(addMethodRecipe).run(args);
}

final addMethodRecipe = CodemodRecipe(
  name: 'add_method',
  description: 'Adds a method to an existing Dart class',
  args: [
    CodemodArg.required('file', help: 'Path to the Dart file to modify'),
    CodemodArg.required(
      'class',
      help: 'Name of the class to add the method to',
    ),
    CodemodArg.required('method', help: 'Name of the method to add'),
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
      ],
    ),
  ],
  postExecution: const [DartFormatPostExecution()],
);
