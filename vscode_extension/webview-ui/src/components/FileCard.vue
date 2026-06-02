<script setup lang="ts">
import { computed } from 'vue';
import type { FilePreview } from '../types';
import type { FileCardSelection } from '../lib/selection';
import PatchRow from './PatchRow.vue';
import { WEBVIEW_TO_EXTENSION } from '../messages';
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
  emit('update:selection', { ...props.selection, include });
}

function updatePatchInclude(patchIndex: number, include: boolean) {
  const patches = props.selection.patches.map((p) =>
    p.index === patchIndex ? { ...p, include } : p
  );
  emit('update:selection', { ...props.selection, patches });
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
      <span class="file-path">
        {{ file.path }}
        <span class="badge">{{ file.isNew ? 'new' : file.kind }}</span>
      </span>
      <div>
        <input
          type="checkbox"
          :checked="selection.include"
          class="file-toggle"
          @click.stop
          @change="updateFileInclude(($event.target as HTMLInputElement).checked)"
        />
        <button type="button" class="secondary pick-btn" @click.stop="openDiff">
          Open Diff
        </button>
      </div>
    </summary>
    <pre v-if="file.snippet" class="file-snippet">{{ file.snippet }}</pre>
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
