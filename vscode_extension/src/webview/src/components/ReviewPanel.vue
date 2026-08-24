<script setup lang="ts">
import { ref } from 'vue';
import type { FilePreview, TranscriptEvent } from '../shared';
import type { FileCardSelection } from '../lib/selection';
import { buildSelection } from '../lib/selection';
import FileCard from './FileCard.vue';
import { useExtensionClient } from '../composables/useExtensionClient';

const client = useExtensionClient();

const props = defineProps<{
  files: FilePreview[];
  transcript: TranscriptEvent[];
  applyMayDiverge: boolean;
  fileSelections: FileCardSelection[];
  activeChangeIndex: number;
  canApply: boolean;
}>();

const emit = defineEmits<{
  'update:fileSelections': [value: FileCardSelection[]];
  'update:activeChangeIndex': [value: number];
}>();

const view = ref<'files' | 'transcript'>('files');

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
    <p v-if="applyMayDiverge" class="disclaimer">
      Apply re-runs this recipe. Skipped post-execution steps are not in the
      preview diffs, so on-disk results may differ after formatters or scripts.
    </p>
    <div class="toolbar">
      <button
        type="button"
        class="secondary"
        :class="{ active: view === 'files' }"
        @click="view = 'files'"
      >
        Files
      </button>
      <button
        type="button"
        class="secondary"
        :class="{ active: view === 'transcript' }"
        @click="view = 'transcript'"
      >
        Transcript
      </button>
    </div>
    <template v-if="view === 'files'">
      <div class="toolbar">
        <button type="button" class="secondary" @click="previousChange">
          Previous Change
        </button>
        <button type="button" class="secondary" @click="nextChange">
          Next Change
        </button>
      </div>
      <p v-if="!files.length" class="empty">No file diffs in this preview.</p>
      <FileCard
        v-for="(file, index) in files"
        :key="file.path"
        :file="file"
        :selection="fileSelections[index]"
        :active="activeChangeIndex === index"
        @update:selection="updateFileSelection(index, $event)"
        @select="selectFile(index)"
      />
    </template>
    <ul v-else class="transcript">
      <li v-for="(event, index) in transcript" :key="index">
        <code>{{ event.kind }}</code>
        {{ event.detail }}
        <span v-if="event.includedInPreview === false" class="skipped">
          (not in preview)
        </span>
      </li>
      <li v-if="!transcript.length">No host API calls recorded.</li>
    </ul>
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

<style scoped>
.disclaimer {
  font-size: 12px;
  opacity: 0.85;
}
.skipped {
  opacity: 0.7;
}
.empty,
.transcript {
  font-size: 12px;
}
.secondary.active {
  font-weight: 600;
}
</style>
