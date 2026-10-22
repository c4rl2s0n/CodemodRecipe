import type { ArgValues } from '../../../shared';
import { createShortcutForRecipe } from '../../recipes/createShortcut';
import { invokeRecipe } from '../../recipes/recipeInvoke';
import type { RecipeRunnerHandlerHost } from './types';

export async function handleInvokeRecipe(
  host: RecipeRunnerHandlerHost,
  recipeId: string,
  mode: 'auto' | 'run' | 'open',
  args?: ArgValues
): Promise<void> {
  const repository = host.recipeRepository;
  if (!repository) {
    return;
  }
  await invokeRecipe(
    {
      repository,
      bridge: host.bridge,
      config: host.config,
      runner: host,
    },
    { recipeId, mode, args }
  );
}

export async function handleCreateShortcut(
  host: RecipeRunnerHandlerHost,
  recipeId: string,
  args?: ArgValues
): Promise<void> {
  const repository = host.recipeRepository;
  if (!repository) {
    return;
  }
  const scalar: Record<string, string> = {};
  if (args) {
    for (const [k, v] of Object.entries(args)) {
      if (typeof v === 'string') {
        scalar[k] = v;
      }
    }
  }
  await createShortcutForRecipe(
    { config: host.config, repository },
    recipeId,
    Object.keys(scalar).length > 0 ? scalar : undefined
  );
}

export async function handleSelectRecipe(
  host: RecipeRunnerHandlerHost,
  recipeId: string
): Promise<void> {
  const recipe = host.state.recipes.find((item) => item.id === recipeId);
  if (recipe) {
    host.run(recipe);
  }
}

