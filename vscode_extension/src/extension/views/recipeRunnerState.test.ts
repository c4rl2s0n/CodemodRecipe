import { describe, expect, it } from 'vitest';
import type { RecipeSchema } from '../../shared';
import { RecipeRunnerState } from './recipeRunnerState';

const recipeA: RecipeSchema = {
  id: 'a',
  name: 'Recipe A',
  description: 'first',
  args: [],
};

const recipeAUpdated: RecipeSchema = {
  id: 'a',
  name: 'Recipe A v2',
  description: 'updated',
  args: [],
};

const recipeB: RecipeSchema = {
  id: 'b',
  name: 'Recipe B',
  description: 'other',
  args: [],
};

describe('RecipeRunnerState.syncRecipesAfterRefresh', () => {
  it('replaces currentRecipe with the fresh list entry without clearing selection', () => {
    const state = new RecipeRunnerState();
    state.currentRecipe = recipeA;
    state.lastArgs = { file: 'lib/a.dart' };
    state.lastFiles = [{ path: 'lib/a.dart', kind: 'patch', isNew: false, skipped: false, patches: [] }];

    state.syncRecipesAfterRefresh([recipeAUpdated, recipeB]);

    expect(state.currentRecipe).toEqual(recipeAUpdated);
    expect(state.lastArgs).toEqual({ file: 'lib/a.dart' });
    expect(state.lastFiles).toEqual([]);
  });

  it('keeps currentRecipe when id is missing from the refreshed list', () => {
    const state = new RecipeRunnerState();
    state.currentRecipe = recipeA;

    state.syncRecipesAfterRefresh([recipeB]);

    expect(state.currentRecipe).toEqual(recipeA);
  });

  it('includes diagnostics in webview state', () => {
    const state = new RecipeRunnerState();
    state.syncRecipesAfterRefresh([], undefined, [
      {
        severity: 'error',
        code: 'E_DUPLICATE_RECIPE_ID',
        message: 'Duplicate recipe id: foo',
        sources: [{ file: '.codemod/recipes/a.yaml', line: 1 }],
      },
    ]);

    expect(state.toWebviewState().diagnostics).toHaveLength(1);
    expect(state.toWebviewState().diagnostics[0]?.code).toBe('E_DUPLICATE_RECIPE_ID');
  });
});
