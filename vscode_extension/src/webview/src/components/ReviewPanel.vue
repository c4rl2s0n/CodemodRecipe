<script setup lang="ts">
import { ref } from 'vue';
import type { FilePreview, TranscriptEvent } from '../shared';
import type { FileCardSelection } from '../lib/selection';
import { buildSelection } from '../lib/selection';
import {
  eventIsExpandable,
  formatExpandedDetail,
  formatStatsBadges,
} from '../lib/transcript';
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
const expanded = ref<Record<number, boolean>>({});

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

function toggleExpand(index: number, event: TranscriptEvent) {
  if (!eventIsExpandable(event)) {
    return;
  }
  expanded.value = {
    ...expanded.value,
    [index]: !expanded.value[index],
  };
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
      <li
        v-for="(event, index) in transcript"
        :key="index"
        class="transcript-row"
        :class="{
          'transcript-log': event.kind === 'log',
          expandable: eventIsExpandable(event),
          expanded: expanded[index],
        }"
        @click="toggleExpand(index, event)"
      >
        <div class="transcript-summary">
          <span v-if="eventIsExpandable(event)" class="chevron">{{
            expanded[index] ? '▾' : '▸'
          }}</span>
          <code>{{ event.kind }}</code>
          <span
            v-for="badge in formatStatsBadges(event)"
            :key="badge"
            class="stat-badge"
            :class="{ add: badge.startsWith('+'), remove: badge.startsWith('-') }"
          >
            {{ badge }}
          </span>
          <span class="detail">{{ event.detail }}</span>
          <span v-if="event.includedInPreview === false" class="skipped">
            (not in preview)
          </span>
        </div>
        <pre v-if="expanded[index]" class="transcript-payload">{{
          formatExpandedDetail(event)
        }}</pre>
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
.transcript {
  list-style: none;
  padding: 0;
  margin: 0;
}
.transcript-row {
  padding: 4px 0;
  border-bottom: 1px solid var(--vscode-widget-border, transparent);
}
.transcript-row.expandable {
  cursor: pointer;
}
.transcript-summary {
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
  gap: 6px;
}
.chevron {
  opacity: 0.7;
  width: 0.8em;
}
.stat-badge {
  font-family: var(--vscode-editor-font-family, monospace);
  font-size: 11px;
  padding: 0 4px;
  border-radius: 3px;
  background: var(--vscode-badge-background);
  color: var(--vscode-badge-foreground);
}
.stat-badge.add {
  color: var(--vscode-gitDecoration-addedResourceForeground, inherit);
}
.stat-badge.remove {
  color: var(--vscode-gitDecoration-deletedResourceForeground, inherit);
}
.detail {
  word-break: break-word;
}
.transcript-payload {
  margin: 6px 0 0;
  padding: 8px;
  max-height: 240px;
  overflow: auto;
  white-space: pre-wrap;
  word-break: break-word;
  font-size: 11px;
  background: var(--vscode-textCodeBlock-background, rgba(127, 127, 127, 0.12));
  border-radius: 4px;
}
.transcript-log {
  color: var(--vscode-descriptionForeground);
}
.transcript-log code {
  color: var(--vscode-textLink-foreground);
}
.secondary.active {
  font-weight: 600;
}
</style>
