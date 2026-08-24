<script setup lang="ts">
import { computed } from 'vue';
import type { FilePreview } from '../shared';
import type { FileCardSelection } from '../lib/selection';
import { statsForFile } from '../lib/diffStats';

const props = defineProps<{
  file: FilePreview;
  selection: FileCardSelection;
  active: boolean;
}>();

const emit = defineEmits<{
  'update:selection': [value: FileCardSelection];
  'select': [];
}>();

const fileStats = computed(() => statsForFile(props.file));

function updateInclude(include: boolean) {
  emit('update:selection', { ...props.selection, include });
}
</script>

<template>
  <div
    class="file file-flat"
    :class="{ active }"
    @click="emit('select')"
  >
    <input
      type="checkbox"
      :checked="selection.include"
      class="file-toggle"
      @click.stop
      @change="updateInclude(($event.target as HTMLInputElement).checked)"
    />
    <span class="file-path">
      {{ file.path }}
      <span class="badge">{{ file.isNew ? 'new' : file.kind }}</span>
    </span>
    <span class="diff-stats">
      <span v-if="fileStats.additions > 0" class="diff-stat-add">+{{ fileStats.additions }}</span>
      <span v-if="fileStats.deletions > 0" class="diff-stat-del">-{{ fileStats.deletions }}</span>
    </span>
  </div>
</template>
