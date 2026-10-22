<script setup lang="ts">
import type {
  ArgValues,
  ComponentCoverageResponse,
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
import CoverageTree from '../components/CoverageTree.vue';
import { useComponentsView } from '../composables/useComponentsView';

const props = defineProps<{
  components: readonly ComponentSchema[];
  stack?: StackSchema;
  instances: readonly ComponentInstance[];
  findings: readonly ComponentFinding[];
  selectedComponentId?: string;
  selectedInstanceId?: string;
  detail?: ComponentDetailSchema;
  action?: string;
  stackAction?: string;
  args: ArgValues;
  files: readonly FilePreview[];
  transcript: readonly TranscriptEvent[];
  scanError?: string;
  opError?: string;
  dryRunResult?: unknown;
  dryRunHook?: string;
  coverage?: ComponentCoverageResponse;
}>();

const emit = defineEmits<{
  'update:args': [value: ArgValues];
}>();

const {
  client,
  expanded,
  fileSelections,
  activeChangeIndex,
  status,
  showCoverageView,
  contextMenu,
  selectedInstance,
  actionArgs,
  canPreview,
  showReview,
  dryRunResultText,
  coverageEnabled,
  coverageChipLabel,
  coverageChipTitle,
  coverageTree,
  openCoverageView,
  closeCoverageView,
  onCoverageOpenFile,
  instancesFor,
  toggle,
  selectComponent,
  selectInstance,
  selectAction,
  selectStackOp,
  rescan,
  runDryRun,
  onKindContextMenu,
  onInstanceContextMenu,
  closeContextMenu,
  actionRecipe,
  runPreview,
  runApply,
  localArgs,
  onArgsChanged,
  pathHint,
  summaryTitle,
  pretty,
} = useComponentsView(props, (e, v) => emit(e, v));
</script>


<template>
  <div class="components-layout" @click="closeContextMenu">
    <template v-if="showCoverageView && coverageEnabled">
      <div class="coverage-panel">
        <div class="tree-header">
          <button type="button" class="secondary" @click="closeCoverageView">
            ← Components
          </button>
          <h2>Coverage</h2>
          <button
            type="button"
            class="secondary"
            @click="client.componentCoverage()"
          >
            Refresh
          </button>
        </div>
        <p v-if="coverage?.aggregate" class="desc">
          Aggregate
          {{ Math.round(coverage.aggregate.ratio * 100) }}%
          ({{ coverage.aggregate.coveredNonblank }} /
          {{
            coverage.aggregate.coveredNonblank + coverage.aggregate.gapNonblank
          }}
          nonblank)
        </p>
        <CoverageTree
          v-if="coverageTree.length"
          :nodes="coverageTree"
          @open-file="onCoverageOpenFile"
        />
        <p v-else class="desc">No files in stack.codebase universe.</p>
      </div>
    </template>
    <template v-else>
    <div class="components-tree">
      <div class="tree-header">
        <h2>Components</h2>
        <div class="header-actions">
          <button
            type="button"
            class="coverage-chip"
            :class="{
              enabled: coverageEnabled,
              skipped: coverage?.skipped,
              error: coverage && !coverage.ok && !coverage.skipped,
            }"
            :title="coverageChipTitle"
            @click="openCoverageView"
          >
            {{ coverageChipLabel }}
          </button>
          <button type="button" class="secondary" @click="rescan">Rescan</button>
        </div>
      </div>
      <p v-if="stack" class="desc">Stack: {{ stack.id }}</p>
      <div v-if="stack?.operations?.length" class="stack-ops">
        <h3>Stack operations</h3>
        <p class="desc">Instance-less fan-out across layers (dry-run → diff → apply).</p>
        <div class="actions">
          <button
            v-for="op in stack.operations"
            :key="op.name"
            type="button"
            class="secondary"
            :class="{ active: stackAction === op.name }"
            @click="selectStackOp(op.name)"
          >
            {{ op.name }}
          </button>
        </div>
        <template v-if="stackAction">
          <h3>{{ stackAction }}</h3>
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
          <ReviewPanel
            v-if="showReview && stackAction"
            :files="[...files]"
            :transcript="[...transcript]"
            :apply-may-diverge="false"
            :file-selections="fileSelections"
            :active-change-index="activeChangeIndex"
            :can-apply="false"
            @update:file-selections="fileSelections = $event"
            @update:active-change-index="activeChangeIndex = $event"
          />
        </template>
      </div>
      <p v-if="coverage?.skipped" class="desc cov-warn">
        {{ coverage.warning }}
      </p>
      <p v-if="scanError" class="error">{{ scanError }}</p>
      <p v-if="!components.length" class="desc">
        No components loaded. Add <code>.codemod/stack.luau</code> and component modules.
      </p>
      <ul class="kind-list">
        <li v-for="kind in components" :key="kind.id">
          <div
            class="kind-row"
            :class="{ selected: selectedComponentId === kind.id && !selectedInstanceId }"
            @contextmenu="onKindContextMenu($event, kind.id)"
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
              @contextmenu="onInstanceContextMenu($event, kind.id, inst.id)"
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
            <pre>{{ pretty(selectedInstance.ast) }}</pre>
          </details>
          <details v-if="selectedInstance.summary">
            <summary>Summary</summary>
            <pre>{{ pretty(selectedInstance.summary) }}</pre>
          </details>
        </template>

        <h3>Debug (dry-run)</h3>
        <p class="desc">
          Runs hooks under Preview WorkingTree and shows result + transcript (<code>c:log</code>).
        </p>
        <div class="actions">
          <button
            v-if="detail.hasIdentify"
            type="button"
            class="secondary"
            @click="runDryRun('identify')"
          >
            identify
          </button>
          <button
            v-if="selectedInstanceId"
            type="button"
            class="secondary"
            @click="runDryRun('parse')"
          >
            parse
          </button>
          <button
            v-if="selectedInstanceId"
            type="button"
            class="secondary"
            @click="runDryRun('verify')"
          >
            verify
          </button>
          <button
            v-if="detail.hasToSummary && selectedInstanceId"
            type="button"
            class="secondary"
            @click="runDryRun('toSummary')"
          >
            toSummary
          </button>
          <button
            v-if="detail.hasScaffold"
            type="button"
            class="secondary"
            @click="runDryRun('scaffold')"
          >
            scaffold
          </button>
          <button
            v-if="action && action !== 'scaffold' && selectedInstanceId"
            type="button"
            class="secondary"
            @click="runDryRun('operation')"
          >
            op: {{ action }}
          </button>
        </div>
        <p v-if="status" class="status">{{ status }}</p>
        <p v-if="opError" class="error">{{ opError }}</p>
        <details v-if="dryRunResultText" open class="dry-run-result">
          <summary>Result{{ dryRunHook ? ` (${dryRunHook})` : '' }}</summary>
          <pre>{{ dryRunResultText }}</pre>
        </details>

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
    </template>

    <div
      v-if="contextMenu"
      class="context-menu"
      :style="{ left: `${contextMenu.x}px`, top: `${contextMenu.y}px` }"
      @click.stop
    >
      <template v-if="contextMenu.kind === 'component'">
        <button
          v-if="
            components.find((c) => c.id === contextMenu!.componentId)?.isRoot
          "
          type="button"
          @click="runDryRun('identify')"
        >
          Dry-run identify
        </button>
        <button
          v-if="
            components.find((c) => c.id === contextMenu!.componentId)
              ?.hasScaffold
          "
          type="button"
          @click="runDryRun('scaffold')"
        >
          Dry-run scaffold
        </button>
      </template>
      <template v-else>
        <button type="button" @click="runDryRun('parse')">Dry-run parse</button>
        <button type="button" @click="runDryRun('verify')">Dry-run verify</button>
        <button
          v-if="detail?.hasToSummary"
          type="button"
          @click="runDryRun('toSummary')"
        >
          Dry-run toSummary
        </button>
      </template>
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
.header-actions {
  display: flex;
  align-items: center;
  gap: 6px;
}
.coverage-chip {
  font-size: 11px;
  padding: 3px 8px;
  border-radius: 4px;
  border: 1px solid var(--vscode-button-secondaryBorder, transparent);
  background: var(--vscode-button-secondaryBackground, transparent);
  color: var(--vscode-button-secondaryForeground, inherit);
  cursor: pointer;
  opacity: 0.85;
}
.coverage-chip.enabled {
  opacity: 1;
  border-color: var(--cov-green, #3d9a5f);
}
.coverage-chip.skipped {
  opacity: 0.65;
}
.coverage-chip.error {
  border-color: var(--cov-red, #c44b4b);
}
.cov-warn {
  color: var(--vscode-editorWarning-foreground, #c4a035);
}
.coverage-panel {
  grid-column: 1 / -1;
  --cov-green: #3d9a5f;
  --cov-yellow: #c4a035;
  --cov-red: #c44b4b;
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
.context-menu {
  position: fixed;
  z-index: 20;
  min-width: 160px;
  padding: 4px;
  background: var(--vscode-menu-background, #252526);
  border: 1px solid var(--vscode-menu-border, #454545);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.36);
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.context-menu button {
  text-align: left;
  background: transparent;
  border: none;
  color: var(--vscode-menu-foreground, #cccccc);
  padding: 4px 8px;
  cursor: pointer;
}
.context-menu button:hover {
  background: var(--vscode-menu-selectionBackground, #094771);
}
.dry-run-result {
  margin: 8px 0;
}
@media (max-width: 520px) {
  .components-layout {
    grid-template-columns: 1fr;
  }
}
</style>
