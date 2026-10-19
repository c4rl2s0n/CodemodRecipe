import { computed, ref, watch, type Ref } from 'vue';
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
import { useExtensionClient } from '../composables/useExtensionClient';
import type { FileCardSelection } from '../lib/selection';
import { defaultFileSelections } from '../lib/selection';
import {
  buildCoverageTree,
  type CoverageTreeNode,
} from '../lib/coverageTree';

export type ComponentsViewProps = {
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
};

export function useComponentsView(
  props: ComponentsViewProps,
  emit: (event: 'update:args', value: ArgValues) => void
) {
const client = useExtensionClient();
const expanded = ref<Record<string, boolean>>({});
const fileSelections = ref<FileCardSelection[]>([]);
const activeChangeIndex = ref(0);
const status = ref('');
const showCoverageView = ref(false);
const contextMenu = ref<{
  x: number;
  y: number;
  kind: 'component' | 'instance';
  componentId: string;
  instanceId?: string;
} | null>(null);

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
  () => [props.opError, props.dryRunResult] as const,
  () => {
    if (status.value) {
      status.value = '';
    }
  }
);

const selectedInstance = computed(() =>
  props.instances.find((i) => i.id === props.selectedInstanceId)
);

const actionArgs = computed(() => {
  if (props.stackAction && props.stack?.operations) {
    return (
      props.stack.operations.find((o) => o.name === props.stackAction)?.args ??
      []
    );
  }
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

const canRunStackOperation = computed(() => !!props.stackAction);

const canPreview = computed(
  () =>
    canRunScaffold.value || canRunOperation.value || canRunStackOperation.value
);

const showReview = computed(
  () =>
    props.files.length > 0 ||
    props.transcript.length > 0 ||
    props.dryRunResult !== undefined
);

const dryRunResultText = computed(() => {
  if (props.dryRunResult === undefined) {
    return '';
  }
  return pretty(props.dryRunResult);
});

const coverageEnabled = computed(
  () =>
    !!props.coverage &&
    props.coverage.ok &&
    !props.coverage.skipped &&
    !!props.coverage.aggregate
);

const coverageChipLabel = computed(() => {
  if (!props.coverage) {
    return 'Coverage…';
  }
  if (props.coverage.skipped) {
    return 'Coverage off';
  }
  if (!props.coverage.ok) {
    return 'Coverage error';
  }
  const ratio = props.coverage.aggregate?.ratio;
  if (ratio === undefined) {
    return 'Coverage';
  }
  return `Coverage ${Math.round(ratio * 100)}%`;
});

const coverageChipTitle = computed(() => {
  if (props.coverage?.skipped) {
    return props.coverage.warning ?? 'Add stack.codebase to enable coverage';
  }
  if (props.coverage?.error) {
    return props.coverage.error;
  }
  return 'Open coverage file tree';
});

const coverageTree = computed(() =>
  buildCoverageTree(props.coverage?.files ?? [])
);

function openCoverageView(): void {
  if (props.coverage?.skipped || !props.coverage?.ok) {
    if (!props.coverage) {
      client.componentCoverage();
    }
    return;
  }
  showCoverageView.value = true;
}

function closeCoverageView(): void {
  showCoverageView.value = false;
  client.clearCoverageGaps();
}

function onCoverageOpenFile(node: CoverageTreeNode): void {
  if (node.kind !== 'file') {
    return;
  }
  client.openCoverageGaps(node.path, node.gaps ?? []);
}

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
  closeContextMenu();
}

function selectInstance(instanceId: string): void {
  client.selectComponentInstance(instanceId);
  closeContextMenu();
}

function selectAction(action: string): void {
  client.selectComponentAction(action);
}

function selectStackOp(operation: string): void {
  client.selectStackOperation(operation);
}

function rescan(): void {
  client.componentScan();
}

function runDryRun(hook: string): void {
  status.value = `Dry-run ${hook}…`;
  closeContextMenu();
  client.componentDryRun(hook, client.nextRequestId());
}

function onKindContextMenu(event: MouseEvent, componentId: string): void {
  event.preventDefault();
  selectComponent(componentId);
  contextMenu.value = {
    x: event.clientX,
    y: event.clientY,
    kind: 'component',
    componentId,
  };
}

function onInstanceContextMenu(
  event: MouseEvent,
  componentId: string,
  instanceId: string
): void {
  event.preventDefault();
  selectInstance(instanceId);
  contextMenu.value = {
    x: event.clientX,
    y: event.clientY,
    kind: 'instance',
    componentId,
    instanceId,
  };
}

function closeContextMenu(): void {
  contextMenu.value = null;
}

const actionRecipe = computed(() => {
  if (props.stackAction && props.stack) {
    return {
      id: props.stack.id,
      name: props.stackAction,
      description: '',
      args: actionArgs.value,
    };
  }
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

function runPreview(): void {
  if (!canPreview.value) {
    return;
  }
  status.value = 'Previewing…';
  if (props.stackAction) {
    client.stackPreview(props.args, client.nextRequestId());
  } else {
    client.componentPreview(props.args, client.nextRequestId());
  }
}

function runApply(): void {
  if (!canPreview.value) {
    return;
  }
  status.value = 'Applying…';
  if (props.stackAction) {
    client.stackApply(props.args, client.nextRequestId());
  } else {
    client.componentApply(props.args, client.nextRequestId());
  }
}

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
  const ast = instance.ast as { path?: string; source?: { path?: string } } | null;
  if (ast?.path) {
    return ast.path;
  }
  if (ast?.source?.path) {
    return ast.source.path;
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

  return {
    client,
    expanded,
    fileSelections,
    activeChangeIndex,
    status,
    showCoverageView,
    contextMenu,
    selectedInstance,
    actionArgs,
    canRunScaffold,
    canRunOperation,
    canRunStackOperation,
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
  };
}
