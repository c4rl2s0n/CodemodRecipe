<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue';
import RecipesTab from './components/RecipesTab.vue';
import RecipeArgForm from './components/RecipeArgForm.vue';
import ReviewPanel from './components/ReviewPanel.vue';
import {
  argsKey,
  collectArgs,
  collectMissingRequiredArgs,
  initArgValues,
} from './lib/args';
import { defaultFileSelections, type FileCardSelection } from './lib/selection';
import { EXTENSION_TO_WEBVIEW, WEBVIEW_TO_EXTENSION } from './messages';
import type {
  FilePreview,
  PersistedWebviewState,
  RecipeViewState,
  RunnerTab,
} from './types';
import {
  getPersistedState,
  onExtensionMessage,
  postToExtension,
  setPersistedState,
} from './vsCodeApi';

const boot = window.__CODEMOD_RECIPE_BOOT__ ?? { autoPreviewDebounceMs: 400 };

const hostState = ref<RecipeViewState>({
  recipes: [],
  initialArgs: {},
  activeTab: 'recipes',
  autoPreviewDebounceMs: boot.autoPreviewDebounceMs,
});

const activeTab = ref<RunnerTab>('recipes');
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
let previewDebounce: ReturnType<typeof setTimeout> | undefined;

const recipe = computed(() => hostState.value.recipe);
const recipes = computed(() => hostState.value.recipes);
const discoveryError = computed(() => hostState.value.discoveryError);

const runnerTitle = computed(() => recipe.value?.name ?? 'Recipe Runner');
const runnerDescription = computed(
  () =>
    recipe.value?.description ??
    'Select a recipe to configure and preview changes.'
);

const autoPreviewDebounceMs = computed(
  () => hostState.value.autoPreviewDebounceMs ?? boot.autoPreviewDebounceMs
);

const canApply = computed(() => {
  const missing = collectMissingRequiredArgs(recipe.value, argValues.value);
  const currentKey = argsKey(collectArgs(recipe.value, argValues.value));
  const previewOutOfDate = lastPreviewArgsKey.value !== currentKey;
  return (
    !previewInFlight.value &&
    lastPreviewSuccess.value &&
    missing.length === 0 &&
    !previewOutOfDate &&
    files.value.length > 0
  );
});

let unsubscribe: (() => void) | undefined;

function persistUiState(): void {
  const persisted: PersistedWebviewState = {
    recipeId: recipe.value?.id,
    activeTab: activeTab.value,
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
  argValues.value = { ...persisted.argValues };
  files.value = persisted.files;
  fileSelections.value = defaultFileSelections(persisted.files);
  activeChangeIndex.value = persisted.activeChangeIndex;
  lastPreviewArgsKey.value = persisted.lastPreviewArgsKey;
  lastPreviewSuccess.value = persisted.lastPreviewSuccess;
  showReview.value =
    persisted.files.length > 0 && persisted.lastPreviewSuccess;
}

function applyHostState(state: RecipeViewState): void {
  const recipeChanged = state.recipe?.id !== hostState.value.recipe?.id;
  hostState.value = state;
  activeTab.value = state.activeTab;

  if (recipeChanged) {
    argValues.value = initArgValues(state.recipe, state.initialArgs);
    files.value = [];
    fileSelections.value = [];
    showReview.value = false;
    activeChangeIndex.value = 0;
    lastPreviewSuccess.value = false;
    lastPreviewArgsKey.value = '';
    restorePersistedForRecipe(state.recipe?.id);
  }

  if (state.recipe && !recipeChanged) {
    argValues.value = initArgValues(state.recipe, {
      ...argValues.value,
      ...state.initialArgs,
    });
  }

  persistUiState();
  if (state.recipe && recipeChanged) {
    onArgsChanged(false);
  }
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
  const missing = collectMissingRequiredArgs(recipe.value, argValues.value);
  if (missing.length > 0) {
    setPreviewStatus('Missing required: ' + missing.join(', '), 'warn');
  } else {
    setPreviewStatus('Preview out of date');
  }
  persistUiState();
  triggerPreview(immediate);
}

function triggerPreview(immediate: boolean) {
  if (!recipe.value) return;
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
    postToExtension({
      type: WEBVIEW_TO_EXTENSION.preview,
      args: collectArgs(recipe.value, argValues.value),
      requestId,
    });
  };
  if (immediate) {
    run();
  } else {
    previewDebounce = setTimeout(run, autoPreviewDebounceMs.value);
  }
}

function switchTab(tab: RunnerTab) {
  activeTab.value = tab;
  persistUiState();
  postToExtension({
    type:
      tab === 'recipes'
        ? WEBVIEW_TO_EXTENSION.showRecipes
        : WEBVIEW_TO_EXTENSION.showRunner,
  });
}

function handleExtensionMessage(msg: import('./messages').ExtensionToWebviewMessage) {
  switch (msg.type) {
    case EXTENSION_TO_WEBVIEW.state:
      applyHostState(msg.state);
      break;
    case EXTENSION_TO_WEBVIEW.filePicked:
      argValues.value = { ...argValues.value, [msg.arg]: msg.value };
      onArgsChanged(false);
      break;
    case EXTENSION_TO_WEBVIEW.previewResult:
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
          : argsKey(collectArgs(recipe.value, argValues.value));
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
      break;
    case EXTENSION_TO_WEBVIEW.applyResult:
      showReview.value = false;
      files.value = [];
      fileSelections.value = [];
      persistUiState();
      break;
    case EXTENSION_TO_WEBVIEW.error:
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
      break;
    case EXTENSION_TO_WEBVIEW.previewState:
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
      break;
  }
}

onMounted(() => {
  const persisted = getPersistedState();
  if (persisted) {
    activeTab.value = persisted.activeTab;
    argValues.value = persisted.argValues;
    files.value = persisted.files;
    fileSelections.value = defaultFileSelections(persisted.files);
    activeChangeIndex.value = persisted.activeChangeIndex;
    lastPreviewArgsKey.value = persisted.lastPreviewArgsKey;
    lastPreviewSuccess.value = persisted.lastPreviewSuccess;
    showReview.value =
      persisted.files.length > 0 && persisted.lastPreviewSuccess;
  }
  unsubscribe = onExtensionMessage(handleExtensionMessage);
  setPreviewStatus('Preview out of date');
  if (recipe.value) {
    onArgsChanged(false);
  }
});

onUnmounted(() => {
  unsubscribe?.();
});

watch([activeTab, argValues, files, activeChangeIndex], persistUiState, {
  deep: true,
});
</script>

<template>
  <div class="tabs">
    <button
      type="button"
      class="tab"
      :class="{ active: activeTab === 'recipes' }"
      @click="switchTab('recipes')"
    >
      Recipes
    </button>
    <button
      type="button"
      class="tab"
      :class="{ active: activeTab === 'runner' }"
      @click="switchTab('runner')"
    >
      Recipe Runner
    </button>
  </div>

  <div v-show="activeTab === 'recipes'">
    <RecipesTab :recipes="recipes" :discovery-error="discoveryError" />
  </div>

  <div v-show="activeTab === 'runner'">
    <h2>{{ runnerTitle }}</h2>
    <div class="desc">{{ runnerDescription }}</div>

    <h3>Parameters</h3>
    <RecipeArgForm
      v-model:arg-values="argValues"
      :recipe="recipe"
      @args-changed="onArgsChanged(false)"
      @submit-preview="onArgsChanged(true)"
    />

    <div
      class="preview-status"
      :class="{ warn: previewStatusKind === 'warn', ok: previewStatusKind === 'ok' }"
    >
      {{ previewStatus }}
    </div>

    <div v-if="errorMessage" class="error">{{ errorMessage }}</div>

    <ReviewPanel
      v-if="showReview && files.length"
      :files="files"
      :file-selections="fileSelections"
      :active-change-index="activeChangeIndex"
      :can-apply="canApply"
      @update:file-selections="fileSelections = $event"
      @update:active-change-index="activeChangeIndex = $event"
      @apply="showReview = false"
    />
  </div>
</template>
