<script setup lang="ts">
import { computed } from 'vue';
import { BOOTSTRAP_PHASES, type BootstrapPhase } from '../shared';

const props = defineProps<{
  inFlight: boolean;
  phase: BootstrapPhase;
  error?: string;
}>();

const emit = defineEmits<{
  retry: [];
}>();

const title = computed(() => {
  switch (props.phase) {
    case BOOTSTRAP_PHASES.startingHost:
      return 'Starting extension…';
    case BOOTSTRAP_PHASES.loadingRecipes:
      return 'Loading recipes…';
    case BOOTSTRAP_PHASES.error:
      return 'Failed to start extension.';
    case BOOTSTRAP_PHASES.ready:
      return '';
  }
});
</script>

<template>
  <div class="bootstrap-screen">
    <div class="bootstrap-card">
      <div v-if="inFlight" class="spinner" aria-label="Loading"></div>
      <div class="bootstrap-title">{{ title }}</div>
      <div v-if="phase === BOOTSTRAP_PHASES.error" class="bootstrap-error">
        {{ error }}
      </div>
      <div v-if="phase === BOOTSTRAP_PHASES.error" class="bootstrap-actions">
        <button type="button" @click="emit('retry')">Retry</button>
      </div>
    </div>
  </div>
</template>

