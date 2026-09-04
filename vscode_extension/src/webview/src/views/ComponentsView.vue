<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import type {
  ArgValues,
  ComponentDetailSchema,
  ComponentFinding,
  ComponentInstance,
  ComponentSchema,
  FilePreview,
  StackSchema,
  TranscriptEvent,
} from '../shared';
import RecipeArgForm from '../components/RecipeArgForm.vue';
import ReviewPanel from '../components/ReviewPanel.vue';
import { useExtensionClient } from '../composables/useExtensionClient';
import type { FileCardSelection } from '../lib/selection';
import { defaultFileSelections } from '../lib/selection';

const props = defineProps<{
  components: readonly ComponentSchema[];
  stack?: StackSchema;
  instances: readonly ComponentInstance[];
  findings: readonly ComponentFinding[];
  selectedComponentId?: string;
  selectedInstanceId?: string;
  detail?: ComponentDetailSchema;
  action?: string;
  args: ArgValues;
  files: readonly FilePreview[];
  transcript: readonly TranscriptEvent[];
  scanError?: string;
  opError?: string;
}>();

const emit = defineEmits<{
  'update:args': [value: ArgValues];
}>();

const client = useExtensionClient();
const expanded = ref<Record<string, boolean>>({});
const fileSelections = ref<FileCardSelection[]>([]);
const activeChangeIndex = ref(0);
const status = ref('');

watch(
  () => props.files,
  (files) => {
    fileSelections.value = defaultFileSelections([...files]);
    activeChangeIndex.value = 0;
    if (status.value) {
      status.value = '';
    }
  },
  { immediate: true }
);

watch(
  () => props.opError,
  (err) => {
    if (err && status.value) {
      status.value = '';
    }
  }
);

const selectedInstance = computed(() =>
  props.instances.find((i) => i.id === props.selectedInstanceId)
);

const actionArgs = computed(() => {
  if (!props.detail || !props.action) {
    return [];
  }
  if (props.action === 'scaffold') {
    return props.detail.scaffoldArgs;
  }
  return (
    props.detail.operations.find((o) => o.name === props.action)?.args ?? []
  );
});

const canRunScaffold = computed(
  () => props.action === 'scaffold' && !!props.detail?.hasScaffold
);

const canRunOperation = computed(
  () =>
    !!props.action &&
    props.action !== 'scaffold' &&
    !!props.selectedInstanceId
);

const canPreview = computed(() => canRunScaffold.value || canRunOperation.value);

const showReview = computed(() => props.files.length > 0);

function instancesFor(componentId: string): ComponentInstance[] {
  return props.instances.filter((i) => i.componentId === componentId);
}

function toggle(componentId: string): void {
  expanded.value = {
    ...expanded.value,
    [componentId]: !expanded.value[componentId],
  };
}

function selectComponent(componentId: string): void {
  expanded.value = { ...expanded.value, [componentId]: true };
  client.selectComponent(componentId);
}

function selectInstance(instanceId: string): void {
  client.selectComponentInstance(instanceId);
}

function selectAction(action: string): void {
  client.selectComponentAction(action);
}

function rescan(): void {
  client.componentScan();
}

const actionRecipe = computed(() => {
  if (!props.detail || !props.action) {
    return undefined;
  }
  return {
    id: props.detail.id,
    name: props.action,
    description: '',
    args: actionArgs.value,
  };
});

const localArgs = computed({
  get: () => props.args,
  set: (value: ArgValues) => emit('update:args', value),
});

function onArgsChanged(): void {
  // Args already updated via v-model; no auto-preview for components.
}

function pathHint(instance: ComponentInstance): string {
  const loc = instance.locator as { path?: string; source?: { path?: string } } | null;
  if (loc?.path) {
    return loc.path;
  }
  if (loc?.source?.path) {
    return loc.source.path;
  }
  const node = instance.node as { path?: string; source?: { path?: string } } | null;
  if (node?.path) {
    return node.path;
  }
  if (node?.source?.path) {
    return node.source.path;
  }
  return instance.id;
}

function summaryTitle(instance: ComponentInstance): string {
  const summary = instance.summary as { title?: string; id?: string } | null;
  if (summary?.title) {
    return String(summary.title);
  }
  if (summary?.id) {
    return String(summary.id);
  }
  return instance.id;
}

function pretty(value: unknown): string {
  try {
    return JSON.stringify(value, null, 2);
  } catch {
    return String(value);
  }
}

function runPreview(): void {
  if (!canPreview.value) {
    return;
  }
  status.value = 'Previewing…';
  client.componentPreview(props.args, client.nextRequestId());
}

function runApply(): void {
  if (!canPreview.value) {
    return;
  }
  status.value = 'Applying…';
  client.componentApply(props.args, client.nextRequestId());
}
</script>

<template>
  <div class="components-layout">
    <div class="components-tree">
      <div class="tree-header">
        <h2>Components</h2>
        <button type="button" class="secondary" @click="rescan">Rescan</button>
      </div>
      <p v-if="stack" class="desc">Stack: {{ stack.id }}</p>
      <p v-if="scanError" class="error">{{ scanError }}</p>
      <p v-if="!components.length" class="desc">
        No components loaded. Add <code>.codemod/stack.luau</code> and component modules.
      </p>
      <ul class="kind-list">
        <li v-for="kind in components" :key="kind.id">
          <div
            class="kind-row"
            :class="{ selected: selectedComponentId === kind.id && !selectedInstanceId }"
          >
            <button type="button" class="expand" @click="toggle(kind.id)">
              {{ expanded[kind.id] ? '▾' : '▸' }}
            </button>
            <button type="button" class="kind-label" @click="selectComponent(kind.id)">
              <span class="id">{{ kind.id }}</span>
              <span v-if="kind.isRoot" class="badge">root</span>
            </button>
          </div>
          <ul v-if="expanded[kind.id]" class="instance-list">
            <li
              v-for="inst in instancesFor(kind.id)"
              :key="inst.id"
              class="instance-row"
              :class="{ selected: selectedInstanceId === inst.id }"
            >
              <button type="button" @click="selectInstance(inst.id)">
                <span class="title">{{ summaryTitle(inst) }}</span>
                <span class="path">{{ pathHint(inst) }}</span>
              </button>
            </li>
            <li v-if="!instancesFor(kind.id).length" class="empty">No instances</li>
          </ul>
        </li>
      </ul>
    </div>

    <div class="components-detail">
      <template v-if="selectedComponentId && !detail">
        <h2>{{ selectedComponentId }}</h2>
        <p v-if="opError" class="error">{{ opError }}</p>
        <p v-else class="desc">Loading component details…</p>
      </template>
      <template v-else-if="selectedComponentId && detail">
        <h2>{{ detail.id }}</h2>
        <p v-if="detail.description" class="desc">{{ detail.description }}</p>
        <p class="meta">
          <span v-if="detail.isRoot">root</span>
          <span v-if="detail.hasScaffold">scaffold</span>
          <span v-if="detail.sourceFile">{{ detail.sourceFile }}</span>
        </p>

        <template v-if="selectedInstance">
          <h3>Instance</h3>
          <p class="desc">{{ selectedInstance.id }}</p>
          <details open>
            <summary>Locator</summary>
            <pre>{{ pretty(selectedInstance.locator) }}</pre>
          </details>
          <details>
            <summary>Node</summary>
            <pre>{{ pretty(selectedInstance.node) }}</pre>
          </details>
          <details v-if="selectedInstance.summary">
            <summary>Summary</summary>
            <pre>{{ pretty(selectedInstance.summary) }}</pre>
          </details>
        </template>

        <h3>Actions</h3>
        <div class="actions">
          <button
            v-if="detail.hasScaffold"
            type="button"
            class="secondary"
            :class="{ active: action === 'scaffold' }"
            @click="selectAction('scaffold')"
          >
            scaffold
          </button>
          <button
            v-for="op in detail.operations"
            :key="op.name"
            type="button"
            class="secondary"
            :class="{ active: action === op.name }"
            :disabled="!selectedInstanceId"
            @click="selectAction(op.name)"
          >
            {{ op.name }}
          </button>
        </div>

        <template v-if="action">
          <h3>{{ action === 'scaffold' ? 'Scaffold' : action }}</h3>
          <RecipeArgForm
            v-if="actionRecipe"
            v-model:arg-values="localArgs"
            :recipe="actionRecipe"
            @args-changed="onArgsChanged"
          />
          <div class="run-row">
            <button type="button" :disabled="!canPreview" @click="runPreview">
              Preview
            </button>
            <button
              type="button"
              class="secondary"
              :disabled="!canPreview"
              @click="runApply"
            >
              Apply
            </button>
            <span v-if="status" class="status">{{ status }}</span>
          </div>
          <p v-if="opError" class="error">{{ opError }}</p>
        </template>

        <ReviewPanel
          v-if="showReview"
          :files="[...files]"
          :transcript="[...transcript]"
          :apply-may-diverge="false"
          :file-selections="fileSelections"
          :active-change-index="activeChangeIndex"
          :can-apply="false"
          @update:file-selections="fileSelections = $event"
          @update:active-change-index="activeChangeIndex = $event"
        />

        <template v-if="findings.length">
          <h3>Findings</h3>
          <ul class="findings">
            <li
              v-for="(f, i) in findings.filter(
                (x) =>
                  !selectedInstanceId || x.instanceId === selectedInstanceId
              )"
              :key="i"
            >
              <strong>{{ f.severity }}</strong> {{ f.code }}: {{ f.message }}
            </li>
          </ul>
        </template>
      </template>
      <p v-else class="desc">Select a component or instance.</p>
    </div>
  </div>
</template>

<style scoped>
.components-layout {
  display: grid;
  grid-template-columns: minmax(180px, 1fr) minmax(220px, 1.4fr);
  gap: 12px;
  align-items: start;
}
.tree-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}
.kind-list,
.instance-list,
.findings {
  list-style: none;
  padding: 0;
  margin: 0;
}
.kind-row,
.instance-row button {
  display: flex;
  align-items: center;
  gap: 4px;
  width: 100%;
  text-align: left;
}
.kind-row.selected,
.instance-row.selected button {
  outline: 1px solid var(--vscode-focusBorder, #007fd4);
}
.expand,
.kind-label,
.instance-row button {
  background: transparent;
  border: none;
  color: inherit;
  cursor: pointer;
  padding: 4px 2px;
}
.id {
  font-family: var(--vscode-editor-font-family, monospace);
  font-size: 12px;
}
.badge {
  font-size: 10px;
  opacity: 0.7;
  margin-left: 4px;
}
.path,
.title {
  display: block;
  font-size: 11px;
}
.path {
  opacity: 0.7;
}
.instance-list {
  margin-left: 16px;
  margin-bottom: 8px;
}
.empty {
  font-size: 11px;
  opacity: 0.6;
  padding: 4px;
}
.meta {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  font-size: 11px;
  opacity: 0.8;
}
.actions {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-bottom: 8px;
}
.actions .active {
  outline: 1px solid var(--vscode-focusBorder, #007fd4);
}
.run-row {
  display: flex;
  align-items: center;
  gap: 8px;
  margin: 8px 0;
}
pre {
  max-height: 200px;
  overflow: auto;
  font-size: 11px;
  white-space: pre-wrap;
}
.error {
  color: var(--vscode-errorForeground, #f44747);
}
@media (max-width: 520px) {
  .components-layout {
    grid-template-columns: 1fr;
  }
}
</style>
