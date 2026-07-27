<script setup lang="ts">
import { computed } from 'vue';
import type { FilePreview, PatchInfo } from '../shared';
import {
  setFileInclude,
  setPatchInclude,
  type FileCardSelection,
} from '../lib/selection';
import {
  statsForFile,
  statsForPatch,
  sumStats,
  type LineChangeStats,
} from '../lib/diffStats';
import PatchRow from './PatchRow.vue';

const props = defineProps<{
  file: FilePreview;
  selection: FileCardSelection;
  activePatchKey: string | null;
}>();

const emit = defineEmits<{
  'update:selection': [value: FileCardSelection];
  'select-patch': [path: string, index: number];
}>();

const displayPatches = computed((): PatchInfo[] => {
  if (props.file.patches.length) {
    return props.file.patches;
  }
  return [
    {
      index: -1,
      offset: 0,
      length: 0,
      description: props.file.isNew ? 'Create file' : 'Whole-file change',
    },
  ];
});

const flattened = computed(() => displayPatches.value.length === 1);

const fileStats = computed((): LineChangeStats => {
  if (props.file.patches.length > 1) {
    return sumStats(
      props.file.patches.map((patch) => statsForPatch(props.file, patch))
    );
  }
  return statsForPatch(props.file, displayPatches.value[0]);
});

function patchKey(path: string, index: number): string {
  return `${path}:${index}`;
}

function isActive(path: string, index: number): boolean {
  return props.activePatchKey === patchKey(path, index);
}

function patchLabel(patch: PatchInfo, position: number): string {
  if (patch.description) {
    return patch.description;
  }
  if (patch.index < 0) {
    return props.file.isNew ? 'Create file' : 'Whole-file change';
  }
  return `Change ${position + 1}`;
}

function updateFileInclude(include: boolean) {
  emit('update:selection', setFileInclude(props.selection, include));
}

function updatePatchInclude(patchIndex: number, include: boolean) {
  emit('update:selection', setPatchInclude(props.selection, patchIndex, include));
}

function onFlattenedSelect() {
  const patch = displayPatches.value[0];
  emit('select-patch', props.file.path, patch.index);
}

function patchInclude(patchIndex: number): boolean {
  return (
    props.selection.patches.find((p) => p.index === patchIndex)?.include ?? true
  );
}
</script>

<template>
  <div v-if="flattened" class="file file-flat" :class="{ active: isActive(file.path, displayPatches[0].index) }" @click="onFlattenedSelect">
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
    <span class="diff-stats">
      <span v-if="fileStats.additions > 0" class="diff-stat-add">+{{ fileStats.additions }}</span>
      <span v-if="fileStats.deletions > 0" class="diff-stat-del">-{{ fileStats.deletions }}</span>
    </span>
  </div>

  <details v-else class="file" open>
    <summary class="file-head">
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
      <span class="diff-stats">
        <span v-if="fileStats.additions > 0" class="diff-stat-add">+{{ fileStats.additions }}</span>
        <span v-if="fileStats.deletions > 0" class="diff-stat-del">-{{ fileStats.deletions }}</span>
      </span>
    </summary>
    <PatchRow
      v-for="(patch, position) in displayPatches"
      :key="patchKey(file.path, patch.index)"
      :label="patchLabel(patch, position)"
      :stats="statsForPatch(file, patch)"
      :active="isActive(file.path, patch.index)"
      :include="patchInclude(patch.index)"
      @select="emit('select-patch', file.path, patch.index)"
      @update:include="updatePatchInclude(patch.index, $event)"
    />
  </details>
</template>
