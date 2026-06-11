import 'package:codemod_recipe/codemod_recipe.dart';

/// Example codemod that scaffolds a new feature directory with model and view files.
///
/// Usage:
///   dart run bin/scaffold_feature.dart --feature=user_profile
///   dart run bin/scaffold_feature.dart --feature=user_profile --apply
///
/// Creates:
///   lib/features/user_profile/user_profile_model.dart
///   lib/features/user_profile/user_profile_view.dart
Future<void> main(List<String> args) {
  return CodemodRunner(scaffoldFeatureRecipe).run(args);
}

final scaffoldFeatureRecipe = CodemodRecipe(
  name: 'scaffold_feature',
  description: 'Scaffolds a new feature with model and view files',
  args: [
    CodemodArg.required(
      'feature',
      help: 'Name of the feature to scaffold (e.g., user_profile)',
    ),
  ],
  operations: [
    // Create the model file
    CreateFileOperation(
      path: (context) => context.render(
        'lib/features/{{feature:snake}}/{{feature:snake}}_model.dart',
      ),
      template: const CodemodTemplate.inline('''
/// Data model for {{feature:pascal}} feature.
class {{feature:pascal}}Model {
  const {{feature:pascal}}Model();

  // TODO: Add model properties
}
'''),
    ),
    // Create the view file
    CreateFileOperation(
      path: (context) => context.render(
        'lib/features/{{feature:snake}}/{{feature:snake}}_view.dart',
      ),
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
