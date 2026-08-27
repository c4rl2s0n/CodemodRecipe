<script setup lang="ts">
import type {
  ArgValues,
  FilePreview,
  RecipeDiagnostic,
  RecipeSchema,
  TranscriptEvent,
} from '../shared';
import type { FileCardSelection } from '../lib/selection';
import RecipeArgForm from '../components/RecipeArgForm.vue';
import RecipeInfo from '../components/RecipeInfo.vue';
import ReviewPanel from '../components/ReviewPanel.vue';
import { useExtensionClient } from '../composables/useExtensionClient';

defineProps<{
  recipe?: RecipeSchema;
  runnerTitle: string;
  runnerDescription: string;
  argValues: ArgValues;
  previewStatus: string;
  previewStatusKind: 'warn' | 'ok' | '';
  errorMessage: string;
  runtimeDiagnostics: readonly RecipeDiagnostic[];
  showReview: boolean;
  files: FilePreview[];
  transcript: TranscriptEvent[];
  applyMayDiverge: boolean;
  fileSelections: FileCardSelection[];
  activeChangeIndex: number;
  canApply: boolean;
  slotsByRecipe: Record<string, string[]>;
}>();

const emit = defineEmits<{
  'update:argValues': [value: ArgValues];
  argsChanged: [immediate: boolean];
  'update:fileSelections': [value: FileCardSelection[]];
  'update:activeChangeIndex': [value: number];
}>();

const client = useExtensionClient();

function showRecipe(recipe: RecipeSchema | undefined) {
  if (recipe?.sourceFile) {
    client.openRecipeFile(recipe.id);
  }
}

function openDiagnosticSource(
  recipe: RecipeSchema | undefined,
  diag: RecipeDiagnostic
) {
  if (recipe?.id && (diag.sources?.[0]?.file || recipe.sourceFile)) {
    client.openRecipeFile(recipe.id);
  }
}
</script>

<template>
  <div class="runner-header">
    <h2>{{ runnerTitle }}</h2>
    <button
      v-if="recipe?.sourceFile"
      type="button"
      class="secondary show-recipe-btn"
      @click="showRecipe(recipe)"
    >
      Show recipe
    </button>
  </div>
  <div class="desc">{{ runnerDescription }}</div>

  <RecipeInfo
    v-if="recipe"
    :recipe="recipe"
    :slots-by-recipe="slotsByRecipe"
  />

  <template v-if="recipe?.args?.length">
    <h3>Parameters</h3>
    <RecipeArgForm
      :arg-values="argValues"
      :recipe="recipe"
      @update:arg-values="emit('update:argValues', $event)"
      @args-changed="emit('argsChanged', false)"
      @submit-preview="emit('argsChanged', true)"
    />
  </template>

  <div
    class="preview-status"
    :class="{ warn: previewStatusKind === 'warn', ok: previewStatusKind === 'ok' }"
  >
    {{ previewStatus }}
  </div>

  <div v-if="errorMessage" class="error">
    <div>{{ errorMessage }}</div>
    <ul v-if="runtimeDiagnostics.length" class="runtime-diagnostics">
      <li v-for="(diag, index) in runtimeDiagnostics" :key="index">
        <strong>{{ diag.code }}</strong>
        {{ diag.message }}
        <span v-if="diag.hint" class="diag-hint"> — {{ diag.hint }}</span>
        <button
          v-if="diag.sources?.[0]?.file || recipe?.sourceFile"
          type="button"
          class="linkish"
          @click="openDiagnosticSource(recipe, diag)"
        >
          Open recipe
        </button>
      </li>
    </ul>
  </div>

  <ReviewPanel
    v-if="showReview && (files.length || transcript.length)"
    :files="files"
    :transcript="transcript"
    :apply-may-diverge="applyMayDiverge"
    :file-selections="fileSelections"
    :active-change-index="activeChangeIndex"
    :can-apply="canApply"
    @update:file-selections="emit('update:fileSelections', $event)"
    @update:active-change-index="emit('update:activeChangeIndex', $event)"
  />
</template>
