import * as vscode from 'vscode';
import type { RecipeRepository } from '../recipes/recipeRepository';
import type { InvokeMode } from '../recipes/recipeInvoke';

export async function pickRecipeId(
  repository: RecipeRepository
): Promise<string | undefined> {
  const picked = await vscode.window.showQuickPick(
    repository.getRecipes().map((item) => ({
      label: item.id,
      description: item.name,
      detail: item.description,
    })),
    { placeHolder: 'Select a recipe' }
  );
  return picked?.label;
}

export async function pickInvokeMode(
  defaultMode: InvokeMode
): Promise<InvokeMode | undefined> {
  const picked = await vscode.window.showQuickPick(
    [
      {
        label: 'auto',
        description: 'Execute when complete; otherwise open runner',
      },
      { label: 'run', description: 'Prefer execute; open runner if incomplete' },
      { label: 'open', description: 'Always open recipe runner prefilled' },
    ],
    {
      placeHolder: `Invoke mode (default: ${defaultMode})`,
    }
  );
  return (picked?.label as InvokeMode | undefined) ?? undefined;
}
