<script setup lang="ts">
import { computed } from 'vue';
import type { FilePreview } from '../shared';
import type { FileCardSelection } from '../lib/selection';
import { allPatchRows } from '../lib/selection';
import FileCard from './FileCard.vue';
import { useExtensionClient } from '../composables/useExtensionClient';
import { buildSelection } from '../lib/selection';

const client = useExtensionClient();

const props = defineProps<{
  files: FilePreview[];
  fileSelections: FileCardSelection[];
  activeChangeIndex: number;
  canApply: boolean;
}>();

const emit = defineEmits<{
  'update:fileSelections': [value: FileCardSelection[]];
  'update:activeChangeIndex': [value: number];
}>();

const activePatchKey = computed(() => {
  const rows = allPatchRows(props.fileSelections);
  const row = rows[props.activeChangeIndex];
  return row ? `${row.path}:${row.index}` : null;
});

function updateFileSelection(index: number, selection: FileCardSelection) {
  const next = [...props.fileSelections];
  next[index] = selection;
  emit('update:fileSelections', next);
}

function selectPatch(path: string, index: number) {
  const rows = allPatchRows(props.fileSelections);
  const idx = rows.findIndex((r) => r.path === path && r.index === index);
  if (idx >= 0) {
    emit('update:activeChangeIndex', idx);
    client.openDiff(path, index);
  }
}

function previousChange() {
  const rows = allPatchRows(props.fileSelections);
  if (!rows.length) return;
  const next = Math.max(0, props.activeChangeIndex - 1);
  emit('update:activeChangeIndex', next);
  client.openDiff(rows[next].path, rows[next].index);
}

function nextChange() {
  const rows = allPatchRows(props.fileSelections);
  if (!rows.length) return;
  const next = Math.min(rows.length - 1, props.activeChangeIndex + 1);
  emit('update:activeChangeIndex', next);
  client.openDiff(rows[next].path, rows[next].index);
}

function applySelected() {
  client.apply(buildSelection(props.fileSelections));
}
</script>

<template>
  <div>
    <h3>Review changes</h3>
    <div class="toolbar">
      <button type="button" class="secondary" @click="previousChange">
        Previous Change
      </button>
      <button type="button" class="secondary" @click="nextChange">
        Next Change
      </button>
    </div>
    <FileCard
      v-for="(file, index) in files"
      :key="file.path"
      :file="file"
      :selection="fileSelections[index]"
      :active-patch-key="activePatchKey"
      @update:selection="updateFileSelection(index, $event)"
      @select-patch="selectPatch"
    />
    <div class="toolbar">
      <button type="button" :disabled="!canApply" @click="applySelected">
        Apply Selected
      </button>
    </div>
  </div>
</template>
