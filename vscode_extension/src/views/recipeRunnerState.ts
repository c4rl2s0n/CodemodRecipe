import { RUNNER_TABS, RunnerTab } from '../constants';
import { FilePreview, RecipeSchema } from '../types';
import { RecipeViewState } from '../webview/webviewState';

export class RecipeRunnerState {
  recipes: readonly RecipeSchema[] = [];
  discoveryError: string | undefined;
  currentRecipe: RecipeSchema | undefined;
  initialArgs: Record<string, string> = {};
  lastArgs: Record<string, string> = {};
  lastFiles: FilePreview[] = [];
  activeTab: RunnerTab = RUNNER_TABS.recipes;

  setRecipes(recipes: readonly RecipeSchema[], discoveryError?: string): void {
    this.recipes = recipes;
    this.discoveryError = discoveryError;
  }

  selectRecipe(recipe: RecipeSchema, initialArgs: Record<string, string>): void {
    this.currentRecipe = recipe;
    this.initialArgs = initialArgs;
    this.lastArgs = {};
    this.lastFiles = [];
    this.activeTab = RUNNER_TABS.runner;
  }

  toWebviewState(): RecipeViewState {
    return {
      recipes: this.recipes,
      discoveryError: this.discoveryError,
      recipe: this.currentRecipe,
      initialArgs: this.initialArgs,
      activeTab: this.activeTab,
    };
  }
}
