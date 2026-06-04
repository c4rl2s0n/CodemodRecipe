<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue';
import BootstrapView from './views/BootstrapView.vue';
import RecipesView from './views/RecipesView.vue';
import RunnerView from './views/RunnerView.vue';
import { useRunnerController } from './composables/useRunnerController.js';
import {
  BOOTSTRAP_PHASES,
  EXTENSION_TO_WEBVIEW,
  RUNNER_TABS,
  WEBVIEW_TO_EXTENSION,
  type RecipeViewState,
  type RunnerTab,
} from './shared.js';
import { onExtensionMessage, postToExtension } from './vsCodeApi.js';

const boot = window.__CODEMOD_RECIPE_BOOT__ ?? { autoPreviewDebounceMs: 400 };

const hostState = ref<RecipeViewState>({
  recipes: [],
  initialArgs: {},
  activeTab: RUNNER_TABS.recipes,
  autoPreviewDebounceMs: boot.autoPreviewDebounceMs,
  recipesRefreshing: false,
  bootstrapInFlight: true,
  bootstrapPhase: BOOTSTRAP_PHASES.startingHost,
});

const activeTab = ref<RunnerTab>(RUNNER_TABS.recipes);

const recipe = computed(() => hostState.value.recipe);
const recipes = computed(() => hostState.value.recipes);
const discoveryError = computed(() => hostState.value.discoveryError);
const recipesRefreshing = computed(() => hostState.value.recipesRefreshing);
const bootstrapInFlight = computed(() => hostState.value.bootstrapInFlight);
const bootstrapPhase = computed(() => hostState.value.bootstrapPhase);
const bootstrapError = computed(() => hostState.value.bootstrapError);

const showBootstrapOverlay = computed(
  () => bootstrapInFlight.value || bootstrapPhase.value === BOOTSTRAP_PHASES.error
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

const {
  argValues,
  files,
  fileSelections,
  activeChangeIndex,
  showReview,
  errorMessage,
  previewStatus,
  previewStatusKind,
  canApply,
  restorePersistedOnMount,
  handleExtensionMessage: handleRunnerExtensionMessage,
  onArgsChanged,
} = useRunnerController({
  recipe,
  autoPreviewDebounceMs,
  activeTab,
  setActiveTab: (tab) => {
    activeTab.value = tab as RunnerTab;
  },
});

let unsubscribe: (() => void) | undefined;

function switchTab(tab: RunnerTab) {
  activeTab.value = tab;
  persistUiState();
  postToExtension({
    type:
      tab === RUNNER_TABS.recipes
        ? WEBVIEW_TO_EXTENSION.showRecipes
        : WEBVIEW_TO_EXTENSION.showRunner,
  });
}

function retryBootstrap(): void {
  postToExtension({ type: WEBVIEW_TO_EXTENSION.bootstrapRetry });
}

function handleExtensionMessage(msg: import('./messages.js').ExtensionToWebviewMessage) {
  switch (msg.type) {
    case EXTENSION_TO_WEBVIEW.state:
      hostState.value = msg.state;
      handleRunnerExtensionMessage(msg, msg.state);
      break;
    default:
      handleRunnerExtensionMessage(msg);
      break;
  }
}

onMounted(() => {
  postToExtension({ type: WEBVIEW_TO_EXTENSION.ready });
  restorePersistedOnMount();
  unsubscribe = onExtensionMessage(handleExtensionMessage);
  if (recipe.value) {
    onArgsChanged(false);
  }
});

onUnmounted(() => {
  unsubscribe?.();
});

</script>

<template>
  <div v-if="showBootstrapOverlay" class="bootstrap-screen">
    <BootstrapView
      :in-flight="bootstrapInFlight"
      :phase="bootstrapPhase"
      :error="bootstrapError"
      @retry="retryBootstrap"
    />
  </div>

  <template v-else>
    <div class="tabs">
      <button
        type="button"
        class="tab"
        :class="{ active: activeTab === RUNNER_TABS.recipes }"
        @click="switchTab(RUNNER_TABS.recipes)"
      >
        Recipes
      </button>
      <button
        type="button"
        class="tab"
        :class="{ active: activeTab === RUNNER_TABS.runner }"
        @click="switchTab(RUNNER_TABS.runner)"
      >
        Recipe Runner
      </button>
    </div>

    <div v-show="activeTab === RUNNER_TABS.recipes">
      <RecipesView
        :recipes="recipes"
        :discovery-error="discoveryError"
        :refreshing="recipesRefreshing"
      />
    </div>

    <div v-show="activeTab === RUNNER_TABS.runner">
      <RunnerView
        :recipe="recipe"
        :runner-title="runnerTitle"
        :runner-description="runnerDescription"
        :arg-values="argValues"
        :preview-status="previewStatus"
        :preview-status-kind="previewStatusKind"
        :error-message="errorMessage"
        :show-review="showReview"
        :files="files"
        :file-selections="fileSelections"
        :active-change-index="activeChangeIndex"
        :can-apply="canApply"
        @update:arg-values="argValues = $event"
        @args-changed="onArgsChanged($event)"
        @update:file-selections="fileSelections = $event"
        @update:active-change-index="activeChangeIndex = $event"
        @apply="showReview = false"
      />
    </div>
  </template>
</template>
