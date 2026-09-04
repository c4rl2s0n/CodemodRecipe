import {
  RUNNER_SESSION_PHASE,
  RUNNER_TABS,
  type ArgValues,
  type ComponentDetailSchema,
  type ComponentFinding,
  type ComponentInstance,
  type ComponentSchema,
  type ContextRecipeMatch,
  type FilePreview,
  type RecipeDiagnostic,
  type RecipeSchema,
  type RecipeViewState,
  type RunnerSessionSnapshot,
  type RunnerTab,
  type StackSchema,
  type TranscriptEvent,
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

  components: readonly ComponentSchema[] = [];
  stack: StackSchema | undefined;
  componentInstances: readonly ComponentInstance[] = [];
  componentFindings: readonly ComponentFinding[] = [];
  selectedComponentId: string | undefined;
  selectedInstanceId: string | undefined;
  componentDetail: ComponentDetailSchema | undefined;
  componentAction: string | undefined;
  componentArgs: ArgValues = {};
  componentFiles: FilePreview[] = [];
  componentTranscript: TranscriptEvent[] = [];
  componentScanError: string | undefined;
  componentOpError: string | undefined;
  componentDryRunResult: unknown | undefined;
  componentDryRunHook: string | undefined;

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

  setComponentCatalog(
    components: readonly ComponentSchema[],
    stack?: StackSchema
  ): void {
    this.components = components;
    this.stack = stack;
  }

  setComponentScan(
    instances: readonly ComponentInstance[],
    findings: readonly ComponentFinding[] = [],
    error?: string
  ): void {
    this.componentInstances = instances;
    this.componentFindings = findings;
    this.componentScanError = error;
    if (
      this.selectedInstanceId &&
      !instances.some((i) => i.id === this.selectedInstanceId)
    ) {
      this.selectedInstanceId = undefined;
    }
  }

  selectComponent(componentId: string, detail?: ComponentDetailSchema): void {
    this.selectedComponentId = componentId;
    this.selectedInstanceId = undefined;
    this.componentDetail = detail;
    this.componentAction = detail?.hasScaffold
      ? 'scaffold'
      : detail?.operations[0]?.name;
    this.componentArgs = {};
    this.componentFiles = [];
    this.componentTranscript = [];
    this.componentOpError = undefined;
    this.componentDryRunResult = undefined;
    this.componentDryRunHook = undefined;
    this.activeTab = RUNNER_TABS.components;
  }

  selectComponentInstance(instanceId: string): void {
    this.selectedInstanceId = instanceId;
    const instance = this.componentInstances.find((i) => i.id === instanceId);
    if (instance && instance.componentId !== this.selectedComponentId) {
      this.selectedComponentId = instance.componentId;
    }
    const ops = this.componentDetail?.operations ?? [];
    if (ops.length > 0) {
      this.componentAction = ops[0]!.name;
    } else if (this.componentDetail?.hasScaffold) {
      this.componentAction = 'scaffold';
    } else {
      this.componentAction = undefined;
    }
    this.componentArgs = {};
    this.componentFiles = [];
    this.componentTranscript = [];
    this.componentOpError = undefined;
    this.componentDryRunResult = undefined;
    this.componentDryRunHook = undefined;
    this.activeTab = RUNNER_TABS.components;
  }

  selectComponentAction(action: string): void {
    this.componentAction = action;
    this.componentArgs = {};
    this.componentFiles = [];
    this.componentTranscript = [];
    this.componentOpError = undefined;
    this.componentDryRunResult = undefined;
    this.componentDryRunHook = undefined;
  }

  setComponentPreviewResult(
    files: FilePreview[],
    transcript: TranscriptEvent[] = [],
    error?: string
  ): void {
    this.componentFiles = files;
    this.componentTranscript = transcript;
    this.componentOpError = error;
  }

  setComponentDryRunResult(
    hook: string,
    result: unknown | undefined,
    files: FilePreview[],
    transcript: TranscriptEvent[],
    error?: string
  ): void {
    this.componentDryRunHook = hook;
    this.componentDryRunResult = result;
    this.componentFiles = files;
    this.componentTranscript = transcript;
    this.componentOpError = error;
  }

  clearComponentPreview(): void {
    this.componentFiles = [];
    this.componentTranscript = [];
    this.componentOpError = undefined;
    this.componentDryRunResult = undefined;
    this.componentDryRunHook = undefined;
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
      components: this.components,
      stack: this.stack,
      componentInstances: this.componentInstances,
      componentFindings: this.componentFindings,
      selectedComponentId: this.selectedComponentId,
      selectedInstanceId: this.selectedInstanceId,
      componentDetail: this.componentDetail,
      componentAction: this.componentAction,
      componentArgs: this.componentArgs,
      componentFiles: this.componentFiles,
      componentTranscript: this.componentTranscript,
      componentScanError: this.componentScanError,
      componentOpError: this.componentOpError,
      componentDryRunResult: this.componentDryRunResult,
      componentDryRunHook: this.componentDryRunHook,
    };
  }
}
