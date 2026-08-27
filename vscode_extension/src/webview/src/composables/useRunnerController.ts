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
  RUNNER_SESSION_PHASE,
  type ArgValues,
  type ExtensionToWebviewMessage,
  type FilePreview,
  type PersistedWebviewState,
  type RecipeSchema,
  type RecipeViewState,
  type TranscriptEvent,
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
  const argValues = ref<ArgValues>({});
  const files = ref<FilePreview[]>([]);
  const transcript = ref<TranscriptEvent[]>([]);
  const applyMayDiverge = ref(false);
  const fileSelections = ref<FileCardSelection[]>([]);
  const activeChangeIndex = ref(0);
  const showReview = ref(false);
  const errorMessage = ref('');
  const runtimeDiagnostics = ref<import('../shared').RecipeDiagnostic[]>([]);
  const previewStatus = ref('');
  const previewStatusKind = ref<'warn' | 'ok' | ''>('');
  /** Local debounce/queue only — host session.phase is authoritative for ops. */
  const previewInFlight = ref(false);
  const lastPreviewSuccess = ref(false);
  const lastPreviewArgsKey = ref('');
  const latestRequestId = ref(0);
  const inFlightRequestId = ref<number | undefined>(undefined);
  const latestHandledRequestId = ref(0);
  const wasRecipesRefreshing = ref(false);
  const lastSyncedRecipeId = ref<string | undefined>(undefined);
  const lastHostSessionId = ref<number | undefined>(undefined);
  const tabOwnedLocally = ref(false);
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
      (files.value.length > 0 || transcript.value.length > 0)
    );
  });

  function persistUiState(): void {
    const persisted: PersistedWebviewState = {
      recipeId: params.recipe.value?.id,
      activeTab: params.activeTab.value as PersistedWebviewState['activeTab'],
      argValues: { ...argValues.value },
      files: files.value,
      transcript: transcript.value,
      applyMayDiverge: applyMayDiverge.value,
      activeChangeIndex: activeChangeIndex.value,
      lastPreviewArgsKey: lastPreviewArgsKey.value,
      // Never persist Apply-ready across host restarts (token lives on host only).
      lastPreviewSuccess: false,
    };
    setPersistedState(persisted);
  }

  function restorePersistedForRecipe(recipeId: string | undefined): void {
    const persisted = getPersistedState();
    if (!persisted || !recipeId || persisted.recipeId !== recipeId) {
      return;
    }
    params.setActiveTab(persisted.activeTab);
    tabOwnedLocally.value = true;
    argValues.value = { ...persisted.argValues };
    files.value = persisted.files;
    transcript.value = persisted.transcript ?? [];
    applyMayDiverge.value = persisted.applyMayDiverge ?? false;
    fileSelections.value = defaultFileSelections(persisted.files);
    activeChangeIndex.value = persisted.activeChangeIndex;
    lastPreviewArgsKey.value = persisted.lastPreviewArgsKey;
    // Host token is gone after reload — require a fresh preview before Apply.
    lastPreviewSuccess.value = false;
    showReview.value =
      persisted.files.length > 0 || (persisted.transcript ?? []).length > 0;
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
      persistUiState();
      // Do not hit the host until required args are filled.
      return;
    }
    setPreviewStatus('Preview out of date');
    persistUiState();
    triggerPreview(immediate);
  }

  function triggerPreview(immediate: boolean) {
    if (!params.recipe.value) return;
    clearTimeout(previewDebounce);
    // Always send after debounce; host supersedes any in-flight preview (kill+rerun).
    const run = () => {
      const requestId = params.client.nextRequestId
        ? params.client.nextRequestId()
        : ++latestRequestId.value;
      latestRequestId.value = requestId;
      inFlightRequestId.value = requestId;
      clearError();
      runtimeDiagnostics.value = [];
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
    clearTimeout(previewDebounce);
    previewDebounce = undefined;
    previewInFlight.value = false;
    inFlightRequestId.value = undefined;
    files.value = [];
    transcript.value = [];
    applyMayDiverge.value = false;
    fileSelections.value = [];
    showReview.value = false;
    activeChangeIndex.value = 0;
    lastPreviewSuccess.value = false;
    lastPreviewArgsKey.value = '';
    runtimeDiagnostics.value = [];
    clearError();
    setPreviewStatus('');
  }

  function isStaleRequest(requestId: number | undefined): boolean {
    return (
      typeof requestId === 'number' && requestId < latestHandledRequestId.value
    );
  }

  function markHandled(requestId: number | undefined): void {
    if (typeof requestId === 'number') {
      latestHandledRequestId.value = Math.max(
        latestHandledRequestId.value,
        requestId
      );
    }
  }

  function clearInFlightIfCurrent(requestId: number | undefined): void {
    if (
      typeof requestId === 'number' &&
      inFlightRequestId.value !== undefined &&
      requestId !== inFlightRequestId.value &&
      requestId < latestRequestId.value
    ) {
      // Completion for an older flight while a newer one is authoritative.
      return;
    }
    if (
      typeof requestId === 'number' &&
      inFlightRequestId.value !== undefined &&
      requestId === inFlightRequestId.value
    ) {
      previewInFlight.value = false;
      inFlightRequestId.value = undefined;
      return;
    }
    // previewState / error without matching id still clears local wait if we are waiting.
    if (previewInFlight.value && typeof requestId === 'number') {
      if (requestId === latestRequestId.value) {
        previewInFlight.value = false;
        inFlightRequestId.value = undefined;
      }
    }
  }

  function applyHostState(state: RecipeViewState): void {
    const recipeId = state.recipe?.id;
    const recipeChanged = recipeId !== lastSyncedRecipeId.value;
    const refreshCompleted = wasRecipesRefreshing.value && !state.recipesRefreshing;
    const sessionChanged =
      lastHostSessionId.value !== undefined &&
      state.session?.id !== lastHostSessionId.value;

    // Only apply host tab on recipe select / first sync — user owns tab otherwise.
    if (recipeChanged || !tabOwnedLocally.value) {
      params.setActiveTab(state.activeTab);
      tabOwnedLocally.value = true;
    }

    if (sessionChanged) {
      clearPreviewState();
    }

    if (recipeChanged) {
      argValues.value = initArgValues(state.recipe, state.initialArgs);
      clearPreviewState();
      restorePersistedForRecipe(state.recipe?.id);
    }
    // Same recipe: do not rewrite argValues from initialArgs (context postState).

    if (refreshCompleted && state.recipe) {
      argValues.value = mergeArgValuesOnRefresh(state.recipe, argValues.value);
      clearPreviewState();
    }

    if (state.session?.phase === RUNNER_SESSION_PHASE.previewing) {
      previewInFlight.value = true;
      if (state.session.previewStatus) {
        setPreviewStatus(state.session.previewStatus);
      }
    } else if (
      state.session?.phase === RUNNER_SESSION_PHASE.idle ||
      state.session?.phase === RUNNER_SESSION_PHASE.reviewing
    ) {
      // Host not previewing — if we were waiting with no matching request, recover.
      if (previewInFlight.value && inFlightRequestId.value === undefined) {
        previewInFlight.value = false;
      }
    }

    if (
      state.session?.phase === RUNNER_SESSION_PHASE.reviewing &&
      state.session.lastPreviewArgsKey &&
      lastPreviewSuccess.value
    ) {
      lastPreviewArgsKey.value = state.session.lastPreviewArgsKey;
    }

    persistUiState();
    if (state.recipe && (recipeChanged || refreshCompleted || sessionChanged)) {
      onArgsChanged(false);
    }

    lastSyncedRecipeId.value = recipeId;
    lastHostSessionId.value = state.session?.id;
    wasRecipesRefreshing.value = state.recipesRefreshing;
  }

  function handlePreviewResult(
    msg: Extract<
      ExtensionToWebviewMessage,
      { type: typeof EXTENSION_TO_WEBVIEW.previewResult }
    >
  ) {
    if (isStaleRequest(msg.requestId)) {
      return;
    }
    markHandled(msg.requestId);
    clearInFlightIfCurrent(msg.requestId);
    previewInFlight.value = false;
    inFlightRequestId.value = undefined;
    files.value = msg.files;
    transcript.value = msg.transcript ?? [];
    applyMayDiverge.value = msg.applyMayDiverge ?? false;
    fileSelections.value = defaultFileSelections(msg.files);
    activeChangeIndex.value = 0;
    lastPreviewSuccess.value = true;
    lastPreviewArgsKey.value =
      typeof msg.argsKey === 'string'
        ? msg.argsKey
        : argsKey(collectArgs(params.recipe.value, argValues.value));
    if (!msg.files.length && !(msg.transcript ?? []).length) {
      setPreviewStatus('');
      showError('No changes produced by this recipe.');
      showReview.value = false;
    } else {
      setPreviewStatus('');
      clearError();
      runtimeDiagnostics.value = [];
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
      transcript.value = [];
      applyMayDiverge.value = false;
      fileSelections.value = [];
      lastPreviewSuccess.value = false;
      lastPreviewArgsKey.value = '';
      runtimeDiagnostics.value = [];
      persistUiState();
    }),
    params.inbound.on(EXTENSION_TO_WEBVIEW.error, (msg) => {
      if (isStaleRequest(msg.requestId)) {
        return;
      }
      markHandled(msg.requestId);
      clearInFlightIfCurrent(msg.requestId);
      previewInFlight.value = false;
      inFlightRequestId.value = undefined;
      lastPreviewSuccess.value = false;
      setPreviewStatus('Host error', 'warn');
      showError(msg.message);
      runtimeDiagnostics.value = msg.diagnostics ?? [];
      if (msg.transcript && msg.transcript.length > 0) {
        transcript.value = msg.transcript;
        showReview.value = true;
      }
      persistUiState();
    }),
    params.inbound.on(EXTENSION_TO_WEBVIEW.previewState, (msg) => {
      // Busy ACK (inFlight true) must NOT markHandled — that caused forever-Previewing.
      if (msg.inFlight) {
        if (
          typeof msg.requestId === 'number' &&
          msg.requestId === latestRequestId.value
        ) {
          previewInFlight.value = true;
          inFlightRequestId.value = msg.requestId;
        }
        return;
      }
      // inFlight false — only clear if this is the current flight (or older completion).
      if (
        typeof msg.requestId === 'number' &&
        inFlightRequestId.value !== undefined &&
        msg.requestId !== inFlightRequestId.value &&
        msg.requestId < latestRequestId.value
      ) {
        return;
      }
      if (!isStaleRequest(msg.requestId)) {
        markHandled(msg.requestId);
      }
      clearInFlightIfCurrent(msg.requestId);
      if (
        typeof msg.requestId !== 'number' ||
        msg.requestId === latestRequestId.value ||
        msg.requestId === inFlightRequestId.value
      ) {
        previewInFlight.value = false;
        inFlightRequestId.value = undefined;
      }
    }),
  ];

  onUnmounted(() => {
    clearTimeout(previewDebounce);
    for (const off of unsubscribers) {
      off();
    }
  });

  function restorePersistedOnMount() {
    const persisted = getPersistedState();
    if (!persisted) return;
    params.setActiveTab(persisted.activeTab);
    tabOwnedLocally.value = true;
    argValues.value = persisted.argValues;
    files.value = persisted.files;
    transcript.value = persisted.transcript ?? [];
    applyMayDiverge.value = persisted.applyMayDiverge ?? false;
    fileSelections.value = defaultFileSelections(persisted.files);
    activeChangeIndex.value = persisted.activeChangeIndex;
    lastPreviewArgsKey.value = persisted.lastPreviewArgsKey;
    lastPreviewSuccess.value = false;
    showReview.value =
      persisted.files.length > 0 || (persisted.transcript ?? []).length > 0;
    lastSyncedRecipeId.value = persisted.recipeId;
  }

  watch(
    [() => params.activeTab.value, argValues, files, transcript, applyMayDiverge, activeChangeIndex],
    persistUiState,
    { deep: true }
  );

  return {
    argValues,
    files,
    transcript,
    applyMayDiverge,
    fileSelections,
    activeChangeIndex,
    showReview,
    errorMessage,
    runtimeDiagnostics,
    previewStatus,
    previewStatusKind,
    previewInFlight,
    canApply,
    restorePersistedOnMount,
    onArgsChanged,
    persistUiState,
  };
}
