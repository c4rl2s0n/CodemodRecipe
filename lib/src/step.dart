/// Types that may appear in [CodemodRecipe.compose] [steps].
abstract class CodemodStep {
    List<CodemodArg> get args => [];
    List<CodemodOperation> get operations => [];
    List<PostExecution> get postExecution => [];
    List<RecipeTemplatePreview> get previewTemplates => [];
}
