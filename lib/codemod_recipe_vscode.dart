/// Headless bridge between [CodemodRecipe]s and the VS Code extension.
///
/// Import this library from a small host entry point that registers your
/// project recipes and forwards process arguments to [CodemodHost]. The host
/// speaks a simple JSON-over-stdio protocol consumed by the extension.
library;

export 'codemod_recipe.dart';
export 'src/vscode/codemod_host.dart';
export 'src/vscode/diff_service.dart';
export 'src/vscode/patch_selector.dart';
export 'src/vscode/recipe_schema.dart';
