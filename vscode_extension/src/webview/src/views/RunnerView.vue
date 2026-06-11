<script setup lang="ts">
import type { FilePreview, RecipeSchema } from '../shared';
import type { FileCardSelection } from '../lib/selection';
import RecipeArgForm from '../components/RecipeArgForm.vue';
import ReviewPanel from '../components/ReviewPanel.vue';

defineProps<{
  recipe?: RecipeSchema;
  runnerTitle: string;
  runnerDescription: string;
  argValues: Record<string, string>;
  previewStatus: string;
  previewStatusKind: 'warn' | 'ok' | '';
  errorMessage: string;
  showReview: boolean;
  files: FilePreview[];
  fileSelections: FileCardSelection[];
  activeChangeIndex: number;
  canApply: boolean;
}>();

const emit = defineEmits<{
  'update:argValues': [value: Record<string, string>];
  argsChanged: [immediate: boolean];
  'update:fileSelections': [value: FileCardSelection[]];
  'update:activeChangeIndex': [value: number];
  apply: [];
}>();
</script>

<template>
  <h2>{{ runnerTitle }}</h2>
  <div class="desc">{{ runnerDescription }}</div>

  <h3>Parameters</h3>
  <RecipeArgForm
    :arg-values="argValues"
    :recipe="recipe"
    @update:arg-values="emit('update:argValues', $event)"
    @args-changed="emit('argsChanged', false)"
    @submit-preview="emit('argsChanged', true)"
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
    @update:file-selections="emit('update:fileSelections', $event)"
    @update:active-change-index="emit('update:activeChangeIndex', $event)"
    @apply="emit('apply')"
  />
</template>

