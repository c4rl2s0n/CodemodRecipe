import {
  RUNNER_TABS,
  type FilePreview,
  type RecipeSchema,
  type RecipeViewState,
  type RunnerTab,
} from '../../shared';

export class RecipeRunnerState {
  recipes: readonly RecipeSchema[] = [];
  discoveryError: string | undefined;
  recipesRefreshing = false;
  bootstrapInFlight = false;
  bootstrapPhase: 'startingHost' | 'loadingRecipes' | 'ready' | 'error' = 'startingHost';
  bootstrapError: string | undefined;
  currentRecipe: RecipeSchema | undefined;
  initialArgs: Record<string, string> = {};
  lastArgs: Record<string, string> = {};
  lastFiles: FilePreview[] = [];
  activeTab: RunnerTab = RUNNER_TABS.recipes;

  setRecipesRefreshing(inFlight: boolean): void {
    this.recipesRefreshing = inFlight;
  }

  setRecipes(recipes: readonly RecipeSchema[], discoveryError?: string): void {
    this.recipes = recipes;
    this.discoveryError = discoveryError;
  }

  syncRecipesAfterRefresh(
    recipes: readonly RecipeSchema[],
    discoveryError?: string
  ): void {
    this.recipes = recipes;
    this.discoveryError = discoveryError;
    if (!this.currentRecipe) {
      return;
    }
    const fresh = recipes.find((item) => item.id === this.currentRecipe!.id);
    if (fresh) {
      this.currentRecipe = fresh;
    }
    this.lastFiles = [];
  }

  setBootstrap(state: {
    inFlight: boolean;
    phase: 'startingHost' | 'loadingRecipes' | 'ready' | 'error';
    error?: string;
  }): void {
    this.bootstrapInFlight = state.inFlight;
    this.bootstrapPhase = state.phase;
    this.bootstrapError = state.error;
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
      recipesRefreshing: this.recipesRefreshing,
      bootstrapInFlight: this.bootstrapInFlight,
      bootstrapPhase: this.bootstrapPhase,
      bootstrapError: this.bootstrapError,
      recipe: this.currentRecipe,
      initialArgs: this.initialArgs,
      activeTab: this.activeTab,
      autoPreviewDebounceMs: 400,
    };
  }
}
