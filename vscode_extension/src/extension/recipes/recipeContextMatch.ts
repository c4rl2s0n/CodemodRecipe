import {
  missingRequiredArgNames,
  prefillArgs,
} from './recipeContextValues';
import type { RecipeSchema, ContextRecipeMatch } from '../../shared';

export type { ContextRecipeMatch } from '../../shared';

/**
 * Tier A: recipes where local `from` / `contextKey` prefills at least one arg.
 * Same rule as Run From Cursor Context QuickPick.
 */
export function matchRecipesToEditorContext(
  recipes: readonly RecipeSchema[],
  contextValues: Record<string, string>
): ContextRecipeMatch[] {
  const matches: ContextRecipeMatch[] = [];
  for (const recipe of recipes) {
    const args = prefillArgs(recipe, contextValues);
    if (Object.keys(args).length === 0) {
      continue;
    }
    matches.push({
      recipeId: recipe.id,
      name: recipe.name,
      description: recipe.description,
      args,
      complete: missingRequiredArgNames(recipe, args).length === 0,
    });
  }
  return matches;
}
