<script setup lang="ts">
import { onMounted, onUnmounted, provide, ref, watch } from 'vue';
import BootstrapView from './views/BootstrapView.vue';
import RecipesView from './views/RecipesView.vue';
import RunnerView from './views/RunnerView.vue';
import ComponentsView from './views/ComponentsView.vue';
import { useRunnerController } from './composables/useRunnerController.js';
import { useHostState } from './composables/useHostState.js';
import { createExtensionInbound } from './extensionInbound.js';
import { createExtensionClient } from './extensionClient.js';
import { postToExtension } from './vsCodeApi.js';
import { extensionClientKey } from './composables/useExtensionClient.js';
import {
  BOOTSTRAP_PHASES,
  RUNNER_TABS,
  type ArgValues,
  type RunnerTab,
} from './shared.js';

const inbound = createExtensionInbound();
const client = createExtensionClient({ post: postToExtension, inbound });
provide(extensionClientKey, client);

const {
  recipe,
  recipes,
  discoveryError,
  diagnostics,
  recipesRefreshing,
  contextMatches,
  slotsByRecipe,
  showBlockingOverlay,
  showBootstrapOverlay,
  bootstrapInFlight,
  bootstrapPhase,
  bootstrapError,
  runnerTitle,
  runnerDescription,
  autoPreviewDebounceMs,
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
} = useHostState(inbound);

const activeTab = ref<RunnerTab>(RUNNER_TABS.recipes);
const localComponentArgs = ref<ArgValues>({});

watch(
  () => componentArgs.value,
  (next) => {
    localComponentArgs.value = { ...(next ?? {}) };
  },
  { immediate: true }
);

const {
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
  canApply,
  restorePersistedOnMount,
  onArgsChanged,
  persistUiState,
} = useRunnerController({
  client,
  inbound,
  recipe,
  autoPreviewDebounceMs,
  activeTab,
  setActiveTab: (tab) => {
    activeTab.value = tab as RunnerTab;
  },
});

function switchTab(tab: RunnerTab) {
  activeTab.value = tab;
  persistUiState();
}

function retryBootstrap(): void {
  client.retryBootstrap();
}

onMounted(() => {
  client.notifyReady();
  restorePersistedOnMount();
  if (recipe.value) {
    onArgsChanged(false);
  }
});

onUnmounted(() => {
  inbound.dispose();
});
</script>

<template>
  <div v-if="showBlockingOverlay" class="bootstrap-screen">
    <BootstrapView
      v-if="showBootstrapOverlay"
      :in-flight="bootstrapInFlight"
      :phase="bootstrapPhase"
      :error="bootstrapError"
      @retry="retryBootstrap"
    />
    <BootstrapView
      v-else
      :in-flight="true"
      :phase="BOOTSTRAP_PHASES.loadingRecipes"
      title="Reloading recipes…"
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
      <button
        type="button"
        class="tab"
        :class="{ active: activeTab === RUNNER_TABS.components }"
        @click="switchTab(RUNNER_TABS.components)"
      >
        Components
      </button>
    </div>

    <div v-show="activeTab === RUNNER_TABS.recipes">
      <RecipesView
        :recipes="recipes"
        :discovery-error="discoveryError"
        :diagnostics="diagnostics"
        :refreshing="recipesRefreshing"
        :context-matches="contextMatches"
        :slots-by-recipe="slotsByRecipe"
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
        :runtime-diagnostics="runtimeDiagnostics"
        :show-review="showReview"
        :files="files"
        :transcript="transcript"
        :apply-may-diverge="applyMayDiverge"
        :file-selections="fileSelections"
        :active-change-index="activeChangeIndex"
        :can-apply="canApply"
        :slots-by-recipe="slotsByRecipe"
        @update:arg-values="argValues = $event"
        @args-changed="onArgsChanged($event)"
        @update:file-selections="fileSelections = $event"
        @update:active-change-index="activeChangeIndex = $event"
      />
    </div>

    <div v-show="activeTab === RUNNER_TABS.components">
      <ComponentsView
        :components="components"
        :stack="stack"
        :instances="componentInstances"
        :findings="componentFindings"
        :selected-component-id="selectedComponentId"
        :selected-instance-id="selectedInstanceId"
        :detail="componentDetail"
        :action="componentAction"
        :args="localComponentArgs"
        :files="componentFiles"
        :transcript="componentTranscript"
        :scan-error="componentScanError"
        :op-error="componentOpError"
        @update:args="localComponentArgs = $event"
      />
    </div>
  </template>
</template>
