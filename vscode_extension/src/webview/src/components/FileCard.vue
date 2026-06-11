<script setup lang="ts">
import { computed } from 'vue';
import type { FilePreview } from '../shared';
import {
  setFileInclude,
  setPatchInclude,
  type FileCardSelection,
} from '../lib/selection';
import PatchRow from './PatchRow.vue';
import { WEBVIEW_TO_EXTENSION } from '../shared';
import { postToExtension } from '../vsCodeApi';

const props = defineProps<{
  file: FilePreview;
  selection: FileCardSelection;
  activePatchKey: string | null;
}>();

const emit = defineEmits<{
  'update:selection': [value: FileCardSelection];
  'select-patch': [path: string, index: number];
}>();

const displayPatches = computed(() => {
  if (props.file.patches.length) {
    return props.file.patches;
  }
  return [
    {
      index: -1,
      offset: 0,
      length: 0,
      description: props.file.isNew ? 'Create file' : 'Whole-file change',
      replacement: props.file.snippet || props.file.modified || '',
      replacementPreview: undefined,
    },
  ];
});

function patchKey(path: string, index: number): string {
  return `${path}:${index}`;
}

function isActive(path: string, index: number): boolean {
  return props.activePatchKey === patchKey(path, index);
}

function updateFileInclude(include: boolean) {
  emit('update:selection', setFileInclude(props.selection, include));
}

function updatePatchInclude(patchIndex: number, include: boolean) {
  emit('update:selection', setPatchInclude(props.selection, patchIndex, include));
}

function openDiff() {
  postToExtension({
    type: WEBVIEW_TO_EXTENSION.openDiff,
    path: props.file.path,
  });
}
</script>

<template>
  <details class="file" open>
    <summary class="file-head">
      <div>
        <input
          type="checkbox"
          :checked="selection.include"
          class="file-toggle"
          @click.stop
          @change="updateFileInclude(($event.target as HTMLInputElement).checked)"
        />
      <span class="file-path">
        {{ file.path }}
        <span class="badge">{{ file.isNew ? 'new' : file.kind }}</span>
      </span>
    </div>
    </summary>
    <PatchRow
      v-for="patch in displayPatches"
      :key="patchKey(file.path, patch.index)"
      :patch="patch"
      :active="isActive(file.path, patch.index)"
      :include="
        selection.patches.find((p) => p.index === patch.index)?.include ?? true
      "
      :is-whole-file="patch.index < 0"
      @select="emit('select-patch', file.path, patch.index)"
      @update:include="updatePatchInclude(patch.index, $event)"
    />
  </details>
</template>
