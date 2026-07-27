import { computed, onUnmounted, ref, watch } from 'vue';
import {
  argsKey,
  collectArgs,
  collectMissingRequiredArgs,
  initArgValues,
  mergeArgValuesOnRefresh,
} from '../lib/args';
import { defaultFileSelections, type FileCardSelection } from '../lib/selection';
import {
  EXTENSION_TO_WEBVIEW,
  type ExtensionToWebviewMessage,
  type FilePreview,
  type PersistedWebviewState,
  type RecipeSchema,
  type RecipeViewState,
} from '../shared';
import type { ExtensionClient } from '../extensionClient';
import type { ExtensionInbound } from '../extensionInbound';
import { getPersistedState, setPersistedState } from '../vsCodeApi';

export function useRunnerController(params: {
  client: ExtensionClient;
  inbound: ExtensionInbound;
  recipe: Readonly<{ value: RecipeSchema | undefined }>;
  autoPreviewDebounceMs: Readonly<{ value: number }>;
  activeTab: Readonly<{ value: string }>;
  setActiveTab: (tab: string) => void;
}) {
  const argValues = ref<Record<string, string>>({});
  const files = ref<FilePreview[]>([]);
  const fileSelections = ref<FileCardSelection[]>([]);
  const activeChangeIndex = ref(0);
  const showReview = ref(false);
  const errorMessage = ref('');
  const previewStatus = ref('');
  const previewStatusKind = ref<'warn' | 'ok' | ''>('');
  const previewInFlight = ref(false);
  const lastPreviewSuccess = ref(false);
  const lastPreviewArgsKey = ref('');
  const latestRequestId = ref(0);
  const latestHandledRequestId = ref(0);
  const pendingAutoPreview = ref(false);
  const wasRecipesRefreshing = ref(false);
  const lastSyncedRecipeId = ref<string | undefined>(undefined);
  let previewDebounce: ReturnType<typeof setTimeout> | undefined;

  const canApply = computed(() => {
    const missing = collectMissingRequiredArgs(params.recipe.value, argValues.value);
    const currentKey = argsKey(collectArgs(params.recipe.value, argValues.value));
    const previewOutOfDate = lastPreviewArgsKey.value !== currentKey;
    return (
      !previewInFlight.value &&
      lastPreviewSuccess.value &&
      missing.length === 0 &&
      !previewOutOfDate &&
      files.value.length > 0
    );
  });

  function persistUiState(): void {
    const persisted: PersistedWebviewState = {
      recipeId: params.recipe.value?.id,
      activeTab: params.activeTab.value as PersistedWebviewState['activeTab'],
      argValues: { ...argValues.value },
      files: files.value,
      activeChangeIndex: activeChangeIndex.value,
      lastPreviewArgsKey: lastPreviewArgsKey.value,
      lastPreviewSuccess: lastPreviewSuccess.value,
    };
    setPersistedState(persisted);
  }

  function restorePersistedForRecipe(recipeId: string | undefined): void {
    const persisted = getPersistedState();
    if (!persisted || !recipeId || persisted.recipeId !== recipeId) {
      return;
    }
    params.setActiveTab(persisted.activeTab);
    argValues.value = { ...persisted.argValues };
    files.value = persisted.files;
    fileSelections.value = defaultFileSelections(persisted.files);
    activeChangeIndex.value = persisted.activeChangeIndex;
    lastPreviewArgsKey.value = persisted.lastPreviewArgsKey;
    lastPreviewSuccess.value = persisted.lastPreviewSuccess;
    showReview.value = persisted.files.length > 0 && persisted.lastPreviewSuccess;
  }

  function setPreviewStatus(text: string, kind: 'warn' | 'ok' | '' = '') {
    previewStatus.value = text;
    previewStatusKind.value = kind;
  }

  function clearError() {
    errorMessage.value = '';
  }

  function showError(msg: string) {
    errorMessage.value = msg;
  }

  function onArgsChanged(immediate: boolean) {
    lastPreviewSuccess.value = false;
    const missing = collectMissingRequiredArgs(params.recipe.value, argValues.value);
    if (missing.length > 0) {
      setPreviewStatus('Missing required: ' + missing.join(', '), 'warn');
    } else {
      setPreviewStatus('Preview out of date');
    }
    persistUiState();
    triggerPreview(immediate);
  }

  function triggerPreview(immediate: boolean) {
    if (!params.recipe.value) return;
    clearTimeout(previewDebounce);
    if (previewInFlight.value) {
      pendingAutoPreview.value = true;
      return;
    }
    const run = () => {
      const requestId = ++latestRequestId.value;
      clearError();
      previewInFlight.value = true;
      setPreviewStatus('Previewing…');
      params.client.requestPreview(
        collectArgs(params.recipe.value, argValues.value),
        requestId
      );
    };
    if (immediate) {
      run();
    } else {
      previewDebounce = setTimeout(run, params.autoPreviewDebounceMs.value);
    }
  }

  function clearPreviewState(): void {
    files.value = [];
    fileSelections.value = [];
    showReview.value = false;
    activeChangeIndex.value = 0;
    lastPreviewSuccess.value = false;
    lastPreviewArgsKey.value = '';
    clearError();
    setPreviewStatus('');
  }

  function applyHostState(state: RecipeViewState): void {
    const recipeId = state.recipe?.id;
    const recipeChanged = recipeId !== lastSyncedRecipeId.value;
    const refreshCompleted = wasRecipesRefreshing.value && !state.recipesRefreshing;
    params.setActiveTab(state.activeTab);

    if (recipeChanged) {
      argValues.value = initArgValues(state.recipe, state.initialArgs);
      clearPreviewState();
      restorePersistedForRecipe(state.recipe?.id);
    } else if (state.recipe) {
      argValues.value = initArgValues(state.recipe, {
        ...argValues.value,
        ...state.initialArgs,
      });
    }

    if (refreshCompleted && state.recipe) {
      argValues.value = mergeArgValuesOnRefresh(state.recipe, argValues.value);
      clearPreviewState();
    }

    persistUiState();
    if (state.recipe && (recipeChanged || refreshCompleted)) {
      onArgsChanged(false);
    }

    lastSyncedRecipeId.value = recipeId;
    wasRecipesRefreshing.value = state.recipesRefreshing;
  }

  function handlePreviewResult(
    msg: Extract<
      ExtensionToWebviewMessage,
      { type: typeof EXTENSION_TO_WEBVIEW.previewResult }
    >
  ) {
    if (
      typeof msg.requestId === 'number' &&
      msg.requestId < latestHandledRequestId.value
    ) {
      return;
    }
    if (typeof msg.requestId === 'number') {
      latestHandledRequestId.value = msg.requestId;
    }
    files.value = msg.files;
    fileSelections.value = defaultFileSelections(msg.files);
    activeChangeIndex.value = 0;
    lastPreviewSuccess.value = true;
    lastPreviewArgsKey.value =
      typeof msg.argsKey === 'string'
        ? msg.argsKey
        : argsKey(collectArgs(params.recipe.value, argValues.value));
    previewInFlight.value = false;
    if (!msg.files.length) {
      setPreviewStatus('');
      showError('No changes produced by this recipe.');
      showReview.value = false;
    } else {
      setPreviewStatus('');
      clearError();
      showReview.value = true;
    }
    persistUiState();
  }

  const unsubscribers = [
    params.inbound.on(EXTENSION_TO_WEBVIEW.state, (msg) => {
      applyHostState(msg.state);
    }),
    params.inbound.on(EXTENSION_TO_WEBVIEW.previewResult, handlePreviewResult),
    params.inbound.on(EXTENSION_TO_WEBVIEW.applyResult, () => {
      showReview.value = false;
      files.value = [];
      fileSelections.value = [];
      persistUiState();
    }),
    params.inbound.on(EXTENSION_TO_WEBVIEW.error, (msg) => {
      if (
        typeof msg.requestId === 'number' &&
        msg.requestId < latestHandledRequestId.value
      ) {
        return;
      }
      previewInFlight.value = false;
      lastPreviewSuccess.value = false;
      setPreviewStatus('Host error', 'warn');
      showError(msg.message);
      persistUiState();
    }),
    params.inbound.on(EXTENSION_TO_WEBVIEW.previewState, (msg) => {
      if (
        typeof msg.requestId === 'number' &&
        msg.requestId < latestHandledRequestId.value
      ) {
        return;
      }
      previewInFlight.value = Boolean(msg.inFlight);
      if (!msg.inFlight && pendingAutoPreview.value) {
        pendingAutoPreview.value = false;
        triggerPreview(false);
      }
    }),
  ];

  onUnmounted(() => {
    for (const off of unsubscribers) {
      off();
    }
  });

  function restorePersistedOnMount() {
    const persisted = getPersistedState();
    if (!persisted) return;
    params.setActiveTab(persisted.activeTab);
    argValues.value = persisted.argValues;
    files.value = persisted.files;
    fileSelections.value = defaultFileSelections(persisted.files);
    activeChangeIndex.value = persisted.activeChangeIndex;
    lastPreviewArgsKey.value = persisted.lastPreviewArgsKey;
    lastPreviewSuccess.value = persisted.lastPreviewSuccess;
    showReview.value = persisted.files.length > 0 && persisted.lastPreviewSuccess;
    lastSyncedRecipeId.value = persisted.recipeId;
  }

  watch([() => params.activeTab.value, argValues, files, activeChangeIndex], persistUiState, {
    deep: true,
  });

  return {
    argValues,
    files,
    fileSelections,
    activeChangeIndex,
    showReview,
    errorMessage,
    previewStatus,
    previewStatusKind,
    previewInFlight,
    canApply,
    restorePersistedOnMount,
    onArgsChanged,
    persistUiState,
  };
}
