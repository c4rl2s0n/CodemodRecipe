import {
  RUNNER_SESSION_PHASE,
  RUNNER_TABS,
  type ArgValues,
  type ContextRecipeMatch,
  type FilePreview,
  type RecipeDiagnostic,
  type RecipeSchema,
  type RecipeViewState,
  type RunnerSessionSnapshot,
  type RunnerTab,
} from '../../shared';

export class RecipeRunnerState {
  recipes: readonly RecipeSchema[] = [];
  discoveryError: string | undefined;
  diagnostics: readonly RecipeDiagnostic[] = [];
  recipesRefreshing = false;
  bootstrapInFlight = true;
  bootstrapPhase: 'startingHost' | 'loadingRecipes' | 'ready' | 'error' = 'startingHost';
  bootstrapError: string | undefined;
  currentRecipe: RecipeSchema | undefined;
  initialArgs: ArgValues = {};
  lastArgs: ArgValues = {};
  lastPreviewToken: string | undefined;
  lastFiles: FilePreview[] = [];
  lastPreviewArgsKey: string | undefined;
  activeTab: RunnerTab = RUNNER_TABS.recipes;
  contextMatches: readonly ContextRecipeMatch[] = [];
  slotsByRecipe: Record<string, string[]> = {};
  sessionId = 1;
  sessionPhase: RunnerSessionSnapshot['phase'] = RUNNER_SESSION_PHASE.idle;
  activeRequestId: number | undefined;

  setRecipesRefreshing(inFlight: boolean): void {
    this.recipesRefreshing = inFlight;
  }

  setRecipes(
    recipes: readonly RecipeSchema[],
    discoveryError?: string,
    diagnostics: readonly RecipeDiagnostic[] = []
  ): void {
    this.recipes = recipes;
    this.discoveryError = discoveryError;
    this.diagnostics = diagnostics;
  }

  setContextMatching(
    matches: readonly ContextRecipeMatch[],
    slotsByRecipe: Record<string, string[]>
  ): void {
    this.contextMatches = matches;
    this.slotsByRecipe = slotsByRecipe;
  }

  syncRecipesAfterRefresh(
    recipes: readonly RecipeSchema[],
    discoveryError?: string,
    diagnostics: readonly RecipeDiagnostic[] = []
  ): void {
    this.recipes = recipes;
    this.discoveryError = discoveryError;
    this.diagnostics = diagnostics;
    this.lastFiles = [];
    this.lastPreviewToken = undefined;
    this.lastPreviewArgsKey = undefined;
    if (this.sessionPhase === RUNNER_SESSION_PHASE.reviewing) {
      this.sessionPhase = RUNNER_SESSION_PHASE.idle;
    }
    if (!this.currentRecipe) {
      return;
    }
    const fresh = recipes.find((item) => item.id === this.currentRecipe!.id);
    if (fresh) {
      this.currentRecipe = fresh;
    }
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

  selectRecipe(recipe: RecipeSchema, initialArgs: ArgValues): void {
    this.currentRecipe = recipe;
    this.initialArgs = initialArgs;
    this.lastArgs = {};
    this.lastPreviewToken = undefined;
    this.lastFiles = [];
    this.lastPreviewArgsKey = undefined;
    this.activeTab = RUNNER_TABS.runner;
    this.sessionPhase = RUNNER_SESSION_PHASE.idle;
    this.activeRequestId = undefined;
  }

  /** Bump session generation (webview ready / recreate). Clears in-flight ops. */
  beginSession(): number {
    this.sessionId += 1;
    this.sessionPhase = RUNNER_SESSION_PHASE.idle;
    this.activeRequestId = undefined;
    return this.sessionId;
  }

  setSessionPhase(
    phase: RunnerSessionSnapshot['phase'],
    requestId?: number
  ): void {
    this.sessionPhase = phase;
    this.activeRequestId = requestId;
  }

  sessionSnapshot(): RunnerSessionSnapshot {
    const previewStatus =
      this.sessionPhase === RUNNER_SESSION_PHASE.previewing
        ? 'Previewing…'
        : this.sessionPhase === RUNNER_SESSION_PHASE.pickingPath
          ? 'Selecting path…'
          : this.sessionPhase === RUNNER_SESSION_PHASE.applying
            ? 'Applying…'
            : undefined;
    return {
      id: this.sessionId,
      phase: this.sessionPhase,
      activeRequestId: this.activeRequestId,
      previewStatus,
      lastPreviewArgsKey: this.lastPreviewArgsKey,
    };
  }

  toWebviewState(): RecipeViewState {
    return {
      recipes: this.recipes,
      discoveryError: this.discoveryError,
      diagnostics: this.diagnostics,
      recipesRefreshing: this.recipesRefreshing,
      bootstrapInFlight: this.bootstrapInFlight,
      bootstrapPhase: this.bootstrapPhase,
      bootstrapError: this.bootstrapError,
      recipe: this.currentRecipe,
      initialArgs: this.initialArgs,
      activeTab: this.activeTab,
      contextMatches: this.contextMatches,
      slotsByRecipe: this.slotsByRecipe,
      session: this.sessionSnapshot(),
    };
  }
}
