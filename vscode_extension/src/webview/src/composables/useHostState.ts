import { computed, onMounted, onUnmounted, ref } from 'vue';
import {
  BOOTSTRAP_PHASES,
  EXTENSION_TO_WEBVIEW,
  WEBVIEW_TO_EXTENSION,
  type ExtensionToWebviewMessage,
  type RecipeViewState,
} from '../shared';
import { onExtensionMessage, postToExtension } from '../vsCodeApi';

export function useHostState() {
  const boot = (window as unknown as { __CODEMOD_RECIPE_BOOT__?: { autoPreviewDebounceMs: number } })
    .__CODEMOD_RECIPE_BOOT__ ?? { autoPreviewDebounceMs: 400 };

  const hostState = ref<RecipeViewState>({
    recipes: [],
    initialArgs: {},
    activeTab: 'recipes',
    autoPreviewDebounceMs: boot.autoPreviewDebounceMs,
    recipesRefreshing: false,
    bootstrapInFlight: true,
    bootstrapPhase: BOOTSTRAP_PHASES.startingHost,
  });

  const recipe = computed(() => hostState.value.recipe);
  const recipes = computed(() => hostState.value.recipes);
  const discoveryError = computed(() => hostState.value.discoveryError);
  const recipesRefreshing = computed(() => hostState.value.recipesRefreshing);
  const bootstrapInFlight = computed(() => hostState.value.bootstrapInFlight);
  const bootstrapPhase = computed(() => hostState.value.bootstrapPhase);
  const bootstrapError = computed(() => hostState.value.bootstrapError);
  const autoPreviewDebounceMs = computed(
    () => hostState.value.autoPreviewDebounceMs ?? boot.autoPreviewDebounceMs
  );

  let unsubscribe: (() => void) | undefined;

  function applyHostState(state: RecipeViewState): void {
    hostState.value = state;
  }

  function handleExtensionMessage(msg: ExtensionToWebviewMessage) {
    if (msg.type === EXTENSION_TO_WEBVIEW.state) {
      applyHostState(msg.state);
    }
  }

  onMounted(() => {
    postToExtension({ type: WEBVIEW_TO_EXTENSION.ready });
    unsubscribe = onExtensionMessage(handleExtensionMessage);
  });

  onUnmounted(() => {
    unsubscribe?.();
  });

  return {
    hostState,
    recipe,
    recipes,
    discoveryError,
    recipesRefreshing,
    bootstrapInFlight,
    bootstrapPhase,
    bootstrapError,
    autoPreviewDebounceMs,
  };
}

