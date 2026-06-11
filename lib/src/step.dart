import 'operation.dart';
import 'post_execution.dart';
import 'recipe.dart';
import 'template.dart';

/// Types that may appear in [CodemodRecipe.compose] [steps].
mixin CodemodStep {
  List<CodemodArgDescriptor> get args => [];
  List<CodemodOperation> get operations => [];
  List<PostExecution> get postExecution => [];
  List<RecipeTemplatePreview> get previewTemplates => [];
}
