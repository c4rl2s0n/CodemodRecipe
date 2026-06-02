<script setup lang="ts">
import { computed } from 'vue';
import type { FilePreview } from '../types';
import type { FileCardSelection } from '../lib/selection';
import { allPatchRows } from '../lib/selection';
import FileCard from './FileCard.vue';
import { WEBVIEW_TO_EXTENSION } from '../messages';
import { postToExtension } from '../vsCodeApi';
import { buildSelection } from '../lib/selection';

const props = defineProps<{
  files: FilePreview[];
  fileSelections: FileCardSelection[];
  activeChangeIndex: number;
  canApply: boolean;
}>();

const emit = defineEmits<{
  'update:fileSelections': [value: FileCardSelection[]];
  'update:activeChangeIndex': [value: number];
  apply: [];
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
    postToExtension({ type: WEBVIEW_TO_EXTENSION.openDiff, path });
  }
}

function previousChange() {
  const rows = allPatchRows(props.fileSelections);
  if (!rows.length) return;
  const next = Math.max(0, props.activeChangeIndex - 1);
  emit('update:activeChangeIndex', next);
  postToExtension({ type: WEBVIEW_TO_EXTENSION.openDiff, path: rows[next].path });
}

function nextChange() {
  const rows = allPatchRows(props.fileSelections);
  if (!rows.length) return;
  const next = Math.min(rows.length - 1, props.activeChangeIndex + 1);
  emit('update:activeChangeIndex', next);
  postToExtension({ type: WEBVIEW_TO_EXTENSION.openDiff, path: rows[next].path });
}

function applySelected() {
  postToExtension({
    type: WEBVIEW_TO_EXTENSION.apply,
    selection: buildSelection(props.fileSelections),
  });
  emit('apply');
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
