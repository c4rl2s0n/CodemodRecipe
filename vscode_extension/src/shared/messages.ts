import type { FilePreview, SelectionPayload, RecipeViewState } from './types';

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
  | { type: typeof WEBVIEW_TO_EXTENSION.pickFile; arg: string }
  | { type: typeof WEBVIEW_TO_EXTENSION.pickDirectory; arg: string }
  | {
      type: typeof WEBVIEW_TO_EXTENSION.preview;
      args: Record<string, string>;
      requestId?: number;
    }
  | { type: typeof WEBVIEW_TO_EXTENSION.openDiff; path: string; patchIndex: number }
  | { type: typeof WEBVIEW_TO_EXTENSION.apply; selection: SelectionPayload }
  | {
      type: typeof WEBVIEW_TO_EXTENSION.invokeRecipe;
      recipeId: string;
      mode: 'auto' | 'run' | 'open';
      args?: Record<string, string>;
    }
  | {
      type: typeof WEBVIEW_TO_EXTENSION.createShortcut;
      recipeId: string;
      args?: Record<string, string>;
    };

export type ExtensionToWebviewMessage =
  | { type: typeof EXTENSION_TO_WEBVIEW.state; state: RecipeViewState }
  | { type: typeof EXTENSION_TO_WEBVIEW.filePicked; arg: string; value: string }
  | {
      type: typeof EXTENSION_TO_WEBVIEW.previewResult;
      files: FilePreview[];
      requestId?: number;
      argsKey?: string;
    }
  | { type: typeof EXTENSION_TO_WEBVIEW.applyResult; applied: string[] }
  | {
      type: typeof EXTENSION_TO_WEBVIEW.error;
      message: string;
      requestId?: number;
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

function isStringRecord(value: unknown): value is Record<string, string> {
  if (!isRecord(value)) {
    return false;
  }
  return Object.values(value).every((v) => typeof v === 'string');
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
      return typeof value.arg === 'string';
    case WEBVIEW_TO_EXTENSION.preview:
      return (
        isStringRecord(value.args) &&
        (value.requestId === undefined || typeof value.requestId === 'number')
      );
    case WEBVIEW_TO_EXTENSION.openDiff:
      return typeof value.path === 'string' && typeof value.patchIndex === 'number';
    case WEBVIEW_TO_EXTENSION.apply:
      return isSelectionPayload(value.selection);
    case WEBVIEW_TO_EXTENSION.invokeRecipe:
      return (
        typeof value.recipeId === 'string' &&
        (value.mode === 'auto' ||
          value.mode === 'run' ||
          value.mode === 'open') &&
        (value.args === undefined || isStringRecord(value.args))
      );
    case WEBVIEW_TO_EXTENSION.createShortcut:
      return (
        typeof value.recipeId === 'string' &&
        (value.args === undefined || isStringRecord(value.args))
      );
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
      return typeof value.arg === 'string' && typeof value.value === 'string';
    case EXTENSION_TO_WEBVIEW.previewResult:
      return (
        Array.isArray(value.files) &&
        (value.requestId === undefined || typeof value.requestId === 'number') &&
        (value.argsKey === undefined || typeof value.argsKey === 'string')
      );
    case EXTENSION_TO_WEBVIEW.applyResult:
      return (
        Array.isArray(value.applied) &&
        value.applied.every((item) => typeof item === 'string')
      );
    case EXTENSION_TO_WEBVIEW.error:
      return (
        typeof value.message === 'string' &&
        (value.requestId === undefined || typeof value.requestId === 'number')
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

