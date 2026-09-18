import { computed, onUnmounted, ref } from 'vue';
import {
  BOOTSTRAP_PHASES,
  EXTENSION_TO_WEBVIEW,
  RUNNER_SESSION_PHASE,
  RUNNER_TABS,
  type RecipeViewState,
  type RunnerTab,
} from '../shared';
import type { ExtensionInbound } from '../extensionInbound';

export function useHostState(inbound: ExtensionInbound) {
  const boot = window.__CODEMOD_RECIPE_BOOT__ ?? { autoPreviewDebounceMs: 400 };

  const hostState = ref<RecipeViewState>({
    recipes: [],
    diagnostics: [],
    initialArgs: {},
    activeTab: RUNNER_TABS.recipes,
    autoPreviewDebounceMs: boot.autoPreviewDebounceMs,
    recipesRefreshing: false,
    bootstrapInFlight: true,
    bootstrapPhase: BOOTSTRAP_PHASES.startingHost,
    contextMatches: [],
    slotsByRecipe: {},
    session: {
      id: 0,
      phase: RUNNER_SESSION_PHASE.idle,
    },
  });

  const recipe = computed(() => hostState.value.recipe);
  const recipes = computed(() => hostState.value.recipes);
  const discoveryError = computed(() => hostState.value.discoveryError);
  const diagnostics = computed(() => hostState.value.diagnostics ?? []);
  const recipesRefreshing = computed(() => hostState.value.recipesRefreshing);
  const bootstrapInFlight = computed(() => hostState.value.bootstrapInFlight);
  const bootstrapPhase = computed(() => hostState.value.bootstrapPhase);
  const bootstrapError = computed(() => hostState.value.bootstrapError);
  const contextMatches = computed(() => hostState.value.contextMatches ?? []);
  const slotsByRecipe = computed(() => hostState.value.slotsByRecipe ?? {});
  const components = computed(() => hostState.value.components ?? []);
  const stack = computed(() => hostState.value.stack);
  const componentInstances = computed(
    () => hostState.value.componentInstances ?? []
  );
  const componentFindings = computed(
    () => hostState.value.componentFindings ?? []
  );
  const selectedComponentId = computed(
    () => hostState.value.selectedComponentId
  );
  const selectedInstanceId = computed(() => hostState.value.selectedInstanceId);
  const componentDetail = computed(() => hostState.value.componentDetail);
  const componentAction = computed(() => hostState.value.componentAction);
  const componentArgs = computed(() => hostState.value.componentArgs ?? {});
  const componentFiles = computed(() => hostState.value.componentFiles ?? []);
  const componentTranscript = computed(
    () => hostState.value.componentTranscript ?? []
  );
  const componentScanError = computed(() => hostState.value.componentScanError);
  const componentOpError = computed(() => hostState.value.componentOpError);
  const componentDryRunResult = computed(
    () => hostState.value.componentDryRunResult
  );
  const componentDryRunHook = computed(
    () => hostState.value.componentDryRunHook
  );
  const componentCoverage = computed(() => hostState.value.componentCoverage);

  const showBootstrapOverlay = computed(
    () => bootstrapInFlight.value || bootstrapPhase.value === BOOTSTRAP_PHASES.error
  );

  const showReloadOverlay = computed(
    () => recipesRefreshing.value && !showBootstrapOverlay.value
  );

  const showBlockingOverlay = computed(
    () => showBootstrapOverlay.value || showReloadOverlay.value
  );

  const runnerTitle = computed(() => recipe.value?.name ?? 'Recipe Runner');
  const runnerDescription = computed(
    () =>
      recipe.value?.description ??
      'Select a recipe to configure and preview changes.'
  );

  const autoPreviewDebounceMs = computed(
    () => hostState.value.autoPreviewDebounceMs ?? boot.autoPreviewDebounceMs
  );

  const unsubscribe = inbound.on(EXTENSION_TO_WEBVIEW.state, (msg) => {
    hostState.value = msg.state;
  });

  onUnmounted(() => {
    unsubscribe();
  });

  function applyHostTab(tab: RunnerTab): void {
    hostState.value = { ...hostState.value, activeTab: tab };
  }

  return {
    hostState,
    recipe,
    recipes,
    discoveryError,
    diagnostics,
    recipesRefreshing,
    bootstrapInFlight,
    bootstrapPhase,
    bootstrapError,
    contextMatches,
    slotsByRecipe,
    components,
    stack,
    componentInstances,
    componentFindings,
    selectedComponentId,
    selectedInstanceId,
    componentDetail,
    componentAction,
    componentArgs,
    componentFiles,
    componentTranscript,
    componentScanError,
    componentOpError,
    componentDryRunResult,
    componentDryRunHook,
    componentCoverage,
    showBootstrapOverlay,
    showReloadOverlay,
    showBlockingOverlay,
    runnerTitle,
    runnerDescription,
    autoPreviewDebounceMs,
    applyHostTab,
  };
}
