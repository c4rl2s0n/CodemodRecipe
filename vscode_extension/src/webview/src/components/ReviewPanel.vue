<script setup lang="ts">
import type { FilePreview } from '../shared';
import type { FileCardSelection } from '../lib/selection';
import { buildSelection } from '../lib/selection';
import FileCard from './FileCard.vue';
import { useExtensionClient } from '../composables/useExtensionClient';

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

function updateFileSelection(index: number, selection: FileCardSelection) {
  const next = [...props.fileSelections];
  next[index] = selection;
  emit('update:fileSelections', next);
}

function selectFile(index: number) {
  emit('update:activeChangeIndex', index);
  client.openDiff(props.files[index].path);
}

function previousChange() {
  if (!props.files.length) return;
  const next = Math.max(0, props.activeChangeIndex - 1);
  emit('update:activeChangeIndex', next);
  client.openDiff(props.files[next].path);
}

function nextChange() {
  if (!props.files.length) return;
  const next = Math.min(props.files.length - 1, props.activeChangeIndex + 1);
  emit('update:activeChangeIndex', next);
  client.openDiff(props.files[next].path);
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
      :active="activeChangeIndex === index"
      @update:selection="updateFileSelection(index, $event)"
      @select="selectFile(index)"
    />
    <div class="toolbar">
      <button
        type="button"
        :disabled="!canApply"
        @click="applySelected"
      >
        Apply Selected
      </button>
    </div>
  </div>
</template>
