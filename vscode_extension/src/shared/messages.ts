import type {
  ArgValues,
  FilePreview,
  SelectionPayload,
  RecipeViewState,
  TranscriptEvent,
} from './types';

export const WEBVIEW_TO_EXTENSION = {
  ready: 'ready',
  bootstrapRetry: 'bootstrapRetry',
  selectRecipe: 'selectRecipe',
  refreshRecipes: 'refreshRecipes',
  configureHost: 'configureHost',
  scaffoldProject: 'scaffoldProject',
  openRecipeFile: 'openRecipeFile',
  pickFile: 'pickFile',
  pickDirectory: 'pickDirectory',
  preview: 'preview',
  openDiff: 'openDiff',
  apply: 'apply',
  invokeRecipe: 'invokeRecipe',
  createShortcut: 'createShortcut',
  selectComponent: 'selectComponent',
  selectComponentInstance: 'selectComponentInstance',
  selectComponentAction: 'selectComponentAction',
  componentScan: 'componentScan',
  componentPreview: 'componentPreview',
  componentApply: 'componentApply',
} as const;

export const EXTENSION_TO_WEBVIEW = {
  state: 'state',
  filePicked: 'filePicked',
  previewResult: 'previewResult',
  applyResult: 'applyResult',
  error: 'error',
  previewState: 'previewState',
} as const;

export type WebviewToExtensionMessage =
  | { type: typeof WEBVIEW_TO_EXTENSION.ready }
  | { type: typeof WEBVIEW_TO_EXTENSION.bootstrapRetry }
  | { type: typeof WEBVIEW_TO_EXTENSION.selectRecipe; id: string }
  | { type: typeof WEBVIEW_TO_EXTENSION.refreshRecipes }
  | { type: typeof WEBVIEW_TO_EXTENSION.configureHost }
  | { type: typeof WEBVIEW_TO_EXTENSION.scaffoldProject }
  | { type: typeof WEBVIEW_TO_EXTENSION.openRecipeFile; id: string }
  | {
      type: typeof WEBVIEW_TO_EXTENSION.pickFile;
      arg: string;
      requestId: number;
    }
  | {
      type: typeof WEBVIEW_TO_EXTENSION.pickDirectory;
      arg: string;
      requestId: number;
    }
  | {
      type: typeof WEBVIEW_TO_EXTENSION.preview;
      args: ArgValues;
      requestId: number;
    }
  | { type: typeof WEBVIEW_TO_EXTENSION.openDiff; path: string }
  | { type: typeof WEBVIEW_TO_EXTENSION.apply; selection: SelectionPayload }
  | {
      type: typeof WEBVIEW_TO_EXTENSION.invokeRecipe;
      recipeId: string;
      mode: 'auto' | 'run' | 'open';
      args?: ArgValues;
    }
  | {
      type: typeof WEBVIEW_TO_EXTENSION.createShortcut;
      recipeId: string;
      args?: ArgValues;
    }
  | {
      type: typeof WEBVIEW_TO_EXTENSION.selectComponent;
      componentId: string;
    }
  | {
      type: typeof WEBVIEW_TO_EXTENSION.selectComponentInstance;
      instanceId: string;
    }
  | {
      type: typeof WEBVIEW_TO_EXTENSION.selectComponentAction;
      action: string;
    }
  | { type: typeof WEBVIEW_TO_EXTENSION.componentScan }
  | {
      type: typeof WEBVIEW_TO_EXTENSION.componentPreview;
      args: ArgValues;
      requestId: number;
    }
  | {
      type: typeof WEBVIEW_TO_EXTENSION.componentApply;
      args: ArgValues;
      requestId: number;
    };

export type ExtensionToWebviewMessage =
  | { type: typeof EXTENSION_TO_WEBVIEW.state; state: RecipeViewState }
  | {
      type: typeof EXTENSION_TO_WEBVIEW.filePicked;
      arg: string;
      requestId: number;
      /** Present when the user selected a path. */
      value?: string;
      /** True when the dialog was dismissed. */
      cancelled?: boolean;
    }
  | {
      type: typeof EXTENSION_TO_WEBVIEW.previewResult;
      files: FilePreview[];
      transcript?: TranscriptEvent[];
      applyMayDiverge?: boolean;
      requestId: number;
      argsKey?: string;
    }
  | { type: typeof EXTENSION_TO_WEBVIEW.applyResult; applied: string[] }
  | {
      type: typeof EXTENSION_TO_WEBVIEW.error;
      message: string;
      requestId?: number;
      diagnostics?: import('./types').RecipeDiagnostic[];
      transcript?: import('./types').TranscriptEvent[];
    }
  | {
      type: typeof EXTENSION_TO_WEBVIEW.previewState;
      inFlight: boolean;
      requestId?: number;
    };

const WEBVIEW_TO_EXTENSION_TYPES = new Set<string>(
  Object.values(WEBVIEW_TO_EXTENSION)
);

const EXTENSION_TO_WEBVIEW_TYPES = new Set<string>(
  Object.values(EXTENSION_TO_WEBVIEW)
);

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null;
}

function isArgValue(value: unknown): value is import('./types').ArgValue {
  if (typeof value === 'string' || typeof value === 'boolean') {
    return true;
  }
  if (Array.isArray(value)) {
    return value.every(isArgValue);
  }
  if (isRecord(value)) {
    return Object.values(value).every(isArgValue);
  }
  return false;
}

function isArgRecord(value: unknown): value is ArgValues {
  if (!isRecord(value) || Array.isArray(value)) {
    return false;
  }
  return Object.values(value).every(isArgValue);
}

function isSelectionPayload(value: unknown): boolean {
  if (!isRecord(value) || !isRecord(value.files)) {
    return false;
  }
  for (const entry of Object.values(value.files)) {
    if (!isRecord(entry) || typeof entry.include !== 'boolean') {
      return false;
    }
  }
  return true;
}

export function isWebviewToExtensionMessage(
  value: unknown
): value is WebviewToExtensionMessage {
  if (!isRecord(value) || typeof value.type !== 'string') {
    return false;
  }
  if (!WEBVIEW_TO_EXTENSION_TYPES.has(value.type)) {
    return false;
  }
  switch (value.type) {
    case WEBVIEW_TO_EXTENSION.ready:
    case WEBVIEW_TO_EXTENSION.bootstrapRetry:
    case WEBVIEW_TO_EXTENSION.refreshRecipes:
    case WEBVIEW_TO_EXTENSION.configureHost:
    case WEBVIEW_TO_EXTENSION.scaffoldProject:
      return true;
    case WEBVIEW_TO_EXTENSION.selectRecipe:
    case WEBVIEW_TO_EXTENSION.openRecipeFile:
      return typeof value.id === 'string';
    case WEBVIEW_TO_EXTENSION.pickFile:
    case WEBVIEW_TO_EXTENSION.pickDirectory:
      return typeof value.arg === 'string' && typeof value.requestId === 'number';
    case WEBVIEW_TO_EXTENSION.preview:
      return isArgRecord(value.args) && typeof value.requestId === 'number';
    case WEBVIEW_TO_EXTENSION.openDiff:
      return typeof value.path === 'string';
    case WEBVIEW_TO_EXTENSION.apply:
      return isSelectionPayload(value.selection);
    case WEBVIEW_TO_EXTENSION.invokeRecipe:
      return (
        typeof value.recipeId === 'string' &&
        (value.mode === 'auto' ||
          value.mode === 'run' ||
          value.mode === 'open') &&
        (value.args === undefined || isArgRecord(value.args))
      );
    case WEBVIEW_TO_EXTENSION.createShortcut:
      return (
        typeof value.recipeId === 'string' &&
        (value.args === undefined || isArgRecord(value.args))
      );
    case WEBVIEW_TO_EXTENSION.selectComponent:
      return typeof value.componentId === 'string';
    case WEBVIEW_TO_EXTENSION.selectComponentInstance:
      return typeof value.instanceId === 'string';
    case WEBVIEW_TO_EXTENSION.selectComponentAction:
      return typeof value.action === 'string';
    case WEBVIEW_TO_EXTENSION.componentScan:
      return true;
    case WEBVIEW_TO_EXTENSION.componentPreview:
    case WEBVIEW_TO_EXTENSION.componentApply:
      return isArgRecord(value.args) && typeof value.requestId === 'number';
    default:
      return false;
  }
}

export function isExtensionToWebviewMessage(
  value: unknown
): value is ExtensionToWebviewMessage {
  if (!isRecord(value) || typeof value.type !== 'string') {
    return false;
  }
  if (!EXTENSION_TO_WEBVIEW_TYPES.has(value.type)) {
    return false;
  }
  switch (value.type) {
    case EXTENSION_TO_WEBVIEW.state:
      return isRecord(value.state);
    case EXTENSION_TO_WEBVIEW.filePicked:
      return (
        typeof value.arg === 'string' &&
        typeof value.requestId === 'number' &&
        (value.value === undefined || typeof value.value === 'string') &&
        (value.cancelled === undefined || typeof value.cancelled === 'boolean')
      );
    case EXTENSION_TO_WEBVIEW.previewResult:
      return (
        Array.isArray(value.files) &&
        typeof value.requestId === 'number' &&
        (value.argsKey === undefined || typeof value.argsKey === 'string') &&
        (value.transcript === undefined || Array.isArray(value.transcript)) &&
        (value.applyMayDiverge === undefined ||
          typeof value.applyMayDiverge === 'boolean')
      );
    case EXTENSION_TO_WEBVIEW.applyResult:
      return (
        Array.isArray(value.applied) &&
        value.applied.every((item) => typeof item === 'string')
      );
    case EXTENSION_TO_WEBVIEW.error:
      return (
        typeof value.message === 'string' &&
        (value.requestId === undefined || typeof value.requestId === 'number') &&
        (value.diagnostics === undefined || Array.isArray(value.diagnostics)) &&
        (value.transcript === undefined || Array.isArray(value.transcript))
      );
    case EXTENSION_TO_WEBVIEW.previewState:
      return (
        typeof value.inFlight === 'boolean' &&
        (value.requestId === undefined || typeof value.requestId === 'number')
      );
    default:
      return false;
  }
}

export function assertNever(value: never): never {
  throw new Error(`Unexpected value: ${String(value)}`);
}
